mod declaration;
mod expr;
mod item;
mod pattern;
mod types;

#[cfg(test)]
mod tests;

use voiles_lexer::{Diagnostic, Keyword, Span, Token, TokenKind, lex};

use crate::{SyntaxElement, SyntaxKind, SyntaxNode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parse {
	pub root: SyntaxNode,
	pub diagnostics: Vec<Diagnostic>,
}

#[must_use]
pub fn parse(source: &str) -> Parse {
	let lexed = lex(source);
	let mut parser = Parser::new(&lexed.tokens);
	let root = parser.parse_module();
	let mut diagnostics = lexed.diagnostics;
	diagnostics.extend(parser.diagnostics);
	Parse { root, diagnostics }
}

pub(super) struct Parser<'tokens> {
	tokens: &'tokens [Token],
	position: usize,
	diagnostics: Vec<Diagnostic>,
}

impl<'tokens> Parser<'tokens> {
	fn new(tokens: &'tokens [Token]) -> Self {
		Self {
			tokens,
			position: 0,
			diagnostics: Vec::new(),
		}
	}

	fn parse_module(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		while !self.at(TokenKind::Eof) {
			self.eat_trivia(&mut children);
			if self.at(TokenKind::Eof) {
				break;
			}
			if self.at(TokenKind::Newline) {
				self.bump_into(&mut children);
				continue;
			}

			let start = self.position;
			children.push(self.parse_module_item().into());
			if self.position == start {
				children.push(
					self.recover_line("VPAR004", "unable to parse module item")
						.into(),
				);
			}
		}

		if self.at(TokenKind::Eof) {
			self.bump_into(&mut children);
		}
		SyntaxNode::new(SyntaxKind::Module, children)
	}

	fn parse_module_item(&mut self) -> SyntaxNode {
		match self.current_kind() {
			TokenKind::Keyword(Keyword::Import) => self.parse_import(false),
			TokenKind::Keyword(Keyword::Extern)
				if self.peek_significant_kind(1) == Some(TokenKind::Keyword(Keyword::Import)) =>
			{
				self.parse_import(true)
			}
			TokenKind::Keyword(Keyword::Export) => self.parse_export(),
			TokenKind::Keyword(Keyword::Const | Keyword::State | Keyword::Shared) => {
				self.parse_binding()
			}
			TokenKind::Keyword(Keyword::Param) => self.parse_param(),
			TokenKind::Keyword(Keyword::Fn | Keyword::Async) => self.parse_function(),
			TokenKind::Keyword(Keyword::Component) => self.parse_component(),
			TokenKind::Keyword(Keyword::Struct) => self.parse_struct(),
			TokenKind::Keyword(Keyword::Enum) => self.parse_enum(),
			_ => self.parse_statement(),
		}
	}

	fn parse_statement(&mut self) -> SyntaxNode {
		match self.current_kind() {
			TokenKind::Keyword(Keyword::Const | Keyword::State | Keyword::Shared) => {
				self.parse_binding()
			}
			TokenKind::Keyword(Keyword::Fn | Keyword::Async) => self.parse_function(),
			TokenKind::Keyword(Keyword::If) => self.parse_if(),
			TokenKind::Keyword(Keyword::For) => self.parse_for(),
			TokenKind::Keyword(Keyword::Match) => self.parse_match(),
			TokenKind::Keyword(Keyword::Return) => self.parse_return(),
			TokenKind::Keyword(Keyword::Slot) => self.parse_slot(),
			TokenKind::Keyword(Keyword::Init | Keyword::Mount | Keyword::Cleanup) => {
				self.parse_lifecycle_block()
			}
			TokenKind::Dedent | TokenKind::Eof => {
				self.error_here("VPAR004", "expected statement");
				SyntaxNode::new(SyntaxKind::Error, Vec::new())
			}
			_ => self.parse_expression_statement(),
		}
	}

	pub(super) fn parse_block(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		self.expect(&mut children, TokenKind::Colon, "expected `:` before block");
		self.expect(
			&mut children,
			TokenKind::Newline,
			"expected newline after block header",
		);
		self.expect(
			&mut children,
			TokenKind::Indent,
			"expected indented block body",
		);

		loop {
			self.eat_trivia(&mut children);
			if self.at(TokenKind::Dedent) {
				self.bump_into(&mut children);
				break;
			}
			if self.at(TokenKind::Eof) {
				self.error_here("VPAR003", "unterminated block; expected dedent");
				break;
			}
			if self.at(TokenKind::Newline) {
				self.bump_into(&mut children);
				continue;
			}

			let start = self.position;
			children.push(self.parse_statement().into());
			if self.position == start {
				children.push(
					self.recover_line("VPAR004", "unable to parse statement")
						.into(),
				);
			}
		}
		SyntaxNode::new(SyntaxKind::Block, children)
	}

	pub(super) fn begin_indented_block(&mut self, children: &mut Vec<SyntaxElement>) {
		self.expect(children, TokenKind::Colon, "expected `:` before block");
		self.expect(
			children,
			TokenKind::Newline,
			"expected newline after block header",
		);
		self.expect(children, TokenKind::Indent, "expected indented block body");
	}

	pub(super) fn finish_simple_line(&mut self, children: &mut Vec<SyntaxElement>) {
		self.eat_trivia(children);
		if self.at(TokenKind::Newline) {
			self.bump_into(children);
		} else {
			self.error_here("VPAR001", "expected end of logical line");
			if !matches!(self.current_kind(), TokenKind::Dedent | TokenKind::Eof) {
				children.push(
					self.recover_line("VPAR001", "unexpected tokens after statement")
						.into(),
				);
			}
		}
	}

	pub(super) fn recover_line(&mut self, code: &'static str, message: &'static str) -> SyntaxNode {
		let mut children = Vec::new();
		self.diagnostics
			.push(Diagnostic::error(code, message, self.current_span()));
		while !matches!(
			self.current_kind(),
			TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof
		) {
			self.bump_into(&mut children);
		}
		if self.at(TokenKind::Newline) || (children.is_empty() && !self.at(TokenKind::Eof)) {
			self.bump_into(&mut children);
		}
		SyntaxNode::new(SyntaxKind::Error, children)
	}

	pub(super) fn recover_until(&mut self, stop: &[TokenKind]) -> SyntaxNode {
		let mut children = Vec::new();
		while !stop.contains(&self.current_kind())
			&& !matches!(
				self.current_kind(),
				TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof
			) {
			self.bump_into(&mut children);
		}
		SyntaxNode::new(SyntaxKind::Error, children)
	}

	pub(super) fn expect(
		&mut self,
		children: &mut Vec<SyntaxElement>,
		kind: TokenKind,
		message: &'static str,
	) {
		self.eat_trivia(children);
		if self.at(kind) {
			self.bump_into(children);
		} else {
			self.error_here("VPAR001", message);
		}
	}

	pub(super) fn expect_keyword(
		&mut self,
		children: &mut Vec<SyntaxElement>,
		keyword: Keyword,
		message: &'static str,
	) {
		self.expect(children, TokenKind::Keyword(keyword), message);
	}

	pub(super) fn eat_trivia(&mut self, children: &mut Vec<SyntaxElement>) {
		while self.current_kind().is_trivia() {
			self.bump_into(children);
		}
	}

	pub(super) fn bump_into(&mut self, children: &mut Vec<SyntaxElement>) {
		if let Some(token) = self.tokens.get(self.position).copied() {
			children.push(token.into());
			self.position += 1;
		}
	}

	pub(super) fn error_here(&mut self, code: &'static str, message: &'static str) {
		self.diagnostics
			.push(Diagnostic::error(code, message, self.current_span()));
	}

	pub(super) fn at(&self, kind: TokenKind) -> bool {
		self.current_kind() == kind
	}

	pub(super) fn at_keyword(&self, keyword: Keyword) -> bool {
		self.at(TokenKind::Keyword(keyword))
	}

	pub(super) fn current_kind(&self) -> TokenKind {
		self.tokens
			.get(self.position)
			.map_or(TokenKind::Eof, |token| token.kind)
	}

	fn current_span(&self) -> Span {
		self.tokens
			.get(self.position)
			.map_or_else(|| Span::empty(0), |token| token.span)
	}

	pub(super) fn peek_significant_kind(&self, significant_offset: usize) -> Option<TokenKind> {
		let mut seen = 0;
		for token in &self.tokens[self.position..] {
			if token.kind.is_trivia() {
				continue;
			}
			if seen == significant_offset {
				return Some(token.kind);
			}
			seen += 1;
		}
		None
	}
}
