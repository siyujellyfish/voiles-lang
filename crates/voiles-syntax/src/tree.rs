use voiles_lexer::{Span, Token};

use crate::SyntaxKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyntaxElement {
	Node(Box<SyntaxNode>),
	Token(Token),
}

impl SyntaxElement {
	#[must_use]
	pub fn span(&self) -> Span {
		match self {
			Self::Node(node) => node.span,
			Self::Token(token) => token.span,
		}
	}
}

impl From<SyntaxNode> for SyntaxElement {
	fn from(node: SyntaxNode) -> Self {
		Self::Node(Box::new(node))
	}
}

impl From<Token> for SyntaxElement {
	fn from(token: Token) -> Self {
		Self::Token(token)
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxNode {
	pub kind: SyntaxKind,
	pub span: Span,
	pub children: Vec<SyntaxElement>,
}

impl SyntaxNode {
	#[must_use]
	pub fn new(kind: SyntaxKind, children: Vec<SyntaxElement>) -> Self {
		let span = match (children.first(), children.last()) {
			(Some(first), Some(last)) => Span::new(first.span().start, last.span().end),
			_ => Span::empty(0),
		};
		Self {
			kind,
			span,
			children,
		}
	}

	#[must_use]
	pub fn source_text(&self, source: &str) -> String {
		let mut text = String::with_capacity(self.span.end.saturating_sub(self.span.start));
		self.push_source_text(source, &mut text);
		text
	}

	fn push_source_text(&self, source: &str, output: &mut String) {
		for child in &self.children {
			match child {
				SyntaxElement::Node(node) => node.push_source_text(source, output),
				SyntaxElement::Token(token) if token.span.start < token.span.end => {
					output.push_str(&source[token.span.start..token.span.end]);
				}
				SyntaxElement::Token(_) => {}
			}
		}
	}

	#[must_use]
	pub fn descendant_count(&self, kind: SyntaxKind) -> usize {
		let own = usize::from(self.kind == kind);
		own + self
			.children
			.iter()
			.map(|child| match child {
				SyntaxElement::Node(node) => node.descendant_count(kind),
				SyntaxElement::Token(_) => 0,
			})
			.sum::<usize>()
	}

	#[must_use]
	pub fn descendant_spans(&self, kind: SyntaxKind) -> Vec<Span> {
		let mut spans = Vec::new();
		self.push_descendant_spans(kind, &mut spans);
		spans
	}

	fn push_descendant_spans(&self, kind: SyntaxKind, spans: &mut Vec<Span>) {
		if self.kind == kind {
			spans.push(self.span);
		}
		for child in &self.children {
			if let SyntaxElement::Node(node) = child {
				node.push_descendant_spans(kind, spans);
			}
		}
	}
}
