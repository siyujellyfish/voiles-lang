use crate::SyntaxKind;

use super::{Parse, parse};

fn has_diagnostic(parse: &Parse, code: &str) -> bool {
	parse
		.diagnostics
		.iter()
		.any(|diagnostic| diagnostic.code == code)
}

#[test]
fn cst_round_trips_source_with_trivia() {
	let source = "# lead\nimport A as B from \"./a.voil\" # keep\n\nstate count: Int = 1\n";
	let parsed = parse(source);

	assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
	assert_eq!(parsed.root.source_text(source), source);
	assert_eq!(parsed.root.descendant_count(SyntaxKind::ImportDecl), 1);
	assert_eq!(parsed.root.descendant_count(SyntaxKind::BindingDecl), 1);
}

#[test]
fn parses_function_parameters_defaults_and_return_type() {
	let source = "fn add(a: Int, b: Int = 2) -> Int:\n\treturn a + b * 2\n";
	let parsed = parse(source);

	assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
	assert_eq!(parsed.root.descendant_count(SyntaxKind::FunctionDecl), 1);
	assert_eq!(parsed.root.descendant_count(SyntaxKind::Parameter), 2);
	assert_eq!(parsed.root.descendant_count(SyntaxKind::BinaryExpr), 2);
	assert_eq!(parsed.root.source_text(source), source);
}

#[test]
fn parses_namespace_and_extern_imports() {
	let source = "import * as store from \"./store.voil\"\nextern import * as lib from \"pkg\"\n";
	let parsed = parse(source);

	assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
	assert_eq!(parsed.root.descendant_count(SyntaxKind::ImportDecl), 1);
	assert_eq!(
		parsed.root.descendant_count(SyntaxKind::ExternImportDecl),
		1
	);
	assert_eq!(parsed.root.descendant_count(SyntaxKind::NamespaceImport), 2);
}

#[test]
fn parses_if_for_key_and_lifecycle_blocks() {
	let source = "init:\n\tstate ready = true\nif ready:\n\tfor user in users key user.id:\n\t\tshow(user)\nelse:\n\tshow_empty()\n";
	let parsed = parse(source);

	assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
	assert_eq!(parsed.root.descendant_count(SyntaxKind::LifecycleBlock), 1);
	assert_eq!(parsed.root.descendant_count(SyntaxKind::IfStmt), 1);
	assert_eq!(parsed.root.descendant_count(SyntaxKind::ForStmt), 1);
	assert_eq!(parsed.root.descendant_count(SyntaxKind::KeyClause), 1);
	assert_eq!(parsed.root.source_text(source), source);
}

#[test]
fn parses_component_slots_and_child_blocks() {
	let source = "component Card(title: String):\n\tcontainer:\n\t\tstd.h2(title)\n\t\tslot\n\nCard(title=\"Profile\"):\n\tslot header:\n\t\tstd.h3(\"Header\")\n\tstd.p(\"Body\")\n";
	let parsed = parse(source);

	assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
	assert_eq!(parsed.root.descendant_count(SyntaxKind::ComponentDecl), 1);
	assert_eq!(parsed.root.descendant_count(SyntaxKind::SlotStmt), 2);
	assert_eq!(
		parsed.root.descendant_count(SyntaxKind::UiChildBlockStmt),
		2
	);
	assert_eq!(parsed.root.source_text(source), source);
}

#[test]
fn parses_struct_enum_and_match_patterns() {
	let source = "export struct User:\n\tid: Int\n\tname: String\n\tnickname: String? = none\n\nexport enum LoadState<T>:\n\tidle\n\tready(T)\n\tfailed(Error)\n\nfn render(state: LoadState<String>):\n\tmatch state:\n\t\tready(data):\n\t\t\tshow(data)\n\t\tfailed(error):\n\t\t\tshow(error)\n\t\t_:\n\t\t\tshow_empty()\n";
	let parsed = parse(source);

	assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
	assert_eq!(parsed.root.descendant_count(SyntaxKind::StructDecl), 1);
	assert_eq!(parsed.root.descendant_count(SyntaxKind::StructField), 3);
	assert_eq!(parsed.root.descendant_count(SyntaxKind::EnumDecl), 1);
	assert_eq!(parsed.root.descendant_count(SyntaxKind::EnumCase), 3);
	assert_eq!(parsed.root.descendant_count(SyntaxKind::MatchStmt), 1);
	assert_eq!(parsed.root.descendant_count(SyntaxKind::MatchArm), 3);
	assert_eq!(parsed.root.descendant_count(SyntaxKind::Pattern), 5);
	assert_eq!(parsed.root.source_text(source), source);
}

#[test]
fn parses_structural_block_attributes_as_ui_child_block() {
	let source = "section(id=\"profile\", class=\"panel\"):\n\tstd.h2(\"Profile\")\n";
	let parsed = parse(source);

	assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
	assert_eq!(
		parsed.root.descendant_count(SyntaxKind::UiChildBlockStmt),
		1
	);
	assert_eq!(parsed.root.descendant_count(SyntaxKind::NamedArgument), 2);
	assert_eq!(parsed.root.source_text(source), source);
}

#[test]
fn rejects_positional_argument_after_named_argument() {
	let parsed = parse("call(a=1, 2)\n");

	assert!(has_diagnostic(&parsed, "VPAR007"));
	assert_eq!(parsed.root.descendant_count(SyntaxKind::NamedArgument), 1);
	assert_eq!(
		parsed.root.descendant_count(SyntaxKind::PositionalArgument),
		1
	);
}

#[test]
fn recovers_after_malformed_statement() {
	let source = "state = 1\nstate good = 2\n";
	let parsed = parse(source);

	assert!(!parsed.diagnostics.is_empty());
	assert_eq!(parsed.root.descendant_count(SyntaxKind::BindingDecl), 2);
	assert_eq!(parsed.root.source_text(source), source);
}
