use voiles_lexer::{Keyword, TokenKind};

use crate::{SyntaxKind, SyntaxNode};

use super::Parser;

impl Parser<'_> {
	pub(super) fn parse_match(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		self.expect_keyword(&mut children, Keyword::Match, "expected `match`");
		children.push(self.parse_expression().into());

		let mut arms = Vec::new();
		self.begin_indented_block(&mut arms);
		while !matches!(self.current_kind(), TokenKind::Dedent | TokenKind::Eof) {
			self.eat_trivia(&mut arms);
			if self.at(TokenKind::Newline) {
				self.bump_into(&mut arms);
				continue;
			}
			if self.at(TokenKind::Dedent) || self.at(TokenKind::Eof) {
				break;
			}
			arms.push(self.parse_match_arm().into());
		}
		if self.at(TokenKind::Dedent) {
			self.bump_into(&mut arms);
		} else {
			self.error_here("VPAR003", "unterminated match statement");
		}
		children.push(SyntaxNode::new(SyntaxKind::DeclarationBlock, arms).into());
		SyntaxNode::new(SyntaxKind::MatchStmt, children)
	}

	fn parse_match_arm(&mut self) -> SyntaxNode {
		let mut children = vec![self.parse_pattern().into()];
		children.push(self.parse_block().into());
		SyntaxNode::new(SyntaxKind::MatchArm, children)
	}

	fn parse_pattern(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		self.eat_trivia(&mut children);
		match self.current_kind() {
			TokenKind::Identifier => {
				self.bump_into(&mut children);
				while self.peek_significant_kind(0) == Some(TokenKind::Dot) {
					self.expect(&mut children, TokenKind::Dot, "expected `.` in pattern");
					self.expect(
						&mut children,
						TokenKind::Identifier,
						"expected qualified pattern case",
					);
				}
				if self.peek_significant_kind(0) == Some(TokenKind::LParen) {
					self.expect(&mut children, TokenKind::LParen, "expected `(`");
					loop {
						self.eat_trivia(&mut children);
						if self.at(TokenKind::RParen) || self.at(TokenKind::Eof) {
							break;
						}
						children.push(self.parse_pattern().into());
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
						"expected `)` after pattern payload",
					);
				}
			}
			TokenKind::Integer | TokenKind::Float | TokenKind::String => {
				self.bump_into(&mut children);
			}
			TokenKind::Keyword(Keyword::True | Keyword::False | Keyword::None) => {
				self.bump_into(&mut children);
			}
			_ => {
				self.error_here("VPAR008", "expected match pattern");
				if !matches!(self.current_kind(), TokenKind::Colon | TokenKind::Eof) {
					self.bump_into(&mut children);
				}
			}
		}
		SyntaxNode::new(SyntaxKind::Pattern, children)
	}
}
