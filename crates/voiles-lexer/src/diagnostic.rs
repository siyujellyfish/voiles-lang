use crate::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
	Error,
	Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
	pub code: &'static str,
	pub severity: Severity,
	pub message: String,
	pub span: Span,
}

impl Diagnostic {
	#[must_use]
	pub fn error(code: &'static str, message: impl Into<String>, span: Span) -> Self {
		Self {
			code,
			severity: Severity::Error,
			message: message.into(),
			span,
		}
	}
}
