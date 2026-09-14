#![forbid(unsafe_code)]

mod diagnostic;
mod lexer;
mod span;
mod token;

pub use diagnostic::{Diagnostic, Severity};
pub use lexer::{Lexed, lex};
pub use span::Span;
pub use token::{Keyword, Token, TokenKind};
