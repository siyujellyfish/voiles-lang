#![forbid(unsafe_code)]

mod model;
mod resolver;

pub use model::{
	Capture, HirModule, HirResult, Reference, Scope, ScopeId, ScopeKind, Symbol, SymbolId, SymbolKind,
};

use voiles_syntax::Parse;

#[must_use]
pub fn resolve_module(source: &str) -> HirResult {
	let parsed = voiles_syntax::parse(source);
	resolver::resolve(source, &parsed)
}

#[must_use]
pub fn resolve_parsed_module(source: &str, parsed: &Parse) -> HirResult {
	resolver::resolve(source, parsed)
}
