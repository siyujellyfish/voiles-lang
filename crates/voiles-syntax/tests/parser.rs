use voiles_syntax::{SyntaxKind, parse};

#[test]
fn preserves_accepted_expression_precedence() {
	let source = "state value = a + b * 2 and c == d or false\n";
	let parsed = parse(source);

	assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
	let spans = parsed.root.descendant_spans(SyntaxKind::BinaryExpr);
	let texts: Vec<_> = spans
		.iter()
		.map(|span| source[span.start..span.end].trim())
		.collect();
	assert!(texts.contains(&"b * 2"));
	assert!(texts.contains(&"a + b * 2"));
	assert!(texts.iter().any(|text| text.contains("and c == d")));
	assert!(texts.iter().any(|text| text.ends_with("or false")));
}

#[test]
fn parses_typed_route_parameter_declaration() {
	let source = "param id: Int\n";
	let parsed = parse(source);

	assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
	assert_eq!(parsed.root.descendant_count(SyntaxKind::ParamDecl), 1);
	assert_eq!(parsed.root.source_text(source), source);
}

#[test]
fn recovers_from_incomplete_block_without_losing_source() {
	let source = "if ready:\n";
	let parsed = parse(source);

	assert!(!parsed.diagnostics.is_empty());
	assert_eq!(parsed.root.descendant_count(SyntaxKind::IfStmt), 1);
	assert_eq!(parsed.root.source_text(source), source);
}

#[test]
fn keeps_lexical_indentation_error_and_following_source() {
	let source = "if ready:\n\tstate a = 1\n    state b = 2\nstate c = 3\n";
	let parsed = parse(source);

	assert!(
		parsed
			.diagnostics
			.iter()
			.any(|diagnostic| diagnostic.code == "VLEX001")
	);
	assert_eq!(parsed.root.source_text(source), source);
	assert!(parsed.root.descendant_count(SyntaxKind::BindingDecl) >= 2);
}
