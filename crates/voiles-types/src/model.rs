use voiles_hir::{BuiltinType, GlobalSymbolId, ModuleId, SymbolId};
use voiles_lexer::Span;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
	Unknown,
	Error,
	None,
	Builtin(BuiltinType, Vec<Type>),
	Named(GlobalSymbolId, Vec<Type>),
	Generic(String),
	Function(FunctionSignature),
}

impl Type {
	#[must_use]
	pub const fn builtin(kind: BuiltinType) -> Self {
		Self::Builtin(kind, Vec::new())
	}

	#[must_use]
	pub fn option(inner: Self) -> Self {
		Self::Builtin(BuiltinType::Option, vec![inner])
	}

	#[must_use]
	pub const fn is_unknown_or_error(&self) -> bool {
		matches!(self, Self::Unknown | Self::Error)
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionParameter {
	pub name: String,
	pub symbol: Option<SymbolId>,
	pub ty: Type,
	pub has_default: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionSignature {
	pub parameters: Vec<FunctionParameter>,
	pub return_type: Box<Type>,
	pub is_async: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedModule {
	pub module: ModuleId,
	pub symbol_types: Vec<Type>,
}

impl TypedModule {
	#[must_use]
	pub fn symbol_type(&self, symbol: SymbolId) -> Option<&Type> {
		self.symbol_types.get(symbol.0)
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeDiagnostic {
	pub module: ModuleId,
	pub code: &'static str,
	pub message: String,
	pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypeCheckResult {
	pub modules: Vec<TypedModule>,
	pub diagnostics: Vec<TypeDiagnostic>,
}

impl TypeCheckResult {
	#[must_use]
	pub fn type_of(&self, symbol: GlobalSymbolId) -> Option<&Type> {
		self.modules
			.get(symbol.module.0)?
			.symbol_types
			.get(symbol.symbol.0)
	}
}
