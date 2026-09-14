use voiles_lexer::{Keyword, TokenKind};

use crate::{SyntaxKind, SyntaxNode};

use super::Parser;

impl Parser<'_> {
	pub(super) fn parse_component(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		self.expect_keyword(&mut children, Keyword::Component, "expected `component`");
		self.expect(
			&mut children,
			TokenKind::Identifier,
			"expected component name",
		);
		children.push(self.parse_parameter_list().into());
		children.push(self.parse_block().into());
		SyntaxNode::new(SyntaxKind::ComponentDecl, children)
	}

	pub(super) fn parse_struct(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		self.expect_keyword(&mut children, Keyword::Struct, "expected `struct`");
		self.expect(
			&mut children,
			TokenKind::Identifier,
			"expected struct name",
		);

		let mut block = Vec::new();
		self.begin_indented_block(&mut block);
		while !matches!(self.current_kind(), TokenKind::Dedent | TokenKind::Eof) {
			self.eat_trivia(&mut block);
			if self.at(TokenKind::Newline) {
				self.bump_into(&mut block);
				continue;
			}
			if self.at(TokenKind::Dedent) || self.at(TokenKind::Eof) {
				break;
			}
			block.push(self.parse_struct_field().into());
		}
		if self.at(TokenKind::Dedent) {
			self.bump_into(&mut block);
		} else {
			self.error_here("VPAR003", "unterminated struct declaration");
		}
		children.push(SyntaxNode::new(SyntaxKind::DeclarationBlock, block).into());
		SyntaxNode::new(SyntaxKind::StructDecl, children)
	}

	fn parse_struct_field(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		self.expect(
			&mut children,
			TokenKind::Identifier,
			"expected struct field name",
		);
		self.expect(
			&mut children,
			TokenKind::Colon,
			"expected `:` after struct field name",
		);
		children.push(self.parse_type().into());
		if self.peek_significant_kind(0) == Some(TokenKind::Equal) {
			self.expect(&mut children, TokenKind::Equal, "expected `=`");
			children.push(self.parse_expression().into());
		}
		self.finish_simple_line(&mut children);
		SyntaxNode::new(SyntaxKind::StructField, children)
	}

	pub(super) fn parse_enum(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		self.expect_keyword(&mut children, Keyword::Enum, "expected `enum`");
		self.expect(
			&mut children,
			TokenKind::Identifier,
			"expected enum name",
		);
		if self.peek_significant_kind(0) == Some(TokenKind::Less) {
			children.push(self.parse_generic_parameter_list().into());
		}

		let mut block = Vec::new();
		self.begin_indented_block(&mut block);
		while !matches!(self.current_kind(), TokenKind::Dedent | TokenKind::Eof) {
			self.eat_trivia(&mut block);
			if self.at(TokenKind::Newline) {
				self.bump_into(&mut block);
				continue;
			}
			if self.at(TokenKind::Dedent) || self.at(TokenKind::Eof) {
				break;
			}
			block.push(self.parse_enum_case().into());
		}
		if self.at(TokenKind::Dedent) {
			self.bump_into(&mut block);
		} else {
			self.error_here("VPAR003", "unterminated enum declaration");
		}
		children.push(SyntaxNode::new(SyntaxKind::DeclarationBlock, block).into());
		SyntaxNode::new(SyntaxKind::EnumDecl, children)
	}

	fn parse_generic_parameter_list(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		self.expect(&mut children, TokenKind::Less, "expected `<`");
		loop {
			self.expect(
				&mut children,
				TokenKind::Identifier,
				"expected generic parameter name",
			);
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
			"expected `>` after generic parameters",
		);
		SyntaxNode::new(SyntaxKind::GenericParameterList, children)
	}

	fn parse_enum_case(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		self.expect(
			&mut children,
			TokenKind::Identifier,
			"expected enum case name",
		);
		if self.peek_significant_kind(0) == Some(TokenKind::LParen) {
			self.expect(&mut children, TokenKind::LParen, "expected `(`");
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
				"expected `)` after enum payload types",
			);
		}
		self.finish_simple_line(&mut children);
		SyntaxNode::new(SyntaxKind::EnumCase, children)
	}

	pub(super) fn parse_slot(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		self.expect_keyword(&mut children, Keyword::Slot, "expected `slot`");
		if self.peek_significant_kind(0) == Some(TokenKind::Identifier) {
			self.expect(
				&mut children,
				TokenKind::Identifier,
				"expected slot name",
			);
		}
		if self.peek_significant_kind(0) == Some(TokenKind::Colon) {
			children.push(self.parse_block().into());
		} else {
			self.finish_simple_line(&mut children);
		}
		SyntaxNode::new(SyntaxKind::SlotStmt, children)
	}
}
