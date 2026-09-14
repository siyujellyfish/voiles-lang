use voiles_hir::{BuiltinType, GlobalSymbolId, HirProject, ModuleId, SymbolId, TypeTarget};
use voiles_lexer::{Keyword, Span, Token, TokenKind};
use voiles_syntax::{Parse, SyntaxKind, SyntaxNode, ast::token_text};

use crate::{
	FunctionParameter, FunctionSignature, Type, TypeCheckResult, TypeDiagnostic, TypedModule,
};

#[must_use]
pub fn check_project(project: &HirProject) -> TypeCheckResult {
	let parses: Vec<Parse> = project
		.modules
		.iter()
		.map(|module| voiles_syntax::parse(&module.source))
		.collect();
	let modules = project
		.modules
		.iter()
		.map(|module| TypedModule {
			module: module.id,
			symbol_types: vec![Type::Unknown; module.hir.symbols.len()],
		})
		.collect();
	let mut checker = Checker {
		project,
		parses,
		result: TypeCheckResult {
			modules,
			diagnostics: Vec::new(),
		},
	};

	for module_index in 0..checker.project.modules.len() {
		checker.collect_declaration_types(ModuleId(module_index));
	}
	checker.propagate_import_types();
	for module_index in 0..checker.project.modules.len() {
		checker.check_module(ModuleId(module_index));
	}
	checker.propagate_import_types();

	checker.result
}

struct Checker<'project> {
	project: &'project HirProject,
	parses: Vec<Parse>,
	result: TypeCheckResult,
}

impl Checker<'_> {
	fn collect_declaration_types(&mut self, module: ModuleId) {
		let root = self.parses[module.0].root.clone();
		self.collect_node_types(module, &root);
	}

	fn collect_node_types(&mut self, module: ModuleId, node: &SyntaxNode) {
		match node.kind {
			SyntaxKind::FunctionDecl => self.collect_function_signature(module, node),
			SyntaxKind::ComponentDecl => self.collect_component_parameters(module, node),
			SyntaxKind::BindingDecl => self.seed_binding_type(module, node),
			_ => {}
		}
		for child in node.child_nodes() {
			self.collect_node_types(module, child);
		}
	}

	fn collect_function_signature(&mut self, module: ModuleId, node: &SyntaxNode) {
		let Some(name) = first_identifier(node) else {
			return;
		};
		let Some(symbol) = self.symbol_at_span(module, name.span) else {
			return;
		};
		let mut parameters = Vec::new();
		if let Some(list) = node
			.child_nodes()
			.find(|child| child.kind == SyntaxKind::ParameterList)
		{
			for parameter in list
				.child_nodes()
				.filter(|child| child.kind == SyntaxKind::Parameter)
			{
				let Some(name_token) = first_identifier(parameter) else {
					continue;
				};
				let parameter_symbol = self.symbol_at_span(module, name_token.span);
				let ty = self
					.first_type_child(parameter)
					.map_or(Type::Unknown, |node| self.resolve_type(module, node));
				if let Some(parameter_symbol) = parameter_symbol {
					self.set_symbol_type(module, parameter_symbol, ty.clone());
				}
				parameters.push(FunctionParameter {
					name: token_text(&self.project.modules[module.0].source, name_token).to_owned(),
					symbol: parameter_symbol,
					ty,
					has_default: parameter
						.child_nodes()
						.any(|child| is_expression_kind(child.kind)),
				});
			}
		}
		let return_type = node
			.child_nodes()
			.find(|child| child.kind == SyntaxKind::ReturnType)
			.and_then(|return_type| self.first_type_child(return_type))
			.map_or(Type::Unknown, |node| self.resolve_type(module, node));
		let is_async = node
			.direct_tokens()
			.any(|token| token.kind == TokenKind::Keyword(Keyword::Async));
		self.set_symbol_type(
			module,
			symbol,
			Type::Function(FunctionSignature {
				parameters,
				return_type: Box::new(return_type),
				is_async,
			}),
		);
	}

	fn collect_component_parameters(&mut self, module: ModuleId, node: &SyntaxNode) {
		let Some(list) = node
			.child_nodes()
			.find(|child| child.kind == SyntaxKind::ParameterList)
		else {
			return;
		};
		for parameter in list
			.child_nodes()
			.filter(|child| child.kind == SyntaxKind::Parameter)
		{
			let Some(name_token) = first_identifier(parameter) else {
				continue;
			};
			let Some(symbol) = self.symbol_at_span(module, name_token.span) else {
				continue;
			};
			let ty = self
				.first_type_child(parameter)
				.map_or(Type::Unknown, |node| self.resolve_type(module, node));
			self.set_symbol_type(module, symbol, ty);
		}
	}

	fn seed_binding_type(&mut self, module: ModuleId, node: &SyntaxNode) {
		let Some(name) = first_identifier(node) else {
			return;
		};
		let Some(symbol) = self.symbol_at_span(module, name.span) else {
			return;
		};
		if let Some(annotation) = self.first_type_child(node) {
			let ty = self.resolve_type(module, annotation);
			self.set_symbol_type(module, symbol, ty);
			return;
		}
		if let Some(initializer) = first_expression_child(node)
			&& initializer.kind == SyntaxKind::LiteralExpr
		{
			let ty = self.infer_literal(module, initializer);
			self.set_symbol_type(module, symbol, ty);
		}
	}

	fn propagate_import_types(&mut self) {
		for module_index in 0..self.project.modules.len() {
			let module = ModuleId(module_index);
			for import in &self.project.modules[module_index].imports {
				let Some(target) = import.target_symbol else {
					continue;
				};
				let ty = self.symbol_type_global(target).clone();
				if !matches!(ty, Type::Unknown) {
					self.set_symbol_type(module, import.local_symbol, ty);
				}
			}
		}
	}

	fn check_module(&mut self, module: ModuleId) {
		let root = self.parses[module.0].root.clone();
		self.check_node(module, &root, None);
	}

	fn check_node(&mut self, module: ModuleId, node: &SyntaxNode, expected_return: Option<&Type>) {
		match node.kind {
			SyntaxKind::Module | SyntaxKind::Block | SyntaxKind::DeclarationBlock => {
				for child in node.child_nodes() {
					self.check_node(module, child, expected_return);
				}
			}
			SyntaxKind::ExportDecl => {
				if let Some(declaration) = node.child_nodes().next() {
					self.check_node(module, declaration, expected_return);
				}
			}
			SyntaxKind::BindingDecl => self.check_binding(module, node),
			SyntaxKind::FunctionDecl => self.check_function(module, node),
			SyntaxKind::ComponentDecl => self.check_component(module, node),
			SyntaxKind::ReturnStmt => self.check_return(module, node, expected_return),
			SyntaxKind::ExprStmt => {
				if let Some(expression) = first_expression_child(node) {
					self.infer_expression(module, expression);
				}
			}
			SyntaxKind::IfStmt => self.check_if(module, node, expected_return),
			SyntaxKind::ForStmt => self.check_for(module, node, expected_return),
			SyntaxKind::UiChildBlockStmt
			| SyntaxKind::StructuralBlock
			| SyntaxKind::ContainerBlock
			| SyntaxKind::ComponentChildBlock => self.check_ui_node(module, node, expected_return),
			_ => {
				for child in node.child_nodes() {
					self.check_node(module, child, expected_return);
				}
			}
		}
	}

	fn check_binding(&mut self, module: ModuleId, node: &SyntaxNode) {
		let Some(name) = first_identifier(node) else {
			return;
		};
		let Some(symbol) = self.symbol_at_span(module, name.span) else {
			return;
		};
		let annotation = self
			.first_type_child(node)
			.map(|node| self.resolve_type(module, node));
		let initializer =
			first_expression_child(node).map(|node| self.infer_expression(module, node));
		match (annotation, initializer) {
			(Some(expected), Some(actual)) => {
				self.expect_assignable(
					module,
					&expected,
					&actual,
					node.span,
					"binding initializer",
				);
				self.set_symbol_type(module, symbol, expected);
			}
			(Some(expected), None) => self.set_symbol_type(module, symbol, expected),
			(None, Some(actual)) => self.set_symbol_type(module, symbol, actual),
			(None, None) => {}
		}
	}

	fn check_function(&mut self, module: ModuleId, node: &SyntaxNode) {
		let Some(name) = first_identifier(node) else {
			return;
		};
		let Some(symbol) = self.symbol_at_span(module, name.span) else {
			return;
		};
		let Type::Function(signature) = self.symbol_type(module, symbol).clone() else {
			return;
		};

		if let Some(list) = node
			.child_nodes()
			.find(|child| child.kind == SyntaxKind::ParameterList)
		{
			for (parameter, signature_parameter) in list
				.child_nodes()
				.filter(|child| child.kind == SyntaxKind::Parameter)
				.zip(&signature.parameters)
			{
				if let Some(default) = first_expression_child(parameter) {
					let actual = self.infer_expression(module, default);
					self.expect_assignable(
						module,
						&signature_parameter.ty,
						&actual,
						default.span,
						"parameter default",
					);
				}
			}
		}

		if let Some(body) = node
			.child_nodes()
			.find(|child| child.kind == SyntaxKind::Block)
		{
			self.check_node(module, body, Some(&signature.return_type));
		}
	}

	fn check_component(&mut self, module: ModuleId, node: &SyntaxNode) {
		if let Some(list) = node
			.child_nodes()
			.find(|child| child.kind == SyntaxKind::ParameterList)
		{
			for parameter in list
				.child_nodes()
				.filter(|child| child.kind == SyntaxKind::Parameter)
			{
				let Some(name) = first_identifier(parameter) else {
					continue;
				};
				let Some(symbol) = self.symbol_at_span(module, name.span) else {
					continue;
				};
				if let Some(default) = first_expression_child(parameter) {
					let actual = self.infer_expression(module, default);
					let expected = self.symbol_type(module, symbol).clone();
					self.expect_assignable(
						module,
						&expected,
						&actual,
						default.span,
						"component parameter default",
					);
				}
			}
		}
		if let Some(body) = node
			.child_nodes()
			.find(|child| child.kind == SyntaxKind::Block)
		{
			self.check_node(module, body, None);
		}
	}

	fn check_return(&mut self, module: ModuleId, node: &SyntaxNode, expected: Option<&Type>) {
		let Some(expected) = expected else {
			return;
		};
		if expected.is_unknown_or_error() {
			return;
		}
		let actual = first_expression_child(node)
			.map(|expression| self.infer_expression(module, expression))
			.unwrap_or_else(|| Type::builtin(BuiltinType::Void));
		if !is_assignable(expected, &actual) {
			self.error(
				module,
				"VTYPE007",
				format!("return type mismatch: expected {expected:?}, found {actual:?}"),
				node.span,
			);
		}
	}

	fn check_if(&mut self, module: ModuleId, node: &SyntaxNode, expected_return: Option<&Type>) {
		let mut children = node.child_nodes();
		if let Some(condition) = children.next() {
			let actual = self.infer_expression(module, condition);
			self.expect_assignable(
				module,
				&Type::builtin(BuiltinType::Bool),
				&actual,
				condition.span,
				"if condition",
			);
		}
		for child in children {
			self.check_node(module, child, expected_return);
		}
	}

	fn check_for(&mut self, module: ModuleId, node: &SyntaxNode, expected_return: Option<&Type>) {
		let mut children = node.child_nodes();
		let iterable_type = children
			.next()
			.map(|iterable| self.infer_expression(module, iterable))
			.unwrap_or(Type::Unknown);
		if let Some(binding_token) = first_identifier(node)
			&& let Some(binding) = self.symbol_at_span(module, binding_token.span)
			&& let Type::Builtin(BuiltinType::List, arguments) = &iterable_type
			&& let Some(item) = arguments.first()
		{
			self.set_symbol_type(module, binding, item.clone());
		}
		for child in children {
			if child.kind == SyntaxKind::KeyClause {
				if let Some(expression) = first_expression_child(child) {
					let key = self.infer_expression(module, expression);
					if !key.is_unknown_or_error()
						&& key != Type::builtin(BuiltinType::String)
						&& key != Type::builtin(BuiltinType::Int)
					{
						self.error(
							module,
							"VTYPE008",
							"repeated UI key must be String or Int",
							expression.span,
						);
					}
				}
			} else {
				self.check_node(module, child, expected_return);
			}
		}
	}

	fn check_ui_node(
		&mut self,
		module: ModuleId,
		node: &SyntaxNode,
		expected_return: Option<&Type>,
	) {
		let mut children = node.child_nodes();
		if let Some(head) = children.next() {
			self.check_ui_head(module, head);
		}
		for child in children {
			self.check_node(module, child, expected_return);
		}
	}

	fn check_ui_head(&mut self, module: ModuleId, node: &SyntaxNode) {
		if node.kind == SyntaxKind::CallExpr {
			if let Some(arguments) = node
				.child_nodes()
				.find(|child| child.kind == SyntaxKind::ArgumentList)
			{
				for argument in arguments.child_nodes() {
					if let Some(expression) = first_expression_child(argument) {
						self.infer_expression(module, expression);
					}
				}
			}
		} else if is_expression_kind(node.kind) && node.kind != SyntaxKind::NameExpr {
			self.infer_expression(module, node);
		}
	}

	fn infer_expression(&mut self, module: ModuleId, node: &SyntaxNode) -> Type {
		match node.kind {
			SyntaxKind::LiteralExpr => self.infer_literal(module, node),
			SyntaxKind::NameExpr => self.infer_name(module, node),
			SyntaxKind::ParenthesizedExpr => first_expression_child(node)
				.map(|child| self.infer_expression(module, child))
				.unwrap_or(Type::Unknown),
			SyntaxKind::UnaryExpr => self.infer_unary(module, node),
			SyntaxKind::BinaryExpr => self.infer_binary(module, node),
			SyntaxKind::AssignmentExpr => self.infer_assignment(module, node),
			SyntaxKind::CallExpr => self.infer_call(module, node),
			SyntaxKind::IndexExpr => self.infer_index(module, node),
			SyntaxKind::TryExpr => self.infer_try(module, node),
			SyntaxKind::MemberExpr => Type::Unknown,
			_ => Type::Unknown,
		}
	}

	fn infer_literal(&self, _module: ModuleId, node: &SyntaxNode) -> Type {
		let Some(token) = node.direct_tokens().find(|token| {
			matches!(
				token.kind,
				TokenKind::Integer
					| TokenKind::Float
					| TokenKind::String
					| TokenKind::Keyword(Keyword::True | Keyword::False | Keyword::None)
			)
		}) else {
			return Type::Unknown;
		};
		match token.kind {
			TokenKind::Integer => Type::builtin(BuiltinType::Int),
			TokenKind::Float => Type::builtin(BuiltinType::Float),
			TokenKind::String => Type::builtin(BuiltinType::String),
			TokenKind::Keyword(Keyword::True | Keyword::False) => Type::builtin(BuiltinType::Bool),
			TokenKind::Keyword(Keyword::None) => Type::None,
			_ => Type::Unknown,
		}
	}

	fn infer_name(&self, module: ModuleId, node: &SyntaxNode) -> Type {
		let Some(token) = first_identifier(node) else {
			return Type::Unknown;
		};
		let resolved = self.project.modules[module.0]
			.hir
			.references
			.iter()
			.find(|reference| reference.span == token.span)
			.and_then(|reference| reference.resolved);
		resolved.map_or(Type::Unknown, |symbol| {
			self.symbol_type(module, symbol).clone()
		})
	}

	fn infer_unary(&mut self, module: ModuleId, node: &SyntaxNode) -> Type {
		let Some(operand_node) = first_expression_child(node) else {
			return Type::Unknown;
		};
		let operand = self.infer_expression(module, operand_node);
		let operator = significant_operator(node);
		match operator {
			Some(TokenKind::Bang | TokenKind::Keyword(Keyword::Not)) => {
				self.expect_assignable(
					module,
					&Type::builtin(BuiltinType::Bool),
					&operand,
					node.span,
					"logical operand",
				);
				Type::builtin(BuiltinType::Bool)
			}
			Some(TokenKind::Plus | TokenKind::Minus) => {
				if operand.is_unknown_or_error()
					|| matches!(
						operand,
						Type::Builtin(BuiltinType::Int | BuiltinType::Float, ref args) if args.is_empty()
					) {
					operand
				} else {
					self.error(
						module,
						"VTYPE001",
						"numeric unary operand required",
						node.span,
					);
					Type::Error
				}
			}
			Some(TokenKind::Keyword(Keyword::Await)) => operand,
			_ => Type::Unknown,
		}
	}

	fn infer_binary(&mut self, module: ModuleId, node: &SyntaxNode) -> Type {
		let mut expressions = node
			.child_nodes()
			.filter(|child| is_expression_kind(child.kind));
		let Some(left_node) = expressions.next() else {
			return Type::Unknown;
		};
		let Some(right_node) = expressions.next() else {
			return Type::Unknown;
		};
		let left = self.infer_expression(module, left_node);
		let right = self.infer_expression(module, right_node);
		let operator = significant_operator(node);
		match operator {
			Some(
				TokenKind::Keyword(Keyword::And | Keyword::Or)
				| TokenKind::AndAnd
				| TokenKind::OrOr,
			) => {
				let boolean = Type::builtin(BuiltinType::Bool);
				self.expect_assignable(module, &boolean, &left, left_node.span, "logical operand");
				self.expect_assignable(
					module,
					&boolean,
					&right,
					right_node.span,
					"logical operand",
				);
				boolean
			}
			Some(TokenKind::EqualEqual | TokenKind::BangEqual) => {
				if !is_assignable(&left, &right) && !is_assignable(&right, &left) {
					self.error(
						module,
						"VTYPE001",
						"incompatible equality operands",
						node.span,
					);
				}
				Type::builtin(BuiltinType::Bool)
			}
			Some(
				TokenKind::Less
				| TokenKind::LessEqual
				| TokenKind::Greater
				| TokenKind::GreaterEqual,
			) => {
				if !same_numeric_type(&left, &right) {
					self.error(
						module,
						"VTYPE001",
						"comparison operands must share a numeric type",
						node.span,
					);
				}
				Type::builtin(BuiltinType::Bool)
			}
			Some(
				TokenKind::Plus
				| TokenKind::Minus
				| TokenKind::Star
				| TokenKind::Slash
				| TokenKind::Percent,
			) => {
				if same_numeric_type(&left, &right) {
					left
				} else if operator == Some(TokenKind::Plus)
					&& left == Type::builtin(BuiltinType::String)
					&& right == Type::builtin(BuiltinType::String)
				{
					left
				} else if left.is_unknown_or_error() || right.is_unknown_or_error() {
					Type::Unknown
				} else {
					self.error(
						module,
						"VTYPE001",
						"invalid arithmetic operand types",
						node.span,
					);
					Type::Error
				}
			}
			_ => Type::Unknown,
		}
	}

	fn infer_assignment(&mut self, module: ModuleId, node: &SyntaxNode) -> Type {
		let mut expressions = node
			.child_nodes()
			.filter(|child| is_expression_kind(child.kind));
		let Some(left_node) = expressions.next() else {
			return Type::Unknown;
		};
		let Some(right_node) = expressions.next() else {
			return Type::Unknown;
		};
		let left = self.infer_expression(module, left_node);
		let right = self.infer_expression(module, right_node);
		self.expect_assignable(module, &left, &right, right_node.span, "assignment");
		right
	}

	fn infer_call(&mut self, module: ModuleId, node: &SyntaxNode) -> Type {
		let mut children = node.child_nodes();
		let Some(callee_node) = children.next() else {
			return Type::Unknown;
		};
		let callee = self.infer_expression(module, callee_node);
		let Some(arguments) = children.find(|child| child.kind == SyntaxKind::ArgumentList) else {
			return Type::Unknown;
		};
		let Type::Function(signature) = callee else {
			for argument in arguments.child_nodes() {
				if let Some(expression) = first_expression_child(argument) {
					self.infer_expression(module, expression);
				}
			}
			if !callee_node_is_unknown(self, module, callee_node) {
				self.error(
					module,
					"VTYPE005",
					"value is not callable",
					callee_node.span,
				);
			}
			return Type::Unknown;
		};
		self.check_call_arguments(module, arguments, &signature);
		(*signature.return_type).clone()
	}

	fn check_call_arguments(
		&mut self,
		module: ModuleId,
		arguments: &SyntaxNode,
		signature: &FunctionSignature,
	) {
		let mut assigned = vec![false; signature.parameters.len()];
		let mut next_positional = 0;
		for argument in arguments.child_nodes() {
			match argument.kind {
				SyntaxKind::PositionalArgument => {
					let Some(expression) = first_expression_child(argument) else {
						continue;
					};
					let actual = self.infer_expression(module, expression);
					if next_positional >= signature.parameters.len() {
						self.error(
							module,
							"VTYPE002",
							"too many positional arguments",
							argument.span,
						);
						continue;
					}
					let parameter = &signature.parameters[next_positional];
					self.expect_assignable(
						module,
						&parameter.ty,
						&actual,
						expression.span,
						"function argument",
					);
					assigned[next_positional] = true;
					next_positional += 1;
				}
				SyntaxKind::NamedArgument => {
					let Some(name_token) = first_identifier(argument) else {
						continue;
					};
					let name = token_text(&self.project.modules[module.0].source, name_token);
					let Some(index) = signature
						.parameters
						.iter()
						.position(|parameter| parameter.name == name)
					else {
						self.error(
							module,
							"VTYPE003",
							format!("unknown named argument `{name}`"),
							name_token.span,
						);
						if let Some(expression) = first_expression_child(argument) {
							self.infer_expression(module, expression);
						}
						continue;
					};
					if assigned[index] {
						self.error(
							module,
							"VTYPE004",
							format!("duplicate argument `{name}`"),
							name_token.span,
						);
						continue;
					}
					if let Some(expression) = first_expression_child(argument) {
						let actual = self.infer_expression(module, expression);
						self.expect_assignable(
							module,
							&signature.parameters[index].ty,
							&actual,
							expression.span,
							"function argument",
						);
					}
					assigned[index] = true;
				}
				_ => {}
			}
		}
		for (index, parameter) in signature.parameters.iter().enumerate() {
			if !assigned[index] && !parameter.has_default {
				self.error(
					module,
					"VTYPE002",
					format!("missing required argument `{}`", parameter.name),
					arguments.span,
				);
			}
		}
	}

	fn infer_index(&mut self, module: ModuleId, node: &SyntaxNode) -> Type {
		let mut expressions = node
			.child_nodes()
			.filter(|child| is_expression_kind(child.kind));
		let Some(target_node) = expressions.next() else {
			return Type::Unknown;
		};
		let Some(index_node) = expressions.next() else {
			return Type::Unknown;
		};
		let target = self.infer_expression(module, target_node);
		let index = self.infer_expression(module, index_node);
		match target {
			Type::Builtin(BuiltinType::List, arguments) => {
				self.expect_assignable(
					module,
					&Type::builtin(BuiltinType::Int),
					&index,
					index_node.span,
					"list index",
				);
				arguments.first().cloned().unwrap_or(Type::Unknown)
			}
			Type::Builtin(BuiltinType::Map, arguments) => {
				if let Some(key) = arguments.first() {
					self.expect_assignable(module, key, &index, index_node.span, "map index");
				}
				arguments.get(1).cloned().unwrap_or(Type::Unknown)
			}
			Type::Unknown | Type::Error => Type::Unknown,
			_ => {
				self.error(module, "VTYPE001", "value is not indexable", node.span);
				Type::Error
			}
		}
	}

	fn infer_try(&mut self, module: ModuleId, node: &SyntaxNode) -> Type {
		let Some(operand) = first_expression_child(node) else {
			return Type::Unknown;
		};
		match self.infer_expression(module, operand) {
			Type::Builtin(BuiltinType::Result | BuiltinType::Option, arguments) => {
				arguments.first().cloned().unwrap_or(Type::Unknown)
			}
			Type::Unknown | Type::Error => Type::Unknown,
			_ => {
				self.error(
					module,
					"VTYPE001",
					"`?` requires Result or Option",
					node.span,
				);
				Type::Error
			}
		}
	}

	fn resolve_type(&mut self, module: ModuleId, node: &SyntaxNode) -> Type {
		match node.kind {
			SyntaxKind::OptionalType => node
				.child_nodes()
				.find(|child| is_type_kind(child.kind))
				.map(|child| Type::option(self.resolve_type(module, child)))
				.unwrap_or(Type::Error),
			SyntaxKind::ParenthesizedType => node
				.child_nodes()
				.find(|child| is_type_kind(child.kind))
				.map(|child| self.resolve_type(module, child))
				.unwrap_or(Type::Error),
			SyntaxKind::FunctionType => self.resolve_function_type(module, node),
			SyntaxKind::TypeRef => self.resolve_type_ref(module, node),
			_ => Type::Error,
		}
	}

	fn resolve_function_type(&mut self, module: ModuleId, node: &SyntaxNode) -> Type {
		let mut children: Vec<&SyntaxNode> = node
			.child_nodes()
			.filter(|child| is_type_kind(child.kind))
			.collect();
		let return_type = children
			.pop()
			.map(|child| self.resolve_type(module, child))
			.unwrap_or_else(|| Type::builtin(BuiltinType::Void));
		let parameters = children
			.into_iter()
			.map(|child| FunctionParameter {
				name: String::new(),
				symbol: None,
				ty: self.resolve_type(module, child),
				has_default: false,
			})
			.collect();
		Type::Function(FunctionSignature {
			parameters,
			return_type: Box::new(return_type),
			is_async: false,
		})
	}

	fn resolve_type_ref(&mut self, module: ModuleId, node: &SyntaxNode) -> Type {
		if let Some(base) = node
			.child_nodes()
			.find(|child| child.kind == SyntaxKind::TypeRef)
		{
			let arguments: Vec<Type> = node
				.child_nodes()
				.find(|child| child.kind == SyntaxKind::GenericArgumentList)
				.into_iter()
				.flat_map(|list| list.child_nodes())
				.filter(|child| is_type_kind(child.kind))
				.map(|child| self.resolve_type(module, child))
				.collect();
			return self.type_from_reference(module, base.span, arguments, node.span);
		}
		self.type_from_reference(module, node.span, Vec::new(), node.span)
	}

	fn type_from_reference(
		&mut self,
		module: ModuleId,
		reference_span: Span,
		arguments: Vec<Type>,
		diagnostic_span: Span,
	) -> Type {
		let target = self.project.modules[module.0]
			.type_references
			.iter()
			.find(|reference| reference.span == reference_span)
			.and_then(|reference| reference.target.clone());
		match target {
			Some(TypeTarget::Builtin(builtin)) => {
				let expected = builtin.generic_arity();
				if arguments.len() != expected {
					self.error(
						module,
						"VTYPE006",
						format!(
							"builtin type {builtin:?} expects {expected} generic arguments, found {}",
							arguments.len()
						),
						diagnostic_span,
					);
					return Type::Error;
				}
				Type::Builtin(builtin, arguments)
			}
			Some(TypeTarget::Symbol(symbol)) => Type::Named(symbol, arguments),
			Some(TypeTarget::Generic(name)) => {
				if !arguments.is_empty() {
					self.error(
						module,
						"VTYPE006",
						"generic parameter cannot take generic arguments",
						diagnostic_span,
					);
					Type::Error
				} else {
					Type::Generic(name)
				}
			}
			None => Type::Error,
		}
	}

	fn first_type_child<'a>(&self, node: &'a SyntaxNode) -> Option<&'a SyntaxNode> {
		node.child_nodes().find(|child| is_type_kind(child.kind))
	}

	fn symbol_at_span(&self, module: ModuleId, span: Span) -> Option<SymbolId> {
		self.project.modules[module.0]
			.hir
			.symbols
			.iter()
			.find(|symbol| symbol.span == span)
			.map(|symbol| symbol.id)
	}

	fn symbol_type(&self, module: ModuleId, symbol: SymbolId) -> &Type {
		&self.result.modules[module.0].symbol_types[symbol.0]
	}

	fn symbol_type_global(&self, symbol: GlobalSymbolId) -> &Type {
		self.symbol_type(symbol.module, symbol.symbol)
	}

	fn set_symbol_type(&mut self, module: ModuleId, symbol: SymbolId, ty: Type) {
		self.result.modules[module.0].symbol_types[symbol.0] = ty;
	}

	fn expect_assignable(
		&mut self,
		module: ModuleId,
		expected: &Type,
		actual: &Type,
		span: Span,
		context: &str,
	) {
		if !is_assignable(expected, actual) {
			self.error(
				module,
				"VTYPE001",
				format!("{context} type mismatch: expected {expected:?}, found {actual:?}"),
				span,
			);
		}
	}

	fn error(
		&mut self,
		module: ModuleId,
		code: &'static str,
		message: impl Into<String>,
		span: Span,
	) {
		self.result.diagnostics.push(TypeDiagnostic {
			module,
			code,
			message: message.into(),
			span,
		});
	}
}

fn callee_node_is_unknown(checker: &Checker<'_>, module: ModuleId, node: &SyntaxNode) -> bool {
	if node.kind == SyntaxKind::NameExpr {
		let Some(token) = first_identifier(node) else {
			return true;
		};
		let resolved = checker.project.modules[module.0]
			.hir
			.references
			.iter()
			.find(|reference| reference.span == token.span)
			.and_then(|reference| reference.resolved);
		return resolved
			.is_none_or(|symbol| matches!(checker.symbol_type(module, symbol), Type::Unknown));
	}
	true
}

fn is_assignable(expected: &Type, actual: &Type) -> bool {
	if expected.is_unknown_or_error() || actual.is_unknown_or_error() {
		return true;
	}
	if matches!(actual, Type::None)
		&& matches!(expected, Type::Builtin(BuiltinType::Option, arguments) if arguments.len() == 1)
	{
		return true;
	}
	expected == actual
}

fn same_numeric_type(left: &Type, right: &Type) -> bool {
	left == right
		&& matches!(
			left,
			Type::Builtin(BuiltinType::Int | BuiltinType::Float, arguments) if arguments.is_empty()
		)
}

fn first_identifier(node: &SyntaxNode) -> Option<Token> {
	node.direct_tokens()
		.find(|token| token.kind == TokenKind::Identifier)
		.copied()
}

fn first_expression_child(node: &SyntaxNode) -> Option<&SyntaxNode> {
	node.child_nodes()
		.find(|child| is_expression_kind(child.kind))
}

fn significant_operator(node: &SyntaxNode) -> Option<TokenKind> {
	node.direct_tokens().find_map(|token| {
		matches!(
			token.kind,
			TokenKind::Plus
				| TokenKind::Minus
				| TokenKind::Star
				| TokenKind::Slash
				| TokenKind::Percent
				| TokenKind::Bang
				| TokenKind::Equal
				| TokenKind::PlusEqual
				| TokenKind::MinusEqual
				| TokenKind::StarEqual
				| TokenKind::SlashEqual
				| TokenKind::PercentEqual
				| TokenKind::EqualEqual
				| TokenKind::BangEqual
				| TokenKind::Less
				| TokenKind::LessEqual
				| TokenKind::Greater
				| TokenKind::GreaterEqual
				| TokenKind::AndAnd
				| TokenKind::OrOr
				| TokenKind::Keyword(Keyword::And | Keyword::Or | Keyword::Not | Keyword::Await)
		)
		.then_some(token.kind)
	})
}

fn is_expression_kind(kind: SyntaxKind) -> bool {
	voiles_syntax::ast::is_expression_kind(kind)
}

fn is_type_kind(kind: SyntaxKind) -> bool {
	matches!(
		kind,
		SyntaxKind::TypeRef
			| SyntaxKind::OptionalType
			| SyntaxKind::ParenthesizedType
			| SyntaxKind::FunctionType
	)
}
