#![forbid(unsafe_code)]

mod checker;
mod model;

pub use checker::check_project;
pub use model::{
	FunctionParameter, FunctionSignature, Type, TypeCheckResult, TypeDiagnostic, TypedModule,
};
