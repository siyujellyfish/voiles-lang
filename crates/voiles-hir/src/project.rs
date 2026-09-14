use std::collections::{HashMap, HashSet};

use voiles_lexer::{Diagnostic, Span, Token, TokenKind};
use voiles_syntax::{Parse, SyntaxKind, SyntaxNode, ast::token_text};

use crate::{
	BuiltinType, DependencyEdge, DependencyKind, ExportedSymbol, GlobalSymbolId, HirProject,
	HirProjectResult, ImportBinding, ImportBindingKind, ModuleId, ModuleSource, ModuleSourceId,
	ProjectDiagnostic, ProjectModule, SymbolId, SymbolKind, TypeReference, TypeTarget, resolver,
};

#[must_use]
pub fn resolve_project(sources: Vec<ModuleSource>) -> HirProjectResult {
	let mut diagnostics = Vec::new();
	let mut unique_sources = Vec::new();
	let mut source_indices = HashMap::new();

	for source in sources {
		let source_id = ModuleSourceId(normalize_path(source.id.as_str()));
		if source_indices.contains_key(&source_id) {
			diagnostics.push(project_error(
				&source_id,
				"VHIR007",
				"duplicate module source identity",
				Span::new(0, 0),
			));
			continue;
		}
		let index = unique_sources.len();
		source_indices.insert(source_id.clone(), index);
		unique_sources.push(ModuleSource {
			id: source_id,
			source: source.source,
		});
	}

	let mut parses = Vec::with_capacity(unique_sources.len());
	let mut modules = Vec::with_capacity(unique_sources.len());
	for (index, source) in unique_sources.into_iter().enumerate() {
		let parsed = voiles_syntax::parse(&source.source);
		let resolved = resolver::resolve(&source.source, &parsed);
		for diagnostic in &resolved.diagnostics {
			diagnostics.push(ProjectDiagnostic {
				source: source.id.clone(),
				diagnostic: diagnostic.clone(),
			});
		}
		parses.push(parsed);
		modules.push(ProjectModule {
			id: ModuleId(index),
			source_id: source.id,
			source: source.source,
			hir: resolved.module,
			exports: Vec::new(),
			imports: Vec::new(),
			type_references: Vec::new(),
		});
	}

	for index in 0..modules.len() {
		modules[index].exports = collect_exports(ModuleId(index), &modules[index], &parses[index]);
	}

	let export_tables: Vec<HashMap<String, ExportedSymbol>> = modules
		.iter()
		.map(|module| {
			module
				.exports
				.iter()
				.cloned()
				.map(|export| (export.name.clone(), export))
				.collect()
		})
		.collect();
	let source_lookup: HashMap<ModuleSourceId, ModuleId> = modules
		.iter()
		.map(|module| (module.source_id.clone(), module.id))
		.collect();

	for index in 0..modules.len() {
		modules[index].imports = collect_imports(
			ModuleId(index),
			&modules[index],
			&parses[index],
			&source_lookup,
			&export_tables,
			&mut diagnostics,
		);
	}

	for index in 0..modules.len() {
		modules[index].type_references = collect_type_references(
			ModuleId(index),
			&modules[index],
			&parses[index],
			&export_tables,
			&mut diagnostics,
		);
	}

	let dependencies = classify_dependencies(&modules);
	let project = HirProject {
		modules,
		dependencies,
	};
	detect_runtime_cycles(&project, &mut diagnostics);

	HirProjectResult {
		project,
		diagnostics,
	}
}

fn collect_exports(
	module_id: ModuleId,
	module: &ProjectModule,
	parsed: &Parse,
) -> Vec<ExportedSymbol> {
	let mut exports = Vec::new();
	for child in parsed.root.child_nodes() {
		let declaration = match child.kind {
			SyntaxKind::ComponentDecl => Some(child),
			SyntaxKind::ExportDecl => child.child_nodes().next(),
			_ => None,
		};
		let Some(declaration) = declaration else {
			continue;
		};
		let Some(name_token) = first_identifier(declaration) else {
			continue;
		};
		let Some(symbol) = module
			.hir
			.symbols
			.iter()
			.find(|symbol| symbol.span == name_token.span)
		else {
			continue;
		};
		exports.push(ExportedSymbol {
			name: token_text(&module.source, name_token).to_owned(),
			target: GlobalSymbolId {
				module: module_id,
				symbol: symbol.id,
			},
			kind: symbol.kind,
		});
	}
	exports
}

fn collect_imports(
	module_id: ModuleId,
	module: &ProjectModule,
	parsed: &Parse,
	source_lookup: &HashMap<ModuleSourceId, ModuleId>,
	export_tables: &[HashMap<String, ExportedSymbol>],
	diagnostics: &mut Vec<ProjectDiagnostic>,
) -> Vec<ImportBinding> {
	let mut imports = Vec::new();
	for node in parsed.root.child_nodes().filter(|node| {
		matches!(
			node.kind,
			SyntaxKind::ImportDecl | SyntaxKind::ExternImportDecl
		)
	}) {
		let Some(specifier_token) = node
			.direct_tokens()
			.find(|token| token.kind == TokenKind::String)
			.copied()
		else {
			continue;
		};
		let specifier = unquote(token_text(&module.source, specifier_token));
		let is_extern = node.kind == SyntaxKind::ExternImportDecl;
		let target_module = if !is_extern && specifier.starts_with('.') {
			let target_id = resolve_relative(&module.source_id, &specifier);
			match source_lookup.get(&target_id).copied() {
				Some(target) => Some(target),
				None => {
					diagnostics.push(project_error(
						&module.source_id,
						"VHIR008",
						format!("cannot resolve module `{specifier}`"),
						specifier_token.span,
					));
					None
				}
			}
		} else {
			None
		};

		for item in node.child_nodes() {
			match item.kind {
				SyntaxKind::ImportItem => {
					let identifiers: Vec<Token> = item
						.direct_tokens()
						.filter(|token| token.kind == TokenKind::Identifier)
						.copied()
						.collect();
					let (Some(imported_token), Some(local_token)) =
						(identifiers.first().copied(), identifiers.last().copied())
					else {
						continue;
					};
					let Some(local_symbol) = symbol_at_span(module, local_token.span) else {
						continue;
					};
					let imported_name = token_text(&module.source, imported_token).to_owned();
					let target_symbol = target_module.and_then(|target| {
						match export_tables[target.0].get(&imported_name) {
							Some(export) => Some(export.target),
							None => {
								diagnostics.push(project_error(
									&module.source_id,
									"VHIR009",
									format!(
										"module `{specifier}` does not export `{imported_name}`"
									),
									imported_token.span,
								));
								None
							}
						}
					});
					imports.push(ImportBinding {
						module: module_id,
						local_symbol,
						kind: ImportBindingKind::Named,
						imported_name: Some(imported_name),
						specifier: specifier.clone(),
						declaration_span: node.span,
						target_module,
						target_symbol,
					});
				}
				SyntaxKind::NamespaceImport => {
					let Some(local_token) = last_identifier(item) else {
						continue;
					};
					let Some(local_symbol) = symbol_at_span(module, local_token.span) else {
						continue;
					};
					imports.push(ImportBinding {
						module: module_id,
						local_symbol,
						kind: ImportBindingKind::Namespace,
						imported_name: None,
						specifier: specifier.clone(),
						declaration_span: node.span,
						target_module,
						target_symbol: None,
					});
				}
				_ => {}
			}
		}
	}
	imports
}

fn collect_type_references(
	module_id: ModuleId,
	module: &ProjectModule,
	parsed: &Parse,
	export_tables: &[HashMap<String, ExportedSymbol>],
	diagnostics: &mut Vec<ProjectDiagnostic>,
) -> Vec<TypeReference> {
	let mut references = Vec::new();
	scan_type_references(
		module_id,
		module,
		&parsed.root,
		&HashSet::new(),
		export_tables,
		diagnostics,
		&mut references,
	);
	references
}

fn scan_type_references(
	module_id: ModuleId,
	module: &ProjectModule,
	node: &SyntaxNode,
	generics: &HashSet<String>,
	export_tables: &[HashMap<String, ExportedSymbol>],
	diagnostics: &mut Vec<ProjectDiagnostic>,
	output: &mut Vec<TypeReference>,
) {
	let mut enum_generics = generics.clone();
	if node.kind == SyntaxKind::EnumDecl
		&& let Some(parameters) = node
			.child_nodes()
			.find(|child| child.kind == SyntaxKind::GenericParameterList)
	{
		for token in parameters
			.direct_tokens()
			.filter(|token| token.kind == TokenKind::Identifier)
		{
			enum_generics.insert(token_text(&module.source, *token).to_owned());
		}
	}
	let active_generics = if node.kind == SyntaxKind::EnumDecl {
		&enum_generics
	} else {
		generics
	};

	if node.kind == SyntaxKind::TypeRef {
		let identifiers: Vec<Token> = node
			.direct_tokens()
			.filter(|token| token.kind == TokenKind::Identifier)
			.copied()
			.collect();
		if !identifiers.is_empty() {
			let path: Vec<String> = identifiers
				.iter()
				.map(|token| token_text(&module.source, *token).to_owned())
				.collect();
			let (target, through_import) = resolve_type_path(
				module_id,
				module,
				&path,
				active_generics,
				export_tables,
				node.span,
				diagnostics,
			);
			output.push(TypeReference {
				module: module_id,
				path,
				span: node.span,
				target,
				through_import,
			});
		}
	}

	for child in node.child_nodes() {
		scan_type_references(
			module_id,
			module,
			child,
			active_generics,
			export_tables,
			diagnostics,
			output,
		);
	}
}

#[allow(clippy::too_many_arguments)]
fn resolve_type_path(
	module_id: ModuleId,
	module: &ProjectModule,
	path: &[String],
	generics: &HashSet<String>,
	export_tables: &[HashMap<String, ExportedSymbol>],
	span: Span,
	diagnostics: &mut Vec<ProjectDiagnostic>,
) -> (Option<TypeTarget>, Option<SymbolId>) {
	if path.len() == 1 {
		let name = &path[0];
		if generics.contains(name) {
			return (Some(TypeTarget::Generic(name.clone())), None);
		}
		if let Some(builtin) = BuiltinType::from_name(name) {
			return (Some(TypeTarget::Builtin(builtin)), None);
		}
		if let Some(symbol) = module.hir.symbols.iter().find(|symbol| {
			symbol.scope == module.hir.root_scope && symbol.name == *name && symbol.kind.is_type()
		}) {
			return (
				Some(TypeTarget::Symbol(GlobalSymbolId {
					module: module_id,
					symbol: symbol.id,
				})),
				None,
			);
		}
		if let Some(import) = module.imports.iter().find(|import| {
			module.hir.symbols[import.local_symbol.0].name == *name
				&& import.kind == ImportBindingKind::Named
		}) && let Some(target) = import.target_symbol
		{
			let target_kind = export_tables[target.module.0]
				.values()
				.find(|export| export.target == target)
				.map(|export| export.kind);
			if target_kind.is_some_and(SymbolKind::is_type) {
				return (Some(TypeTarget::Symbol(target)), Some(import.local_symbol));
			}
			diagnostics.push(project_error(
				&module.source_id,
				"VHIR011",
				format!("`{name}` does not refer to a type"),
				span,
			));
			return (None, Some(import.local_symbol));
		}
	}

	if path.len() == 2 {
		let namespace = &path[0];
		let member = &path[1];
		if let Some(import) = module.imports.iter().find(|import| {
			module.hir.symbols[import.local_symbol.0].name == *namespace
				&& import.kind == ImportBindingKind::Namespace
		}) && let Some(target_module) = import.target_module
			&& let Some(export) = export_tables[target_module.0].get(member)
		{
			if export.kind.is_type() {
				return (
					Some(TypeTarget::Symbol(export.target)),
					Some(import.local_symbol),
				);
			}
			diagnostics.push(project_error(
				&module.source_id,
				"VHIR011",
				format!("`{namespace}.{member}` does not refer to a type"),
				span,
			));
			return (None, Some(import.local_symbol));
		}
	}

	diagnostics.push(project_error(
		&module.source_id,
		"VHIR012",
		format!("unresolved type `{}`", path.join(".")),
		span,
	));
	(None, None)
}

fn classify_dependencies(modules: &[ProjectModule]) -> Vec<DependencyEdge> {
	let mut dependencies = Vec::new();
	for module in modules {
		let mut seen = HashSet::new();
		for import in &module.imports {
			let key = (
				import.declaration_span.start,
				import.declaration_span.end,
				import.specifier.clone(),
			);
			if !seen.insert(key) {
				continue;
			}
			let group: Vec<&ImportBinding> = module
				.imports
				.iter()
				.filter(|candidate| {
					candidate.declaration_span == import.declaration_span
						&& candidate.specifier == import.specifier
				})
				.collect();

			if !import.specifier.starts_with('.') {
				dependencies.push(DependencyEdge {
					from: module.id,
					to: None,
					kind: DependencyKind::External,
					specifier: import.specifier.clone(),
					span: import.declaration_span,
				});
				continue;
			}
			let Some(target) = import.target_module else {
				continue;
			};
			let all_type_only = group
				.iter()
				.all(|binding| binding_is_type_only(modules, module, binding));
			dependencies.push(DependencyEdge {
				from: module.id,
				to: Some(target),
				kind: if all_type_only {
					DependencyKind::TypeOnly
				} else {
					DependencyKind::Runtime
				},
				specifier: import.specifier.clone(),
				span: import.declaration_span,
			});
		}
	}
	dependencies
}

fn binding_is_type_only(
	modules: &[ProjectModule],
	module: &ProjectModule,
	binding: &ImportBinding,
) -> bool {
	let has_value_reference = module
		.hir
		.references
		.iter()
		.any(|reference| reference.resolved == Some(binding.local_symbol));
	if has_value_reference {
		return false;
	}
	match binding.kind {
		ImportBindingKind::Named => binding.target_symbol.is_some_and(|target| {
			modules[target.module.0].hir.symbols[target.symbol.0]
				.kind
				.is_type()
		}),
		ImportBindingKind::Namespace => module
			.type_references
			.iter()
			.any(|reference| reference.through_import == Some(binding.local_symbol)),
	}
}

fn detect_runtime_cycles(project: &HirProject, diagnostics: &mut Vec<ProjectDiagnostic>) {
	let mut state = vec![0_u8; project.modules.len()];
	for module in 0..project.modules.len() {
		if state[module] == 0 {
			dfs_runtime(ModuleId(module), project, &mut state, diagnostics);
		}
	}
}

fn dfs_runtime(
	module: ModuleId,
	project: &HirProject,
	state: &mut [u8],
	diagnostics: &mut Vec<ProjectDiagnostic>,
) {
	state[module.0] = 1;
	for edge in project
		.dependencies
		.iter()
		.filter(|edge| edge.from == module && edge.kind == DependencyKind::Runtime)
	{
		let Some(target) = edge.to else {
			continue;
		};
		match state[target.0] {
			0 => dfs_runtime(target, project, state, diagnostics),
			1 => diagnostics.push(project_error(
				&project.modules[module.0].source_id,
				"VHIR010",
				format!(
					"runtime import cycle reaches `{}`",
					project.modules[target.0].source_id.as_str()
				),
				edge.span,
			)),
			_ => {}
		}
	}
	state[module.0] = 2;
}

fn symbol_at_span(module: &ProjectModule, span: Span) -> Option<SymbolId> {
	module
		.hir
		.symbols
		.iter()
		.find(|symbol| symbol.span == span)
		.map(|symbol| symbol.id)
}

fn first_identifier(node: &SyntaxNode) -> Option<Token> {
	node.direct_tokens()
		.find(|token| token.kind == TokenKind::Identifier)
		.copied()
}

fn last_identifier(node: &SyntaxNode) -> Option<Token> {
	node.direct_tokens()
		.filter(|token| token.kind == TokenKind::Identifier)
		.copied()
		.last()
}

fn resolve_relative(importer: &ModuleSourceId, specifier: &str) -> ModuleSourceId {
	let parent = importer
		.as_str()
		.rsplit_once('/')
		.map_or("", |(parent, _)| parent);
	let joined = if parent.is_empty() {
		specifier.to_owned()
	} else {
		format!("{parent}/{specifier}")
	};
	ModuleSourceId(normalize_path(&joined))
}

fn normalize_path(path: &str) -> String {
	let mut parts = Vec::new();
	for part in path.split('/') {
		match part {
			"" | "." => {}
			".." => {
				parts.pop();
			}
			_ => parts.push(part),
		}
	}
	parts.join("/")
}

fn unquote(value: &str) -> String {
	if value.len() >= 2 {
		let bytes = value.as_bytes();
		if (bytes[0] == b'\'' && bytes[value.len() - 1] == b'\'')
			|| (bytes[0] == b'"' && bytes[value.len() - 1] == b'"')
		{
			return value[1..value.len() - 1].to_owned();
		}
	}
	value.to_owned()
}

fn project_error(
	source: &ModuleSourceId,
	code: &'static str,
	message: impl Into<String>,
	span: Span,
) -> ProjectDiagnostic {
	ProjectDiagnostic {
		source: source.clone(),
		diagnostic: Diagnostic::error(code, message, span),
	}
}
