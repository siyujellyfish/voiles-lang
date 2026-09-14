use voiles_lexer::{Diagnostic, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModuleId(pub usize);

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ModuleSourceId(pub String);

impl ModuleSourceId {
	#[must_use]
	pub fn new(path: impl Into<String>) -> Self {
		Self(path.into())
	}

	#[must_use]
	pub fn as_str(&self) -> &str {
		&self.0
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalSymbolId {
	pub module: ModuleId,
	pub symbol: SymbolId,
}

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

	#[must_use]
	pub const fn is_type(self) -> bool {
		matches!(self, Self::Struct | Self::Enum)
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleSource {
	pub id: ModuleSourceId,
	pub source: String,
}

impl ModuleSource {
	#[must_use]
	pub fn new(id: impl Into<String>, source: impl Into<String>) -> Self {
		Self {
			id: ModuleSourceId::new(id),
			source: source.into(),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportedSymbol {
	pub name: String,
	pub target: GlobalSymbolId,
	pub kind: SymbolKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImportBindingKind {
	Named,
	Namespace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportBinding {
	pub module: ModuleId,
	pub local_symbol: SymbolId,
	pub kind: ImportBindingKind,
	pub imported_name: Option<String>,
	pub specifier: String,
	pub declaration_span: Span,
	pub target_module: Option<ModuleId>,
	pub target_symbol: Option<GlobalSymbolId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DependencyKind {
	Runtime,
	TypeOnly,
	External,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyEdge {
	pub from: ModuleId,
	pub to: Option<ModuleId>,
	pub kind: DependencyKind,
	pub specifier: String,
	pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuiltinType {
	Void,
	Bool,
	Int,
	Float,
	String,
	List,
	Map,
	Option,
	Result,
	TrustedHtml,
	Url,
	JsValue,
}

impl BuiltinType {
	#[must_use]
	pub fn from_name(name: &str) -> Option<Self> {
		match name {
			"Void" => Some(Self::Void),
			"Bool" => Some(Self::Bool),
			"Int" => Some(Self::Int),
			"Float" => Some(Self::Float),
			"String" => Some(Self::String),
			"List" => Some(Self::List),
			"Map" => Some(Self::Map),
			"Option" => Some(Self::Option),
			"Result" => Some(Self::Result),
			"TrustedHtml" => Some(Self::TrustedHtml),
			"Url" => Some(Self::Url),
			"JsValue" => Some(Self::JsValue),
			_ => None,
		}
	}

	#[must_use]
	pub const fn generic_arity(self) -> usize {
		match self {
			Self::List | Self::Option => 1,
			Self::Map | Self::Result => 2,
			_ => 0,
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeTarget {
	Builtin(BuiltinType),
	Symbol(GlobalSymbolId),
	Generic(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeReference {
	pub module: ModuleId,
	pub path: Vec<String>,
	pub span: Span,
	pub target: Option<TypeTarget>,
	pub through_import: Option<SymbolId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectModule {
	pub id: ModuleId,
	pub source_id: ModuleSourceId,
	pub source: String,
	pub hir: HirModule,
	pub exports: Vec<ExportedSymbol>,
	pub imports: Vec<ImportBinding>,
	pub type_references: Vec<TypeReference>,
}

impl ProjectModule {
	#[must_use]
	pub fn export(&self, name: &str) -> Option<&ExportedSymbol> {
		self.exports.iter().find(|export| export.name == name)
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HirProject {
	pub modules: Vec<ProjectModule>,
	pub dependencies: Vec<DependencyEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectDiagnostic {
	pub source: ModuleSourceId,
	pub diagnostic: Diagnostic,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HirProjectResult {
	pub project: HirProject,
	pub diagnostics: Vec<ProjectDiagnostic>,
}
