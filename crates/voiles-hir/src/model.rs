use voiles_lexer::{Diagnostic, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScopeKind {
	Module,
	Function,
	Component,
	Block,
	Loop,
	MatchArm,
	LifecycleInit,
	LifecycleMount,
	LifecycleCleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
	Import,
	Const,
	State,
	Shared,
	Function,
	Component,
	Struct,
	Enum,
	Parameter,
	RouteParam,
	LoopBinding,
	PatternBinding,
}

impl SymbolKind {
	#[must_use]
	pub const fn is_mutable(self) -> bool {
		matches!(self, Self::State | Self::Shared)
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scope {
	pub id: ScopeId,
	pub parent: Option<ScopeId>,
	pub kind: ScopeKind,
	pub owner_symbol: Option<SymbolId>,
	pub symbols: Vec<SymbolId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
	pub id: SymbolId,
	pub name: String,
	pub kind: SymbolKind,
	pub span: Span,
	pub scope: ScopeId,
}

impl Symbol {
	#[must_use]
	pub const fn is_mutable(&self) -> bool {
		self.kind.is_mutable()
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
	pub name: String,
	pub span: Span,
	pub scope: ScopeId,
	pub resolved: Option<SymbolId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Capture {
	pub function: SymbolId,
	pub symbol: SymbolId,
	pub first_reference: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirModule {
	pub root_scope: ScopeId,
	pub scopes: Vec<Scope>,
	pub symbols: Vec<Symbol>,
	pub references: Vec<Reference>,
	pub captures: Vec<Capture>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirResult {
	pub module: HirModule,
	pub diagnostics: Vec<Diagnostic>,
}
