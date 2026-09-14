use voiles_hir::{SymbolKind, resolve_module};

fn has_diagnostic(result: &voiles_hir::HirResult, code: &str) -> bool {
	result
		.diagnostics
		.iter()
		.any(|diagnostic| diagnostic.code == code)
}

#[test]
fn resolves_nearest_lexical_binding() {
	let source = "state value = 1\nfn read() -> Int:\n\tstate value = 2\n\treturn value\n";
	let result = resolve_module(source);

	assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
	let inner = result
		.module
		.symbols
		.iter()
		.find(|symbol| symbol.name == "value" && symbol.scope != result.module.root_scope)
		.expect("inner value");
	let reference = result
		.module
		.references
		.iter()
		.find(|reference| reference.name == "value")
		.expect("value reference");
	assert_eq!(reference.resolved, Some(inner.id));
}

#[test]
fn reports_use_before_declaration() {
	let source = "fn read():\n\tconst before = later\n\tconst later = 1\n\treturn before\n";
	let result = resolve_module(source);

	assert!(has_diagnostic(&result, "VHIR002"));
	assert!(!has_diagnostic(&result, "VHIR003"));
}

#[test]
fn reports_same_scope_duplicate_declaration() {
	let result = resolve_module("state count = 0\nconst count = 1\n");

	assert!(has_diagnostic(&result, "VHIR001"));
}

#[test]
fn rejects_nested_shared_declaration() {
	let result = resolve_module("fn bad():\n\tshared count = 0\n");

	assert!(has_diagnostic(&result, "VHIR005"));
}

#[test]
fn rejects_assignment_to_immutable_binding() {
	let result = resolve_module("const title = \"A\"\ntitle = \"B\"\n");

	assert!(has_diagnostic(&result, "VHIR004"));
}

#[test]
fn records_nested_function_state_capture() {
	let source = "fn make():\n\tstate i = 0\n\tfn next():\n\t\ti += 1\n\treturn next\n";
	let result = resolve_module(source);

	assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
	assert_eq!(result.module.captures.len(), 1);
	let capture = result.module.captures[0];
	assert_eq!(result.module.symbols[capture.symbol.0].name, "i");
	assert_eq!(result.module.symbols[capture.function.0].name, "next");
}

#[test]
fn leaves_direct_ui_head_for_later_component_or_html_resolution() {
	let source = "import * as std from \"@voiles/html-base\"\nstate title = \"Profile\"\nsection(id=title):\n\tstd.p(title)\n";
	let result = resolve_module(source);

	assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
	assert!(
		!result
			.module
			.references
			.iter()
			.any(|reference| reference.name == "section")
	);
	assert_eq!(
		result
			.module
			.references
			.iter()
			.filter(|reference| reference.name == "title")
			.count(),
		2
	);
}

#[test]
fn validates_lifecycle_lexical_owner() {
	let result = resolve_module("fn bad():\n\tmount:\n\t\tconst value = 1\n");

	assert!(has_diagnostic(&result, "VHIR006"));
}

#[test]
fn component_parameter_defaults_are_left_to_right() {
	let source = "component Card(first: Int = second, second: Int = 2):\n\tstate local = first\n";
	let result = resolve_module(source);

	assert!(has_diagnostic(&result, "VHIR002"));
	assert!(
		result
			.module
			.symbols
			.iter()
			.any(|symbol| symbol.name == "Card" && symbol.kind == SymbolKind::Component)
	);
}
