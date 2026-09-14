use std::cmp::Ordering;

use crate::{Diagnostic, Keyword, Span, Token, TokenKind};

const INDENT_TAB_WIDTH: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lexed {
	pub tokens: Vec<Token>,
	pub diagnostics: Vec<Diagnostic>,
}

#[must_use]
pub fn lex(source: &str) -> Lexed {
	Lexer::new(source).run()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IndentLevel {
	column: usize,
	alternate_column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Delimiter {
	Paren,
	Bracket,
	Brace,
}

impl Delimiter {
	const fn closing_text(self) -> &'static str {
		match self {
			Self::Paren => ")",
			Self::Bracket => "]",
			Self::Brace => "}",
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OpenDelimiter {
	kind: Delimiter,
	span: Span,
}

struct Lexer<'source> {
	source: &'source str,
	position: usize,
	tokens: Vec<Token>,
	diagnostics: Vec<Diagnostic>,
	indent_stack: Vec<IndentLevel>,
	delimiters: Vec<OpenDelimiter>,
	at_line_start: bool,
	logical_line_has_code: bool,
}

impl<'source> Lexer<'source> {
	fn new(source: &'source str) -> Self {
		Self {
			source,
			position: 0,
			tokens: Vec::new(),
			diagnostics: Vec::new(),
			indent_stack: vec![IndentLevel {
				column: 0,
				alternate_column: 0,
			}],
			delimiters: Vec::new(),
			at_line_start: true,
			logical_line_has_code: false,
		}
	}

	fn run(mut self) -> Lexed {
		while self.position < self.source.len() {
			if self.at_line_start {
				self.prepare_line_start();
				if self.position >= self.source.len() {
					break;
				}
			}

			if self.is_line_break_start() {
				self.lex_line_break();
				continue;
			}

			let Some(character) = self.peek_char() else {
				break;
			};

			match character {
				' ' | '\t' => self.lex_whitespace(),
				'#' => self.lex_comment(),
				'\'' | '"' => self.lex_string(),
				character if character.is_ascii_digit() => self.lex_number(),
				character if is_identifier_start(character) => self.lex_identifier_or_keyword(),
				_ => self.lex_punctuation_or_operator(),
			}
		}

		self.finish();

		Lexed {
			tokens: self.tokens,
			diagnostics: self.diagnostics,
		}
	}

	fn prepare_line_start(&mut self) {
		let start = self.position;
		let mut column = 0;
		let mut alternate_column = 0;

		while let Some(byte) = self.peek_byte() {
			match byte {
				b' ' => {
					self.position += 1;
					column += 1;
					alternate_column += 1;
				}
				b'\t' => {
					self.position += 1;
					column = ((column / INDENT_TAB_WIDTH) + 1) * INDENT_TAB_WIDTH;
					alternate_column += 1;
				}
				_ => break,
			}
		}

		if self.position > start {
			self.emit(TokenKind::Whitespace, Span::new(start, self.position));
		}

		if self.position >= self.source.len() {
			return;
		}

		if self.is_line_break_start() || self.peek_char() == Some('#') {
			self.at_line_start = false;
			return;
		}

		if self.delimiters.is_empty() {
			self.apply_indentation(column, alternate_column, self.position);
		}

		self.at_line_start = false;
	}

	fn apply_indentation(&mut self, column: usize, alternate_column: usize, at: usize) {
		let current = *self
			.indent_stack
			.last()
			.expect("indentation stack always contains zero");

		match column.cmp(&current.column) {
			Ordering::Equal => {
				if alternate_column != current.alternate_column {
					self.report_ambiguous_indentation(at);
				}
			}
			Ordering::Greater => {
				if alternate_column <= current.alternate_column {
					self.report_ambiguous_indentation(at);
				}
				self.indent_stack.push(IndentLevel {
					column,
					alternate_column,
				});
				self.emit(TokenKind::Indent, Span::empty(at));
			}
			Ordering::Less => self.apply_dedent(column, alternate_column, at),
		}
	}

	fn apply_dedent(&mut self, column: usize, alternate_column: usize, at: usize) {
		if let Some(target_index) = self
			.indent_stack
			.iter()
			.rposition(|level| level.column == column)
		{
			while self.indent_stack.len() - 1 > target_index {
				self.indent_stack.pop();
				self.emit(TokenKind::Dedent, Span::empty(at));
			}

			let target = *self
				.indent_stack
				.last()
				.expect("matched indentation level must exist");
			if alternate_column != target.alternate_column {
				self.report_ambiguous_indentation(at);
			}
			return;
		}

		self.diagnostics.push(Diagnostic::error(
			"VLEX001",
			"dedent does not match any outer indentation level",
			Span::empty(at),
		));

		while self.indent_stack.len() > 1
			&& column
				< self
					.indent_stack
					.last()
					.expect("indentation stack is non-empty")
					.column
		{
			self.indent_stack.pop();
			self.emit(TokenKind::Dedent, Span::empty(at));
		}

		let recovery = *self
			.indent_stack
			.last()
			.expect("indentation stack always contains zero");
		if column > recovery.column {
			if alternate_column <= recovery.alternate_column {
				self.report_ambiguous_indentation(at);
			}
			self.indent_stack.push(IndentLevel {
				column,
				alternate_column,
			});
			self.emit(TokenKind::Indent, Span::empty(at));
		} else if alternate_column != recovery.alternate_column {
			self.report_ambiguous_indentation(at);
		}
	}

	fn report_ambiguous_indentation(&mut self, at: usize) {
		self.diagnostics.push(Diagnostic::error(
			"VLEX002",
			"tabs and spaces produce ambiguous indentation",
			Span::empty(at),
		));
	}

	fn lex_whitespace(&mut self) {
		let start = self.position;
		while matches!(self.peek_byte(), Some(b' ' | b'\t')) {
			self.position += 1;
		}
		self.emit(TokenKind::Whitespace, Span::new(start, self.position));
	}

	fn lex_comment(&mut self) {
		let start = self.position;
		while self.position < self.source.len() && !self.is_line_break_start() {
			self.bump_char();
		}
		self.emit(TokenKind::Comment, Span::new(start, self.position));
	}

	fn lex_line_break(&mut self) {
		let start = self.position;
		if self.peek_byte() == Some(b'\r') && self.peek_byte_at(self.position + 1) == Some(b'\n') {
			self.position += 2;
		} else {
			self.position += 1;
		}

		let structural = self.delimiters.is_empty() && self.logical_line_has_code;
		let kind = if structural {
			TokenKind::Newline
		} else {
			TokenKind::LineBreak
		};
		self.emit(kind, Span::new(start, self.position));

		if structural {
			self.logical_line_has_code = false;
		}
		self.at_line_start = true;
	}

	fn lex_identifier_or_keyword(&mut self) {
		let start = self.position;
		self.bump_char();
		while self.peek_char().is_some_and(is_identifier_continue) {
			self.bump_char();
		}

		let text = &self.source[start..self.position];
		let kind = keyword(text).map_or(TokenKind::Identifier, TokenKind::Keyword);
		self.emit_code(kind, Span::new(start, self.position));
	}

	fn lex_number(&mut self) {
		let start = self.position;
		self.consume_ascii_digits_and_underscores();
		let mut kind = TokenKind::Integer;

		if self.peek_byte() == Some(b'.')
			&& self
				.peek_byte_at(self.position + 1)
				.is_some_and(|byte| byte.is_ascii_digit())
		{
			kind = TokenKind::Float;
			self.position += 1;
			self.consume_ascii_digits_and_underscores();
		}

		if matches!(self.peek_byte(), Some(b'e' | b'E')) {
			let exponent_start = self.position;
			self.position += 1;
			if matches!(self.peek_byte(), Some(b'+' | b'-')) {
				self.position += 1;
			}
			if self.peek_byte().is_some_and(|byte| byte.is_ascii_digit()) {
				kind = TokenKind::Float;
				self.consume_ascii_digits_and_underscores();
			} else {
				self.position = exponent_start;
			}
		}

		self.emit_code(kind, Span::new(start, self.position));
	}

	fn consume_ascii_digits_and_underscores(&mut self) {
		while self
			.peek_byte()
			.is_some_and(|byte| byte.is_ascii_digit() || byte == b'_')
		{
			self.position += 1;
		}
	}

	fn lex_string(&mut self) {
		let start = self.position;
		let quote = self
			.peek_char()
			.expect("lex_string is called only at a quote");
		self.bump_char();
		let mut terminated = false;

		while self.position < self.source.len() {
			if self.is_line_break_start() {
				break;
			}

			let Some(character) = self.peek_char() else {
				break;
			};

			if character == quote {
				self.bump_char();
				terminated = true;
				break;
			}

			if character == '\\' {
				self.bump_char();
				if self.position < self.source.len() && !self.is_line_break_start() {
					self.bump_char();
				}
				continue;
			}

			self.bump_char();
		}

		let span = Span::new(start, self.position);
		self.emit_code(TokenKind::String, span);
		if !terminated {
			self.diagnostics.push(Diagnostic::error(
				"VLEX004",
				"unterminated string literal",
				span,
			));
		}
	}

	fn lex_punctuation_or_operator(&mut self) {
		for (text, kind) in [
			("&&", TokenKind::AndAnd),
			("||", TokenKind::OrOr),
			("==", TokenKind::EqualEqual),
			("!=", TokenKind::BangEqual),
			("<=", TokenKind::LessEqual),
			(">=", TokenKind::GreaterEqual),
			("->", TokenKind::Arrow),
			("+=", TokenKind::PlusEqual),
			("-=", TokenKind::MinusEqual),
			("*=", TokenKind::StarEqual),
			("/=", TokenKind::SlashEqual),
			("%=", TokenKind::PercentEqual),
		] {
			if self.source[self.position..].starts_with(text) {
				let start = self.position;
				self.position += text.len();
				self.emit_code(kind, Span::new(start, self.position));
				return;
			}
		}

		let start = self.position;
		let character = self
			.bump_char()
			.expect("punctuation lexing requires a character");
		let span = Span::new(start, self.position);

		let kind = match character {
			'(' => {
				self.delimiters.push(OpenDelimiter {
					kind: Delimiter::Paren,
					span,
				});
				TokenKind::LParen
			}
			')' => {
				self.close_delimiter(Delimiter::Paren, span);
				TokenKind::RParen
			}
			'[' => {
				self.delimiters.push(OpenDelimiter {
					kind: Delimiter::Bracket,
					span,
				});
				TokenKind::LBracket
			}
			']' => {
				self.close_delimiter(Delimiter::Bracket, span);
				TokenKind::RBracket
			}
			'{' => {
				self.delimiters.push(OpenDelimiter {
					kind: Delimiter::Brace,
					span,
				});
				TokenKind::LBrace
			}
			'}' => {
				self.close_delimiter(Delimiter::Brace, span);
				TokenKind::RBrace
			}
			':' => TokenKind::Colon,
			',' => TokenKind::Comma,
			'.' => TokenKind::Dot,
			'?' => TokenKind::Question,
			'+' => TokenKind::Plus,
			'-' => TokenKind::Minus,
			'*' => TokenKind::Star,
			'/' => TokenKind::Slash,
			'%' => TokenKind::Percent,
			'=' => TokenKind::Equal,
			'<' => TokenKind::Less,
			'>' => TokenKind::Greater,
			'!' => TokenKind::Bang,
			_ => {
				self.diagnostics.push(Diagnostic::error(
					"VLEX003",
					format!("unexpected character {character:?}"),
					span,
				));
				TokenKind::Unknown
			}
		};

		self.emit_code(kind, span);
	}

	fn close_delimiter(&mut self, expected: Delimiter, span: Span) {
		let Some(open) = self.delimiters.pop() else {
			self.diagnostics.push(Diagnostic::error(
				"VLEX005",
				format!("unexpected closing delimiter {}", expected.closing_text()),
				span,
			));
			return;
		};

		if open.kind != expected {
			self.diagnostics.push(Diagnostic::error(
				"VLEX005",
				format!(
					"mismatched closing delimiter {}; expected {}",
					expected.closing_text(),
					open.kind.closing_text()
				),
				span,
			));
		}
	}

	fn finish(&mut self) {
		if self.logical_line_has_code && self.delimiters.is_empty() {
			self.emit(TokenKind::Newline, Span::empty(self.position));
			self.logical_line_has_code = false;
		}

		while self.indent_stack.len() > 1 {
			self.indent_stack.pop();
			self.emit(TokenKind::Dedent, Span::empty(self.position));
		}

		let unclosed = std::mem::take(&mut self.delimiters);
		for open in unclosed {
			self.diagnostics.push(Diagnostic::error(
				"VLEX006",
				format!("unclosed delimiter; expected {}", open.kind.closing_text()),
				open.span,
			));
		}

		self.emit(TokenKind::Eof, Span::empty(self.position));
	}

	fn emit(&mut self, kind: TokenKind, span: Span) {
		self.tokens.push(Token::new(kind, span));
	}

	fn emit_code(&mut self, kind: TokenKind, span: Span) {
		self.logical_line_has_code = true;
		self.emit(kind, span);
	}

	fn is_line_break_start(&self) -> bool {
		matches!(self.peek_byte(), Some(b'\n' | b'\r'))
	}

	fn peek_byte(&self) -> Option<u8> {
		self.peek_byte_at(self.position)
	}

	fn peek_byte_at(&self, position: usize) -> Option<u8> {
		self.source.as_bytes().get(position).copied()
	}

	fn peek_char(&self) -> Option<char> {
		self.source[self.position..].chars().next()
	}

	fn bump_char(&mut self) -> Option<char> {
		let character = self.peek_char()?;
		self.position += character.len_utf8();
		Some(character)
	}
}

fn is_identifier_start(character: char) -> bool {
	character == '_' || character.is_alphabetic()
}

fn is_identifier_continue(character: char) -> bool {
	character == '_' || character.is_alphanumeric()
}

fn keyword(text: &str) -> Option<Keyword> {
	Some(match text {
		"and" => Keyword::And,
		"as" => Keyword::As,
		"async" => Keyword::Async,
		"await" => Keyword::Await,
		"cleanup" => Keyword::Cleanup,
		"component" => Keyword::Component,
		"const" => Keyword::Const,
		"else" => Keyword::Else,
		"enum" => Keyword::Enum,
		"export" => Keyword::Export,
		"extern" => Keyword::Extern,
		"false" => Keyword::False,
		"fn" => Keyword::Fn,
		"for" => Keyword::For,
		"from" => Keyword::From,
		"if" => Keyword::If,
		"import" => Keyword::Import,
		"in" => Keyword::In,
		"init" => Keyword::Init,
		"key" => Keyword::Key,
		"match" => Keyword::Match,
		"mount" => Keyword::Mount,
		"none" => Keyword::None,
		"not" => Keyword::Not,
		"or" => Keyword::Or,
		"param" => Keyword::Param,
		"return" => Keyword::Return,
		"shared" => Keyword::Shared,
		"slot" => Keyword::Slot,
		"state" => Keyword::State,
		"struct" => Keyword::Struct,
		"true" => Keyword::True,
		_ => return None,
	})
}

#[cfg(test)]
mod tests {
	use super::*;

	fn count_kind(tokens: &[Token], kind: TokenKind) -> usize {
		tokens.iter().filter(|token| token.kind == kind).count()
	}

	fn has_diagnostic(result: &Lexed, code: &str) -> bool {
		result
			.diagnostics
			.iter()
			.any(|diagnostic| diagnostic.code == code)
	}

	#[test]
	fn emits_indent_and_dedent_for_nested_blocks() {
		let result = lex(
			"if ready:\n\tstate x = 1\n\tif x:\n\t\tx += 1\n\treturn x\n",
		);

		assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
		assert_eq!(count_kind(&result.tokens, TokenKind::Indent), 2);
		assert_eq!(count_kind(&result.tokens, TokenKind::Dedent), 2);
		assert_eq!(count_kind(&result.tokens, TokenKind::Newline), 5);
	}

	#[test]
	fn blank_and_comment_only_lines_do_not_change_indentation() {
		let result = lex("if ready:\n\t# comment\n\n\tstate x = 1\n");

		assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
		assert_eq!(count_kind(&result.tokens, TokenKind::Indent), 1);
		assert_eq!(count_kind(&result.tokens, TokenKind::Dedent), 1);
		assert_eq!(count_kind(&result.tokens, TokenKind::Newline), 2);
		assert_eq!(count_kind(&result.tokens, TokenKind::LineBreak), 2);
		assert_eq!(count_kind(&result.tokens, TokenKind::Comment), 1);
	}

	#[test]
	fn bracket_continuation_has_no_structural_indent_tokens() {
		let result = lex("UserCard(\n\tname=user.name,\n\tdisabled=true\n)\n");

		assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
		assert_eq!(count_kind(&result.tokens, TokenKind::Indent), 0);
		assert_eq!(count_kind(&result.tokens, TokenKind::Dedent), 0);
		assert_eq!(count_kind(&result.tokens, TokenKind::LineBreak), 3);
		assert_eq!(count_kind(&result.tokens, TokenKind::Newline), 1);
	}

	#[test]
	fn eof_emits_synthetic_newline_and_remaining_dedent() {
		let result = lex("if ready:\n\tstate x = 1");

		assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
		assert_eq!(count_kind(&result.tokens, TokenKind::Indent), 1);
		assert_eq!(count_kind(&result.tokens, TokenKind::Dedent), 1);
		assert_eq!(count_kind(&result.tokens, TokenKind::Newline), 2);
		assert_eq!(result.tokens.last().map(|token| token.kind), Some(TokenKind::Eof));
	}

	#[test]
	fn reports_ambiguous_tabs_and_spaces() {
		let result = lex("if ready:\n\tstate a = 1\n        state b = 2\n");

		assert!(has_diagnostic(&result, "VLEX002"));
	}

	#[test]
	fn reports_dedent_to_unknown_level() {
		let result = lex("if a:\n\tif b:\n\t\tstate x = 1\n    state y = 2\n");

		assert!(has_diagnostic(&result, "VLEX001"));
	}

	#[test]
	fn preserves_comments_as_tokens() {
		let source = "state x = 1 # keep me\n";
		let result = lex(source);
		let comment = result
			.tokens
			.iter()
			.find(|token| token.kind == TokenKind::Comment)
			.expect("comment token");

		assert_eq!(&source[comment.span.start..comment.span.end], "# keep me");
	}

	#[test]
	fn reports_unclosed_delimiter() {
		let result = lex("state value = (\n\t1\n");

		assert!(has_diagnostic(&result, "VLEX006"));
	}
}
