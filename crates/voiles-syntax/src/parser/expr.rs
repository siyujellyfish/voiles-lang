use voiles_lexer::{Keyword, TokenKind};

use crate::{SyntaxKind, SyntaxNode};

use super::Parser;

impl Parser<'_> {
	pub(super) fn parse_expression(&mut self) -> SyntaxNode {
		self.parse_expression_bp(0)
	}

	fn parse_expression_bp(&mut self, minimum_bp: u8) -> SyntaxNode {
		let mut left = self.parse_prefix_expression();

		loop {
			match self.peek_significant_kind(0) {
				Some(TokenKind::LParen) => {
					let mut children = vec![left.into()];
					self.eat_trivia(&mut children);
					children.push(self.parse_argument_list().into());
					left = SyntaxNode::new(SyntaxKind::CallExpr, children);
					continue;
				}
				Some(TokenKind::Dot) => {
					let mut children = vec![left.into()];
					self.expect(&mut children, TokenKind::Dot, "expected `.`");
					self.expect(
						&mut children,
						TokenKind::Identifier,
						"expected member name after `.`",
					);
					left = SyntaxNode::new(SyntaxKind::MemberExpr, children);
					continue;
				}
				Some(TokenKind::LBracket) => {
					let mut children = vec![left.into()];
					self.expect(&mut children, TokenKind::LBracket, "expected `[` ");
					children.push(self.parse_expression().into());
					self.expect(
						&mut children,
						TokenKind::RBracket,
						"expected `]` after index expression",
					);
					left = SyntaxNode::new(SyntaxKind::IndexExpr, children);
					continue;
				}
				Some(TokenKind::Question) => {
					let mut children = vec![left.into()];
					self.expect(&mut children, TokenKind::Question, "expected `?`");
					left = SyntaxNode::new(SyntaxKind::TryExpr, children);
					continue;
				}
				_ => {}
			}

			let Some(operator) = self.peek_significant_kind(0) else {
				break;
			};
			let Some((left_bp, right_bp, kind)) = infix_binding_power(operator) else {
				break;
			};
			if left_bp < minimum_bp {
				break;
			}

			let mut children = vec![left.into()];
			self.eat_trivia(&mut children);
			self.bump_into(&mut children);
			children.push(self.parse_expression_bp(right_bp).into());
			left = SyntaxNode::new(kind, children);
		}

		left
	}

	fn parse_prefix_expression(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		self.eat_trivia(&mut children);
		match self.current_kind() {
			TokenKind::Identifier => {
				self.bump_into(&mut children);
				SyntaxNode::new(SyntaxKind::NameExpr, children)
			}
			TokenKind::Integer | TokenKind::Float | TokenKind::String => {
				self.bump_into(&mut children);
				SyntaxNode::new(SyntaxKind::LiteralExpr, children)
			}
			TokenKind::Keyword(Keyword::True | Keyword::False | Keyword::None) => {
				self.bump_into(&mut children);
				SyntaxNode::new(SyntaxKind::LiteralExpr, children)
			}
			TokenKind::Plus
			| TokenKind::Minus
			| TokenKind::Bang
			| TokenKind::Keyword(Keyword::Not | Keyword::Await) => {
				self.bump_into(&mut children);
				children.push(self.parse_expression_bp(15).into());
				SyntaxNode::new(SyntaxKind::UnaryExpr, children)
			}
			TokenKind::LParen => {
				self.bump_into(&mut children);
				children.push(self.parse_expression().into());
				self.expect(
					&mut children,
					TokenKind::RParen,
					"expected `)` after expression",
				);
				SyntaxNode::new(SyntaxKind::ParenthesizedExpr, children)
			}
			_ => {
				self.error_here("VPAR002", "expected expression");
				if !is_expression_boundary(self.current_kind()) {
					self.bump_into(&mut children);
				}
				SyntaxNode::new(SyntaxKind::Error, children)
			}
		}
	}

	fn parse_argument_list(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		let mut seen_named = false;
		self.expect(
			&mut children,
			TokenKind::LParen,
			"expected `(` before arguments",
		);

		loop {
			self.eat_trivia(&mut children);
			if self.at(TokenKind::RParen) || self.at(TokenKind::Eof) {
				break;
			}

			let is_named = self.at(TokenKind::Identifier)
				&& self.peek_significant_kind(1) == Some(TokenKind::Equal);
			let argument = if is_named {
				seen_named = true;
				let mut argument_children = Vec::new();
				self.bump_into(&mut argument_children);
				self.expect(
					&mut argument_children,
					TokenKind::Equal,
					"expected `=` in named argument",
				);
				argument_children.push(self.parse_expression().into());
				SyntaxNode::new(SyntaxKind::NamedArgument, argument_children)
			} else {
				if seen_named {
					self.error_here(
						"VPAR007",
						"positional argument cannot follow a named argument",
					);
				}
				SyntaxNode::new(
					SyntaxKind::PositionalArgument,
					vec![self.parse_expression().into()],
				)
			};
			children.push(argument.into());
			self.eat_trivia(&mut children);

			if self.at(TokenKind::Comma) {
				self.bump_into(&mut children);
				continue;
			}
			if !self.at(TokenKind::RParen) {
				self.error_here("VPAR006", "expected `,` or `)` after argument");
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
			"expected `)` after arguments",
		);
		SyntaxNode::new(SyntaxKind::ArgumentList, children)
	}
}

fn infix_binding_power(kind: TokenKind) -> Option<(u8, u8, SyntaxKind)> {
	let result = match kind {
		TokenKind::Equal
		| TokenKind::PlusEqual
		| TokenKind::MinusEqual
		| TokenKind::StarEqual
		| TokenKind::SlashEqual
		| TokenKind::PercentEqual => (1, 1, SyntaxKind::AssignmentExpr),
		TokenKind::Keyword(Keyword::Or) | TokenKind::OrOr => (3, 4, SyntaxKind::BinaryExpr),
		TokenKind::Keyword(Keyword::And) | TokenKind::AndAnd => (5, 6, SyntaxKind::BinaryExpr),
		TokenKind::EqualEqual | TokenKind::BangEqual => (7, 8, SyntaxKind::BinaryExpr),
		TokenKind::Less | TokenKind::LessEqual | TokenKind::Greater | TokenKind::GreaterEqual => {
			(9, 10, SyntaxKind::BinaryExpr)
		}
		TokenKind::Plus | TokenKind::Minus => (11, 12, SyntaxKind::BinaryExpr),
		TokenKind::Star | TokenKind::Slash | TokenKind::Percent => (13, 14, SyntaxKind::BinaryExpr),
		_ => return None,
	};
	Some(result)
}

fn is_expression_boundary(kind: TokenKind) -> bool {
	matches!(
		kind,
		TokenKind::Newline
			| TokenKind::Dedent
			| TokenKind::Eof
			| TokenKind::Comma
			| TokenKind::RParen
			| TokenKind::RBracket
			| TokenKind::Colon
	)
}
