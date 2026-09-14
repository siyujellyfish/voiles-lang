use voiles_lexer::{Keyword, Token, TokenKind};

use crate::{SyntaxKind, SyntaxNode};

pub trait AstNode<'a>: Sized {
	const KIND: SyntaxKind;

	fn cast(node: &'a SyntaxNode) -> Option<Self>;
	fn syntax(&self) -> &'a SyntaxNode;
}

macro_rules! ast_node {
	($name:ident, $kind:ident) => {
		#[derive(Debug, Clone, Copy)]
		pub struct $name<'a> {
			syntax: &'a SyntaxNode,
		}

		impl<'a> AstNode<'a> for $name<'a> {
			const KIND: SyntaxKind = SyntaxKind::$kind;

			fn cast(node: &'a SyntaxNode) -> Option<Self> {
				(node.kind == Self::KIND).then_some(Self { syntax: node })
			}

			fn syntax(&self) -> &'a SyntaxNode {
				self.syntax
			}
		}
	};
}

ast_node!(Module, Module);
ast_node!(ImportDecl, ImportDecl);
ast_node!(ExternImportDecl, ExternImportDecl);
ast_node!(ExportDecl, ExportDecl);
ast_node!(BindingDecl, BindingDecl);
ast_node!(ParamDecl, ParamDecl);
ast_node!(FunctionDecl, FunctionDecl);
ast_node!(ComponentDecl, ComponentDecl);
ast_node!(StructDecl, StructDecl);
ast_node!(EnumDecl, EnumDecl);
ast_node!(Parameter, Parameter);
ast_node!(Block, Block);
ast_node!(ForStmt, ForStmt);
ast_node!(MatchArm, MatchArm);
ast_node!(LifecycleBlock, LifecycleBlock);
ast_node!(NameExpr, NameExpr);

#[derive(Debug, Clone, Copy)]
pub enum Item<'a> {
	Import(ImportDecl<'a>),
	ExternImport(ExternImportDecl<'a>),
	Export(ExportDecl<'a>),
	Binding(BindingDecl<'a>),
	Param(ParamDecl<'a>),
	Function(FunctionDecl<'a>),
	Component(ComponentDecl<'a>),
	Struct(StructDecl<'a>),
	Enum(EnumDecl<'a>),
	Other(&'a SyntaxNode),
}

impl<'a> Module<'a> {
	pub fn items(self) -> impl Iterator<Item = Item<'a>> {
		self.syntax.child_nodes().map(|node| match node.kind {
			SyntaxKind::ImportDecl => Item::Import(ImportDecl::cast(node).expect("kind checked")),
			SyntaxKind::ExternImportDecl => {
				Item::ExternImport(ExternImportDecl::cast(node).expect("kind checked"))
			}
			SyntaxKind::ExportDecl => Item::Export(ExportDecl::cast(node).expect("kind checked")),
			SyntaxKind::BindingDecl => {
				Item::Binding(BindingDecl::cast(node).expect("kind checked"))
			}
			SyntaxKind::ParamDecl => Item::Param(ParamDecl::cast(node).expect("kind checked")),
			SyntaxKind::FunctionDecl => {
				Item::Function(FunctionDecl::cast(node).expect("kind checked"))
			}
			SyntaxKind::ComponentDecl => {
				Item::Component(ComponentDecl::cast(node).expect("kind checked"))
			}
			SyntaxKind::StructDecl => Item::Struct(StructDecl::cast(node).expect("kind checked")),
			SyntaxKind::EnumDecl => Item::Enum(EnumDecl::cast(node).expect("kind checked")),
			_ => Item::Other(node),
		})
	}
}

impl<'a> ExportDecl<'a> {
	pub fn declaration(self) -> Option<&'a SyntaxNode> {
		self.syntax.child_nodes().next()
	}
}

impl<'a> BindingDecl<'a> {
	pub fn keyword(self) -> Option<Keyword> {
		self.syntax
			.direct_tokens()
			.find_map(|token| match token.kind {
				TokenKind::Keyword(
					keyword @ (Keyword::Const | Keyword::State | Keyword::Shared),
				) => Some(keyword),
				_ => None,
			})
	}

	pub fn name_token(self) -> Option<Token> {
		first_identifier(self.syntax)
	}

	pub fn initializer(self) -> Option<&'a SyntaxNode> {
		self.syntax
			.child_nodes()
			.find(|node| is_expression_kind(node.kind))
	}
}

impl ParamDecl<'_> {
	pub fn name_token(self) -> Option<Token> {
		first_identifier(self.syntax)
	}
}

impl<'a> FunctionDecl<'a> {
	pub fn name_token(self) -> Option<Token> {
		first_identifier(self.syntax)
	}

	pub fn parameters(self) -> impl Iterator<Item = Parameter<'a>> {
		self.syntax
			.child_nodes()
			.find(|node| node.kind == SyntaxKind::ParameterList)
			.into_iter()
			.flat_map(|list| list.child_nodes())
			.filter_map(Parameter::cast)
	}

	pub fn body(self) -> Option<Block<'a>> {
		self.syntax.child_nodes().find_map(Block::cast)
	}
}

impl<'a> ComponentDecl<'a> {
	pub fn name_token(self) -> Option<Token> {
		first_identifier(self.syntax)
	}

	pub fn parameters(self) -> impl Iterator<Item = Parameter<'a>> {
		self.syntax
			.child_nodes()
			.find(|node| node.kind == SyntaxKind::ParameterList)
			.into_iter()
			.flat_map(|list| list.child_nodes())
			.filter_map(Parameter::cast)
	}

	pub fn body(self) -> Option<Block<'a>> {
		self.syntax.child_nodes().find_map(Block::cast)
	}
}

impl<'a> Parameter<'a> {
	pub fn name_token(self) -> Option<Token> {
		first_identifier(self.syntax)
	}

	pub fn default_expression(self) -> Option<&'a SyntaxNode> {
		self.syntax
			.child_nodes()
			.find(|node| is_expression_kind(node.kind))
	}
}

impl StructDecl<'_> {
	pub fn name_token(self) -> Option<Token> {
		first_identifier(self.syntax)
	}
}

impl EnumDecl<'_> {
	pub fn name_token(self) -> Option<Token> {
		first_identifier(self.syntax)
	}
}

impl ForStmt<'_> {
	pub fn binding_token(self) -> Option<Token> {
		first_identifier(self.syntax)
	}
}

impl<'a> LifecycleBlock<'a> {
	pub fn keyword(self) -> Option<Keyword> {
		self.syntax
			.direct_tokens()
			.find_map(|token| match token.kind {
				TokenKind::Keyword(
					keyword @ (Keyword::Init | Keyword::Mount | Keyword::Cleanup),
				) => Some(keyword),
				_ => None,
			})
	}

	pub fn body(self) -> Option<Block<'a>> {
		self.syntax.child_nodes().find_map(Block::cast)
	}
}

impl NameExpr<'_> {
	pub fn token(self) -> Option<Token> {
		first_identifier(self.syntax)
	}
}

#[must_use]
pub fn token_text(source: &str, token: Token) -> &str {
	&source[token.span.start..token.span.end]
}

#[must_use]
pub const fn is_expression_kind(kind: SyntaxKind) -> bool {
	matches!(
		kind,
		SyntaxKind::NameExpr
			| SyntaxKind::LiteralExpr
			| SyntaxKind::ParenthesizedExpr
			| SyntaxKind::UnaryExpr
			| SyntaxKind::BinaryExpr
			| SyntaxKind::AssignmentExpr
			| SyntaxKind::CallExpr
			| SyntaxKind::MemberExpr
			| SyntaxKind::IndexExpr
			| SyntaxKind::TryExpr
	)
}

fn first_identifier(node: &SyntaxNode) -> Option<Token> {
	node.direct_tokens()
		.find(|token| token.kind == TokenKind::Identifier)
		.copied()
}

#[cfg(test)]
mod tests {
	use super::{AstNode, BindingDecl, ComponentDecl, Module, token_text};
	use crate::parse;

	#[test]
	fn typed_ast_exposes_declaration_names_without_losing_cst() {
		let source = "state count = 0\ncomponent Counter(value: Int):\n\tstate local = value\n";
		let parsed = parse(source);
		let module = Module::cast(&parsed.root).expect("module");
		let mut items = module.syntax().child_nodes();

		let binding = BindingDecl::cast(items.next().expect("binding")).expect("binding ast");
		assert_eq!(
			token_text(source, binding.name_token().expect("name")),
			"count"
		);

		let component =
			ComponentDecl::cast(items.next().expect("component")).expect("component ast");
		assert_eq!(
			token_text(source, component.name_token().expect("name")),
			"Counter"
		);
		assert_eq!(parsed.root.source_text(source), source);
	}
}
