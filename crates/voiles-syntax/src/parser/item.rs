use voiles_lexer::{Keyword, TokenKind};

use crate::{SyntaxKind, SyntaxNode};

use super::Parser;

impl Parser<'_> {
	pub(super) fn parse_import(&mut self, is_extern: bool) -> SyntaxNode {
		let mut children = Vec::new();
		let kind = if is_extern {
			self.expect_keyword(
				&mut children,
				Keyword::Extern,
				"expected `extern` before foreign import",
			);
			SyntaxKind::ExternImportDecl
		} else {
			SyntaxKind::ImportDecl
		};
		self.expect_keyword(&mut children, Keyword::Import, "expected `import`");
		self.eat_trivia(&mut children);

		if self.at(TokenKind::Star) {
			let mut namespace = Vec::new();
			self.bump_into(&mut namespace);
			self.expect_keyword(&mut namespace, Keyword::As, "expected `as` after `*`");
			self.expect(
				&mut namespace,
				TokenKind::Identifier,
				"expected namespace import name",
			);
			children.push(SyntaxNode::new(SyntaxKind::NamespaceImport, namespace).into());
		} else {
			loop {
				self.eat_trivia(&mut children);
				if !self.at(TokenKind::Identifier) {
					self.error_here("VPAR001", "expected imported symbol name");
					break;
				}
				children.push(self.parse_import_item().into());
				self.eat_trivia(&mut children);
				if self.at(TokenKind::Comma) {
					self.bump_into(&mut children);
					continue;
				}
				break;
			}
		}

		self.expect_keyword(&mut children, Keyword::From, "expected `from` in import");
		self.expect(
			&mut children,
			TokenKind::String,
			"expected quoted module specifier",
		);
		self.finish_simple_line(&mut children);
		SyntaxNode::new(kind, children)
	}

	fn parse_import_item(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		self.expect(
			&mut children,
			TokenKind::Identifier,
			"expected imported symbol name",
		);
		if self.peek_significant_kind(0) == Some(TokenKind::Keyword(Keyword::As)) {
			self.expect_keyword(&mut children, Keyword::As, "expected `as`");
			self.expect(
				&mut children,
				TokenKind::Identifier,
				"expected import alias name",
			);
		}
		SyntaxNode::new(SyntaxKind::ImportItem, children)
	}

	pub(super) fn parse_export(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		self.expect_keyword(&mut children, Keyword::Export, "expected `export`");
		self.eat_trivia(&mut children);
		let declaration = match self.current_kind() {
			TokenKind::Keyword(Keyword::Const | Keyword::State | Keyword::Shared) => {
				Some(self.parse_binding())
			}
			TokenKind::Keyword(Keyword::Fn | Keyword::Async) => Some(self.parse_function()),
			_ => None,
		};

		if let Some(declaration) = declaration {
			children.push(declaration.into());
		} else {
			self.error_here("VPAR005", "expected exportable declaration after `export`");
			children.push(
				self.recover_line("VPAR005", "invalid export declaration")
					.into(),
			);
		}
		SyntaxNode::new(SyntaxKind::ExportDecl, children)
	}

	pub(super) fn parse_binding(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		self.bump_into(&mut children);
		self.expect(
			&mut children,
			TokenKind::Identifier,
			"expected binding name",
		);

		if self.peek_significant_kind(0) == Some(TokenKind::Colon) {
			self.expect(&mut children, TokenKind::Colon, "expected `:`");
			children.push(self.parse_type().into());
		}

		self.expect(
			&mut children,
			TokenKind::Equal,
			"expected `=` in binding declaration",
		);
		children.push(self.parse_expression().into());
		self.finish_simple_line(&mut children);
		SyntaxNode::new(SyntaxKind::BindingDecl, children)
	}

	pub(super) fn parse_function(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		if self.at_keyword(Keyword::Async) {
			self.bump_into(&mut children);
			self.expect_keyword(&mut children, Keyword::Fn, "expected `fn` after `async`");
		} else {
			self.expect_keyword(&mut children, Keyword::Fn, "expected `fn`");
		}

		self.expect(
			&mut children,
			TokenKind::Identifier,
			"expected function name",
		);
		children.push(self.parse_parameter_list().into());

		if self.peek_significant_kind(0) == Some(TokenKind::Arrow) {
			let mut return_type = Vec::new();
			self.expect(&mut return_type, TokenKind::Arrow, "expected `->`");
			return_type.push(self.parse_type().into());
			children.push(SyntaxNode::new(SyntaxKind::ReturnType, return_type).into());
		}

		children.push(self.parse_block().into());
		SyntaxNode::new(SyntaxKind::FunctionDecl, children)
	}

	fn parse_parameter_list(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		self.expect(
			&mut children,
			TokenKind::LParen,
			"expected `(` before parameters",
		);

		loop {
			self.eat_trivia(&mut children);
			if self.at(TokenKind::RParen) || self.at(TokenKind::Eof) {
				break;
			}
			children.push(self.parse_parameter().into());
			self.eat_trivia(&mut children);
			if self.at(TokenKind::Comma) {
				self.bump_into(&mut children);
				continue;
			}
			if !self.at(TokenKind::RParen) {
				self.error_here("VPAR006", "expected `,` or `)` after parameter");
				children.push(
					self.recover_until(&[TokenKind::Comma, TokenKind::RParen])
						.into(),
				);
				if self.at(TokenKind::Comma) {
					self.bump_into(&mut children);
				}
			}
		}

		self.expect(
			&mut children,
			TokenKind::RParen,
			"expected `)` after parameters",
		);
		SyntaxNode::new(SyntaxKind::ParameterList, children)
	}

	fn parse_parameter(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		self.expect(
			&mut children,
			TokenKind::Identifier,
			"expected parameter name",
		);
		if self.peek_significant_kind(0) == Some(TokenKind::Colon) {
			self.expect(&mut children, TokenKind::Colon, "expected `:`");
			children.push(self.parse_type().into());
		}
		if self.peek_significant_kind(0) == Some(TokenKind::Equal) {
			self.expect(&mut children, TokenKind::Equal, "expected `=`");
			children.push(self.parse_expression().into());
		}
		SyntaxNode::new(SyntaxKind::Parameter, children)
	}

	pub(super) fn parse_if(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		self.expect_keyword(&mut children, Keyword::If, "expected `if`");
		children.push(self.parse_expression().into());
		children.push(self.parse_block().into());

		self.eat_trivia(&mut children);
		if self.at_keyword(Keyword::Else) {
			let mut else_children = Vec::new();
			self.bump_into(&mut else_children);
			else_children.push(self.parse_block().into());
			children.push(SyntaxNode::new(SyntaxKind::ElseClause, else_children).into());
		}
		SyntaxNode::new(SyntaxKind::IfStmt, children)
	}

	pub(super) fn parse_for(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		self.expect_keyword(&mut children, Keyword::For, "expected `for`");
		self.expect(
			&mut children,
			TokenKind::Identifier,
			"expected loop binding name",
		);
		self.expect_keyword(&mut children, Keyword::In, "expected `in` in loop");
		children.push(self.parse_expression().into());

		if self.peek_significant_kind(0) == Some(TokenKind::Keyword(Keyword::Key)) {
			let mut key_children = Vec::new();
			self.expect_keyword(&mut key_children, Keyword::Key, "expected `key`");
			key_children.push(self.parse_expression().into());
			children.push(SyntaxNode::new(SyntaxKind::KeyClause, key_children).into());
		}

		children.push(self.parse_block().into());
		SyntaxNode::new(SyntaxKind::ForStmt, children)
	}

	pub(super) fn parse_return(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		self.expect_keyword(&mut children, Keyword::Return, "expected `return`");
		if self.peek_significant_kind(0) != Some(TokenKind::Newline) {
			children.push(self.parse_expression().into());
		}
		self.finish_simple_line(&mut children);
		SyntaxNode::new(SyntaxKind::ReturnStmt, children)
	}

	pub(super) fn parse_lifecycle_block(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		self.bump_into(&mut children);
		children.push(self.parse_block().into());
		SyntaxNode::new(SyntaxKind::LifecycleBlock, children)
	}

	pub(super) fn parse_expression_statement(&mut self) -> SyntaxNode {
		let mut children = vec![self.parse_expression().into()];
		self.finish_simple_line(&mut children);
		SyntaxNode::new(SyntaxKind::ExprStmt, children)
	}
}
