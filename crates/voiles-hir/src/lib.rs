#![forbid(unsafe_code)]

mod model;
mod project;
mod resolver;

pub use model::{
	BuiltinType, Capture, DependencyEdge, DependencyKind, ExportedSymbol, GlobalSymbolId,
	HirModule, HirProject, HirProjectResult, HirResult, ImportBinding, ImportBindingKind, ModuleId,
	ModuleSource, ModuleSourceId, ProjectDiagnostic, ProjectModule, Reference, Scope, ScopeId,
	ScopeKind, Symbol, SymbolId, SymbolKind, TypeReference, TypeTarget,
};
pub use project::resolve_project;

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
