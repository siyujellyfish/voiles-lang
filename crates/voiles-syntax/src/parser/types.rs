use voiles_lexer::{Keyword, TokenKind};

use crate::{SyntaxKind, SyntaxNode};

use super::Parser;

impl Parser<'_> {
	pub(super) fn parse_type(&mut self) -> SyntaxNode {
		let mut node = self.parse_type_primary();
		loop {
			match self.peek_significant_kind(0) {
				Some(TokenKind::Less) => {
					let arguments = self.parse_generic_arguments();
					node =
						SyntaxNode::new(SyntaxKind::TypeRef, vec![node.into(), arguments.into()]);
				}
				Some(TokenKind::Question) => {
					let mut children = vec![node.into()];
					self.expect(&mut children, TokenKind::Question, "expected `?`");
					node = SyntaxNode::new(SyntaxKind::OptionalType, children);
				}
				_ => break,
			}
		}
		node
	}

	fn parse_type_primary(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		self.eat_trivia(&mut children);
		match self.current_kind() {
			TokenKind::Identifier => {
				self.bump_into(&mut children);
				while self.peek_significant_kind(0) == Some(TokenKind::Dot) {
					self.expect(&mut children, TokenKind::Dot, "expected `.`");
					self.expect(
						&mut children,
						TokenKind::Identifier,
						"expected qualified type name",
					);
				}
				SyntaxNode::new(SyntaxKind::TypeRef, children)
			}
			TokenKind::Keyword(Keyword::Fn) => self.parse_function_type(children),
			TokenKind::LParen => {
				self.bump_into(&mut children);
				children.push(self.parse_type().into());
				self.expect(&mut children, TokenKind::RParen, "expected `)` after type");
				SyntaxNode::new(SyntaxKind::ParenthesizedType, children)
			}
			_ => {
				self.error_here("VPAR001", "expected type");
				if !matches!(
					self.current_kind(),
					TokenKind::Newline
						| TokenKind::Dedent
						| TokenKind::Eof | TokenKind::Comma
						| TokenKind::RParen
				) {
					self.bump_into(&mut children);
				}
				SyntaxNode::new(SyntaxKind::Error, children)
			}
		}
	}

	fn parse_function_type(&mut self, mut children: Vec<crate::SyntaxElement>) -> SyntaxNode {
		self.bump_into(&mut children);
		self.expect(
			&mut children,
			TokenKind::LParen,
			"expected `(` in function type",
		);
		loop {
			self.eat_trivia(&mut children);
			if self.at(TokenKind::RParen) || self.at(TokenKind::Eof) {
				break;
			}
			children.push(self.parse_type().into());
			self.eat_trivia(&mut children);
			if self.at(TokenKind::Comma) {
				self.bump_into(&mut children);
				continue;
			}
			break;
		}
		self.expect(
			&mut children,
			TokenKind::RParen,
			"expected `)` in function type",
		);
		self.expect(
			&mut children,
			TokenKind::Arrow,
			"expected `->` in function type",
		);
		children.push(self.parse_type().into());
		SyntaxNode::new(SyntaxKind::FunctionType, children)
	}

	fn parse_generic_arguments(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		self.expect(&mut children, TokenKind::Less, "expected `<`");
		loop {
			self.eat_trivia(&mut children);
			if self.at(TokenKind::Greater) || self.at(TokenKind::Eof) {
				break;
			}
			children.push(self.parse_type().into());
			self.eat_trivia(&mut children);
			if self.at(TokenKind::Comma) {
				self.bump_into(&mut children);
				continue;
			}
			break;
		}
		self.expect(
			&mut children,
			TokenKind::Greater,
			"expected `>` after generic arguments",
		);
		SyntaxNode::new(SyntaxKind::GenericArgumentList, children)
	}
}
