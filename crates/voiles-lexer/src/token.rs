use crate::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Keyword {
	And,
	As,
	Async,
	Await,
	Cleanup,
	Component,
	Const,
	Else,
	Enum,
	Export,
	Extern,
	False,
	Fn,
	For,
	From,
	If,
	Import,
	In,
	Init,
	Key,
	Match,
	Mount,
	None,
	Not,
	Or,
	Param,
	Return,
	Shared,
	Slot,
	State,
	Struct,
	True,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
	Keyword(Keyword),
	Identifier,
	Integer,
	Float,
	String,
	Comment,
	Whitespace,
	LineBreak,
	Newline,
	Indent,
	Dedent,
	LParen,
	RParen,
	LBracket,
	RBracket,
	LBrace,
	RBrace,
	Colon,
	Comma,
	Dot,
	Question,
	Plus,
	Minus,
	Star,
	Slash,
	Percent,
	Equal,
	Less,
	Greater,
	Bang,
	AndAnd,
	OrOr,
	EqualEqual,
	BangEqual,
	LessEqual,
	GreaterEqual,
	Arrow,
	PlusEqual,
	MinusEqual,
	StarEqual,
	SlashEqual,
	PercentEqual,
	Unknown,
	Eof,
}

impl TokenKind {
	#[must_use]
	pub const fn is_trivia(self) -> bool {
		matches!(
			self,
			Self::Comment | Self::Whitespace | Self::LineBreak
		)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Token {
	pub kind: TokenKind,
	pub span: Span,
}

impl Token {
	#[must_use]
	pub const fn new(kind: TokenKind, span: Span) -> Self {
		Self { kind, span }
	}
}
