#![forbid(unsafe_code)]

pub mod ast;
mod kind;
#[path = "parser/mod.rs"]
mod parser;
mod tree;

pub use kind::SyntaxKind;
pub use parser::{Parse, parse};
pub use tree::{SyntaxElement, SyntaxNode};
