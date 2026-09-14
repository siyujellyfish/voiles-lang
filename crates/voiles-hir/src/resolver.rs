use std::collections::HashMap;

use voiles_lexer::{Diagnostic, Keyword, Span, Token, TokenKind};
use voiles_syntax::{
	Parse, SyntaxKind, SyntaxNode,
	ast::{
		AstNode, BindingDecl, ComponentDecl, ForStmt, FunctionDecl, LifecycleBlock, NameExpr,
		ParamDecl, Parameter, token_text,
	},
};

use crate::{
	Capture, HirModule, HirResult, Reference, Scope, ScopeId, ScopeKind, Symbol, SymbolId,
	SymbolKind,
};

#[derive(Debug, Clone, Copy)]
struct PendingDecl {
	activation: usize,
	span: Span,
}

#[derive(Default)]
struct ScopeState {
	declared: HashMap<String, SymbolId>,
	pending: HashMap<String, Vec<PendingDecl>>,
}

enum Lookup {
	Resolved(SymbolId),
	UseBefore(Span),
	Missing,
}

pub(crate) fn resolve(source: &str, parsed: &Parse) -> HirResult {
	let mut resolver = Resolver::new(source);
	resolver.prepare_scope(&parsed.root, ScopeId(0));
	resolver.resolve_children_in_scope(&parsed.root, ScopeId(0));
	let mut diagnostics = parsed.diagnostics.clone();
	diagnostics.extend(resolver.diagnostics);
	HirResult {
		module: resolver.module,
		diagnostics,
	}
}

struct Resolver<'source> {
	source: &'source str,
	module: HirModule,
	states: Vec<ScopeState>,
	diagnostics: Vec<Diagnostic>,
}

impl<'source> Resolver<'source> {
	fn new(source: &'source str) -> Self {
		let root_scope = ScopeId(0);
		Self {
			source,
			module: HirModule {
				root_scope,
				scopes: vec![Scope {
					id: root_scope,
					parent: None,
					kind: ScopeKind::Module,
					owner_symbol: None,
					symbols: Vec::new(),
				}],
				symbols: Vec::new(),
				references: Vec::new(),
				captures: Vec::new(),
			},
			states: vec![ScopeState::default()],
			diagnostics: Vec::new(),
		}
	}

	fn resolve_children_in_scope(&mut self, node: &SyntaxNode, scope: ScopeId) {
		for child in node.child_nodes() {
			self.resolve_node(child, scope);
		}
	}

	fn resolve_node(&mut self, node: &SyntaxNode, scope: ScopeId) {
		match node.kind {
			SyntaxKind::ImportDecl | SyntaxKind::ExternImportDecl => {
				self.resolve_import(node, scope)
			}
			SyntaxKind::ExportDecl => {
				if let Some(declaration) = node.child_nodes().next() {
					self.resolve_node(declaration, scope);
				}
			}
			SyntaxKind::BindingDecl => self.resolve_binding(node, scope),
			SyntaxKind::ParamDecl => self.resolve_route_param(node, scope),
			SyntaxKind::FunctionDecl => self.resolve_function(node, scope),
			SyntaxKind::ComponentDecl => self.resolve_component(node, scope),
			SyntaxKind::StructDecl => self.resolve_named_type(node, scope, SymbolKind::Struct),
			SyntaxKind::EnumDecl => self.resolve_named_type(node, scope, SymbolKind::Enum),
			SyntaxKind::IfStmt => self.resolve_if(node, scope),
			SyntaxKind::ForStmt => self.resolve_for(node, scope),
			SyntaxKind::MatchStmt => self.resolve_match(node, scope),
			SyntaxKind::ReturnStmt | SyntaxKind::ExprStmt => {
				self.resolve_expression_children(node, scope)
			}
			SyntaxKind::LifecycleBlock => self.resolve_lifecycle(node, scope),
			SyntaxKind::SlotStmt => self.resolve_slot(node, scope),
			SyntaxKind::UiChildBlockStmt
			| SyntaxKind::StructuralBlock
			| SyntaxKind::ContainerBlock
			| SyntaxKind::ComponentChildBlock => self.resolve_ui_child(node, scope),
			SyntaxKind::Block => self.resolve_new_block(node, scope, ScopeKind::Block, None),
			SyntaxKind::Error => {}
			kind if voiles_syntax::ast::is_expression_kind(kind) => {
				self.resolve_expression(node, scope);
			}
			_ => {}
		}
	}

	fn resolve_import(&mut self, node: &SyntaxNode, scope: ScopeId) {
		for child in node.child_nodes() {
			if matches!(
				child.kind,
				SyntaxKind::ImportItem | SyntaxKind::NamespaceImport
			) {
				if let Some(token) = last_identifier(child) {
					let name = self.text(token).to_owned();
					self.declare(scope, name, SymbolKind::Import, token.span);
				}
			}
		}
	}

	fn resolve_binding(&mut self, node: &SyntaxNode, scope: ScopeId) {
		let Some(binding) = BindingDecl::cast(node) else {
			return;
		};
		if let Some(initializer) = binding.initializer() {
			self.resolve_expression(initializer, scope);
		}

		let Some(keyword) = binding.keyword() else {
			return;
		};
		let Some(name_token) = binding.name_token() else {
			return;
		};
		let kind = match keyword {
			Keyword::Const => SymbolKind::Const,
			Keyword::State => SymbolKind::State,
			Keyword::Shared => {
				if self.scope(scope).kind != ScopeKind::Module {
					self.error(
						"VHIR005",
						"`shared` declarations are only allowed at module top level",
						name_token.span,
					);
				}
				SymbolKind::Shared
			}
			_ => return,
		};
		self.declare(
			scope,
			self.text(name_token).to_owned(),
			kind,
			name_token.span,
		);
	}

	fn resolve_route_param(&mut self, node: &SyntaxNode, scope: ScopeId) {
		let Some(param) = ParamDecl::cast(node) else {
			return;
		};
		let Some(token) = param.name_token() else {
			return;
		};
		self.declare(
			scope,
			self.text(token).to_owned(),
			SymbolKind::RouteParam,
			token.span,
		);
	}

	fn resolve_function(&mut self, node: &SyntaxNode, scope: ScopeId) {
		let Some(function) = FunctionDecl::cast(node) else {
			return;
		};
		let Some(name_token) = function.name_token() else {
			return;
		};
		let symbol = self.declare(
			scope,
			self.text(name_token).to_owned(),
			SymbolKind::Function,
			name_token.span,
		);
		let function_scope = self.new_scope(scope, ScopeKind::Function, Some(symbol));
		let parameters: Vec<_> = function.parameters().collect();
		self.prepare_parameters(&parameters, function_scope);
		for parameter in parameters {
			self.resolve_parameter(parameter, function_scope);
		}
		if let Some(body) = function.body() {
			self.prepare_scope(body.syntax(), function_scope);
			self.resolve_children_in_scope(body.syntax(), function_scope);
		}
	}

	fn resolve_component(&mut self, node: &SyntaxNode, scope: ScopeId) {
		let Some(component) = ComponentDecl::cast(node) else {
			return;
		};
		let Some(name_token) = component.name_token() else {
			return;
		};
		let symbol = self.declare(
			scope,
			self.text(name_token).to_owned(),
			SymbolKind::Component,
			name_token.span,
		);
		let component_scope = self.new_scope(scope, ScopeKind::Component, Some(symbol));
		let parameters: Vec<_> = component.parameters().collect();
		self.prepare_parameters(&parameters, component_scope);
		for parameter in parameters {
			self.resolve_parameter(parameter, component_scope);
		}
		if let Some(body) = component.body() {
			self.prepare_scope(body.syntax(), component_scope);
			self.resolve_children_in_scope(body.syntax(), component_scope);
		}
	}

	fn resolve_parameter(&mut self, parameter: Parameter<'_>, scope: ScopeId) {
		if let Some(default) = parameter.default_expression() {
			self.resolve_expression(default, scope);
		}
		let Some(token) = parameter.name_token() else {
			return;
		};
		self.declare(
			scope,
			self.text(token).to_owned(),
			SymbolKind::Parameter,
			token.span,
		);
	}

	fn resolve_named_type(&mut self, node: &SyntaxNode, scope: ScopeId, kind: SymbolKind) {
		let Some(token) = first_identifier(node) else {
			return;
		};
		self.declare(scope, self.text(token).to_owned(), kind, token.span);
		if node.kind == SyntaxKind::StructDecl {
			for block in node
				.child_nodes()
				.filter(|child| child.kind == SyntaxKind::DeclarationBlock)
			{
				for field in block
					.child_nodes()
					.filter(|child| child.kind == SyntaxKind::StructField)
				{
					self.resolve_expression_children(field, scope);
				}
			}
		}
	}

	fn resolve_if(&mut self, node: &SyntaxNode, scope: ScopeId) {
		let mut children = node.child_nodes();
		if let Some(condition) = children.next() {
			self.resolve_expression(condition, scope);
		}
		for child in children {
			match child.kind {
				SyntaxKind::Block => self.resolve_new_block(child, scope, ScopeKind::Block, None),
				SyntaxKind::ElseClause => {
					if let Some(block) = child
						.child_nodes()
						.find(|nested| nested.kind == SyntaxKind::Block)
					{
						self.resolve_new_block(block, scope, ScopeKind::Block, None);
					}
				}
				_ => {}
			}
		}
	}

	fn resolve_for(&mut self, node: &SyntaxNode, scope: ScopeId) {
		let Some(for_stmt) = ForStmt::cast(node) else {
			return;
		};
		let mut child_nodes = node.child_nodes();
		if let Some(iterable) = child_nodes.next() {
			self.resolve_expression(iterable, scope);
		}

		let loop_scope = self.new_scope(scope, ScopeKind::Loop, None);
		if let Some(binding) = for_stmt.binding_token() {
			self.declare(
				loop_scope,
				self.text(binding).to_owned(),
				SymbolKind::LoopBinding,
				binding.span,
			);
		}

		for child in child_nodes {
			match child.kind {
				SyntaxKind::KeyClause => self.resolve_expression_children(child, loop_scope),
				SyntaxKind::Block => {
					self.prepare_scope(child, loop_scope);
					self.resolve_children_in_scope(child, loop_scope);
				}
				_ => {}
			}
		}
	}

	fn resolve_match(&mut self, node: &SyntaxNode, scope: ScopeId) {
		let mut children = node.child_nodes();
		if let Some(scrutinee) = children.next() {
			self.resolve_expression(scrutinee, scope);
		}
		for block in children.filter(|child| child.kind == SyntaxKind::DeclarationBlock) {
			for arm in block
				.child_nodes()
				.filter(|child| child.kind == SyntaxKind::MatchArm)
			{
				let arm_scope = self.new_scope(scope, ScopeKind::MatchArm, None);
				let mut arm_children = arm.child_nodes();
				if let Some(pattern) = arm_children.next() {
					self.bind_pattern_payloads(pattern, arm_scope, true);
				}
				if let Some(body) = arm_children.find(|child| child.kind == SyntaxKind::Block) {
					self.prepare_scope(body, arm_scope);
					self.resolve_children_in_scope(body, arm_scope);
				}
			}
		}
	}

	fn bind_pattern_payloads(&mut self, pattern: &SyntaxNode, scope: ScopeId, is_root: bool) {
		let nested: Vec<_> = pattern
			.child_nodes()
			.filter(|child| child.kind == SyntaxKind::Pattern)
			.collect();
		if !is_root && nested.is_empty() {
			let identifiers: Vec<_> = pattern
				.direct_tokens()
				.filter(|token| token.kind == TokenKind::Identifier)
				.copied()
				.collect();
			if identifiers.len() == 1 && self.text(identifiers[0]) != "_" {
				let token = identifiers[0];
				self.declare(
					scope,
					self.text(token).to_owned(),
					SymbolKind::PatternBinding,
					token.span,
				);
			}
		}
		for child in nested {
			self.bind_pattern_payloads(child, scope, false);
		}
	}

	fn resolve_lifecycle(&mut self, node: &SyntaxNode, scope: ScopeId) {
		let Some(lifecycle) = LifecycleBlock::cast(node) else {
			return;
		};
		let Some(keyword) = lifecycle.keyword() else {
			return;
		};
		let (kind, valid) = match keyword {
			Keyword::Init => (
				ScopeKind::LifecycleInit,
				self.scope(scope).kind == ScopeKind::Module,
			),
			Keyword::Mount => (
				ScopeKind::LifecycleMount,
				self.scope(scope).kind == ScopeKind::Component,
			),
			Keyword::Cleanup => (
				ScopeKind::LifecycleCleanup,
				matches!(
					self.scope(scope).kind,
					ScopeKind::LifecycleInit | ScopeKind::LifecycleMount
				),
			),
			_ => return,
		};
		if !valid {
			self.error(
				"VHIR006",
				"lifecycle block is not valid in this lexical owner",
				node.span,
			);
		}
		if let Some(body) = lifecycle.body() {
			let lifecycle_scope = self.new_scope(scope, kind, None);
			self.prepare_scope(body.syntax(), lifecycle_scope);
			self.resolve_children_in_scope(body.syntax(), lifecycle_scope);
		}
	}

	fn resolve_slot(&mut self, node: &SyntaxNode, scope: ScopeId) {
		if let Some(block) = node
			.child_nodes()
			.find(|child| child.kind == SyntaxKind::Block)
		{
			self.resolve_new_block(block, scope, ScopeKind::Block, None);
		}
	}

	fn resolve_ui_child(&mut self, node: &SyntaxNode, scope: ScopeId) {
		let mut children = node.child_nodes();
		if let Some(head) = children.next() {
			self.resolve_ui_head(head, scope);
		}
		for child in children {
			if child.kind == SyntaxKind::Block {
				self.resolve_new_block(child, scope, ScopeKind::Block, None);
			}
		}
	}

	fn resolve_ui_head(&mut self, node: &SyntaxNode, scope: ScopeId) {
		match node.kind {
			SyntaxKind::NameExpr => {}
			SyntaxKind::CallExpr => {
				let mut children = node.child_nodes();
				if let Some(callee) = children.next() {
					if callee.kind != SyntaxKind::NameExpr {
						self.resolve_expression(callee, scope);
					}
				}
				for child in children {
					self.resolve_expression(child, scope);
				}
			}
			_ => self.resolve_expression(node, scope),
		}
	}

	fn resolve_expression_children(&mut self, node: &SyntaxNode, scope: ScopeId) {
		for child in node.child_nodes() {
			if voiles_syntax::ast::is_expression_kind(child.kind)
				|| matches!(
					child.kind,
					SyntaxKind::ArgumentList
						| SyntaxKind::NamedArgument
						| SyntaxKind::PositionalArgument
						| SyntaxKind::KeyClause
				) {
				self.resolve_expression(child, scope);
			}
		}
	}

	fn resolve_expression(&mut self, node: &SyntaxNode, scope: ScopeId) -> Option<SymbolId> {
		if node.kind == SyntaxKind::NameExpr {
			let name = NameExpr::cast(node)?;
			let token = name.token()?;
			return self.resolve_reference(token, scope);
		}

		if node.kind == SyntaxKind::AssignmentExpr {
			let mut children = node.child_nodes();
			if let Some(left) = children.next() {
				let resolved = self.resolve_expression(left, scope);
				if left.kind == SyntaxKind::NameExpr {
					if let Some(symbol) = resolved {
						if !self.module.symbols[symbol.0].is_mutable() {
							self.error("VHIR004", "cannot assign to immutable binding", left.span);
						}
					}
				}
			}
			for child in children {
				self.resolve_expression(child, scope);
			}
			return None;
		}

		for child in node.child_nodes() {
			self.resolve_expression(child, scope);
		}
		None
	}

	fn resolve_reference(&mut self, token: Token, scope: ScopeId) -> Option<SymbolId> {
		let name = self.text(token).to_owned();
		let lookup = self.lookup(scope, &name, token.span.start);
		let resolved = match lookup {
			Lookup::Resolved(symbol) => {
				self.record_capture(scope, symbol, token.span);
				Some(symbol)
			}
			Lookup::UseBefore(declaration_span) => {
				self.error(
					"VHIR002",
					format!("`{name}` is used before its declaration"),
					token.span,
				);
				let _ = declaration_span;
				None
			}
			Lookup::Missing => {
				self.error("VHIR003", format!("unresolved name `{name}`"), token.span);
				None
			}
		};
		self.module.references.push(Reference {
			name,
			span: token.span,
			scope,
			resolved,
		});
		resolved
	}

	fn lookup(&self, mut scope: ScopeId, name: &str, reference_start: usize) -> Lookup {
		loop {
			if let Some(symbol) = self.states[scope.0].declared.get(name) {
				return Lookup::Resolved(*symbol);
			}
			if let Some(pending) = self.states[scope.0].pending.get(name) {
				if let Some(declaration) = pending
					.iter()
					.filter(|decl| reference_start < decl.activation)
					.min_by_key(|decl| decl.activation)
				{
					return Lookup::UseBefore(declaration.span);
				}
			}
			let Some(parent) = self.scope(scope).parent else {
				return Lookup::Missing;
			};
			scope = parent;
		}
	}

	fn record_capture(&mut self, scope: ScopeId, symbol: SymbolId, reference: Span) {
		let Some(function_scope) = self.nearest_function_scope(scope) else {
			return;
		};
		let captured = &self.module.symbols[symbol.0];
		if self.scope(captured.scope).kind == ScopeKind::Module
			|| self.is_descendant_or_same(captured.scope, function_scope)
		{
			return;
		}
		let Some(function) = self.scope(function_scope).owner_symbol else {
			return;
		};
		if self
			.module
			.captures
			.iter()
			.any(|capture| capture.function == function && capture.symbol == symbol)
		{
			return;
		}
		self.module.captures.push(Capture {
			function,
			symbol,
			first_reference: reference,
		});
	}

	fn nearest_function_scope(&self, mut scope: ScopeId) -> Option<ScopeId> {
		loop {
			if self.scope(scope).kind == ScopeKind::Function {
				return Some(scope);
			}
			scope = self.scope(scope).parent?;
		}
	}

	fn is_descendant_or_same(&self, mut scope: ScopeId, ancestor: ScopeId) -> bool {
		loop {
			if scope == ancestor {
				return true;
			}
			let Some(parent) = self.scope(scope).parent else {
				return false;
			};
			scope = parent;
		}
	}

	fn resolve_new_block(
		&mut self,
		block: &SyntaxNode,
		parent: ScopeId,
		kind: ScopeKind,
		owner_symbol: Option<SymbolId>,
	) {
		let scope = self.new_scope(parent, kind, owner_symbol);
		self.prepare_scope(block, scope);
		self.resolve_children_in_scope(block, scope);
	}

	fn new_scope(
		&mut self,
		parent: ScopeId,
		kind: ScopeKind,
		owner_symbol: Option<SymbolId>,
	) -> ScopeId {
		let id = ScopeId(self.module.scopes.len());
		self.module.scopes.push(Scope {
			id,
			parent: Some(parent),
			kind,
			owner_symbol,
			symbols: Vec::new(),
		});
		self.states.push(ScopeState::default());
		id
	}

	fn prepare_scope(&mut self, node: &SyntaxNode, scope: ScopeId) {
		for child in node.child_nodes() {
			self.prepare_pending_node(child, scope);
		}
	}

	fn prepare_pending_node(&mut self, node: &SyntaxNode, scope: ScopeId) {
		if node.kind == SyntaxKind::ExportDecl {
			if let Some(declaration) = node.child_nodes().next() {
				self.prepare_pending_node(declaration, scope);
			}
			return;
		}
		match node.kind {
			SyntaxKind::ImportDecl | SyntaxKind::ExternImportDecl => {
				for child in node.child_nodes() {
					if matches!(
						child.kind,
						SyntaxKind::ImportItem | SyntaxKind::NamespaceImport
					) {
						if let Some(token) = last_identifier(child) {
							self.add_pending(scope, token, node.span.end);
						}
					}
				}
			}
			SyntaxKind::BindingDecl => {
				if let Some(token) = BindingDecl::cast(node).and_then(BindingDecl::name_token) {
					self.add_pending(scope, token, node.span.end);
				}
			}
			SyntaxKind::ParamDecl => {
				if let Some(token) = ParamDecl::cast(node).and_then(ParamDecl::name_token) {
					self.add_pending(scope, token, node.span.end);
				}
			}
			SyntaxKind::FunctionDecl
			| SyntaxKind::ComponentDecl
			| SyntaxKind::StructDecl
			| SyntaxKind::EnumDecl => {
				if let Some(token) = first_identifier(node) {
					self.add_pending(scope, token, token.span.end);
				}
			}
			_ => {}
		}
	}

	fn prepare_parameters(&mut self, parameters: &[Parameter<'_>], scope: ScopeId) {
		for parameter in parameters {
			if let Some(token) = parameter.name_token() {
				self.add_pending(scope, token, parameter.syntax().span.end);
			}
		}
	}

	fn add_pending(&mut self, scope: ScopeId, token: Token, activation: usize) {
		let name = self.text(token).to_owned();
		self.states[scope.0]
			.pending
			.entry(name)
			.or_default()
			.push(PendingDecl {
				activation,
				span: token.span,
			});
	}

	fn declare(&mut self, scope: ScopeId, name: String, kind: SymbolKind, span: Span) -> SymbolId {
		if let Some(existing) = self.states[scope.0].declared.get(&name).copied() {
			self.error(
				"VHIR001",
				format!("duplicate declaration of `{name}` in the same scope"),
				span,
			);
			return existing;
		}
		let id = SymbolId(self.module.symbols.len());
		self.module.symbols.push(Symbol {
			id,
			name: name.clone(),
			kind,
			span,
			scope,
		});
		self.module.scopes[scope.0].symbols.push(id);
		self.states[scope.0].declared.insert(name, id);
		id
	}

	fn scope(&self, id: ScopeId) -> &Scope {
		&self.module.scopes[id.0]
	}

	fn text(&self, token: Token) -> &'source str {
		token_text(self.source, token)
	}

	fn error(&mut self, code: &'static str, message: impl Into<String>, span: Span) {
		self.diagnostics
			.push(Diagnostic::error(code, message, span));
	}
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
