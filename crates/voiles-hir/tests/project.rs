use voiles_hir::{
	DependencyKind, ImportBindingKind, ModuleSource, SymbolKind, TypeTarget, resolve_project,
};

fn has_diagnostic(result: &voiles_hir::HirProjectResult, code: &str) -> bool {
	result
		.diagnostics
		.iter()
		.any(|diagnostic| diagnostic.diagnostic.code == code)
}

#[test]
fn resolves_component_and_explicit_exports_across_modules() {
	let result = resolve_project(vec![
		ModuleSource::new(
			"ui/button.voil",
			"export fn helper(value: Int) -> Int:\n\treturn value\n\nexport struct User:\n\tid: Int\n\ncomponent Button(text: String):\n\tstate clicks = 0\n",
		),
		ModuleSource::new(
			"app/main.voil",
			"import Button, helper as local_helper, User from \"../ui/button.voil\"\nconst value: Int = local_helper(1)\n",
		),
	]);

	assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
	let library = &result.project.modules[0];
	assert_eq!(library.export("Button").expect("component export").kind, SymbolKind::Component);
	assert_eq!(library.export("helper").expect("function export").kind, SymbolKind::Function);
	assert_eq!(library.export("User").expect("type export").kind, SymbolKind::Struct);

	let main = &result.project.modules[1];
	assert_eq!(main.imports.len(), 3);
	assert!(main.imports.iter().all(|binding| binding.target_module.is_some()));
	let helper = main
		.imports
		.iter()
		.find(|binding| binding.imported_name.as_deref() == Some("helper"))
		.expect("helper import");
	assert_eq!(helper.kind, ImportBindingKind::Named);
	assert!(helper.target_symbol.is_some());
	assert_eq!(result.project.dependencies[0].kind, DependencyKind::Runtime);
}

#[test]
fn rejects_runtime_module_cycles() {
	let result = resolve_project(vec![
		ModuleSource::new(
			"a.voil",
			"import run_b from \"./b.voil\"\nexport fn run_a():\n\trun_b()\n",
		),
		ModuleSource::new(
			"b.voil",
			"import run_a from \"./a.voil\"\nexport fn run_b():\n\trun_a()\n",
		),
	]);

	assert!(has_diagnostic(&result, "VHIR010"));
	assert!(result
		.project
		.dependencies
		.iter()
		.all(|edge| edge.kind == DependencyKind::Runtime));
}

#[test]
fn allows_cycles_when_edges_are_type_only() {
	let result = resolve_project(vec![
		ModuleSource::new(
			"a.voil",
			"import B from \"./b.voil\"\nexport struct A:\n\tchild: B?\n",
		),
		ModuleSource::new(
			"b.voil",
			"import A from \"./a.voil\"\nexport struct B:\n\tparent: A?\n",
		),
	]);

	assert!(!has_diagnostic(&result, "VHIR010"), "{:?}", result.diagnostics);
	assert!(result
		.project
		.dependencies
		.iter()
		.all(|edge| edge.kind == DependencyKind::TypeOnly));
	assert!(result.project.modules.iter().all(|module| {
		module
			.type_references
			.iter()
			.any(|reference| matches!(reference.target, Some(TypeTarget::Symbol(_))))
	}));
}

#[test]
fn resolves_namespace_qualified_type_references_without_runtime_edge() {
	let result = resolve_project(vec![
		ModuleSource::new("models.voil", "export struct User:\n\tid: Int\n"),
		ModuleSource::new(
			"page.voil",
			"import * as models from \"./models.voil\"\nstate current: models.User? = none\n",
		),
	]);

	assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
	assert_eq!(result.project.dependencies[0].kind, DependencyKind::TypeOnly);
	let page = &result.project.modules[1];
	assert!(page.type_references.iter().any(|reference| {
		reference.path == ["models", "User"] && matches!(reference.target, Some(TypeTarget::Symbol(_)))
	}));
}

#[test]
fn reports_unknown_named_exports() {
	let result = resolve_project(vec![
		ModuleSource::new("lib.voil", "export const version = \"1\"\n"),
		ModuleSource::new("main.voil", "import missing from \"./lib.voil\"\n"),
	]);

	assert!(has_diagnostic(&result, "VHIR009"));
}
