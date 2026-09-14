use voiles_hir::{BuiltinType, ModuleSource, resolve_project};
use voiles_types::{Type, check_project};

fn type_diagnostic(result: &voiles_types::TypeCheckResult, code: &str) -> bool {
	result.diagnostics.iter().any(|diagnostic| diagnostic.code == code)
}

#[test]
fn infers_primitive_binding_types_and_checks_assignment() {
	let hir = resolve_project(vec![ModuleSource::new(
		"main.voil",
		"state count = 0\ncount = 1\ncount = \"wrong\"\n",
	)]);
	assert!(hir.diagnostics.is_empty(), "{:?}", hir.diagnostics);

	let checked = check_project(&hir.project);
	let module = &hir.project.modules[0];
	let count = module
		.hir
		.symbols
		.iter()
		.find(|symbol| symbol.name == "count")
		.expect("count symbol");
	assert_eq!(
		checked.modules[0].symbol_type(count.id),
		Some(&Type::builtin(BuiltinType::Int))
	);
	assert!(type_diagnostic(&checked, "VTYPE001"));
}

#[test]
fn checks_function_defaults_named_arguments_and_return_types() {
	let hir = resolve_project(vec![ModuleSource::new(
		"main.voil",
		"fn add(left: Int, right: Int = 1) -> Int:\n\treturn left + right\n\nconst good: Int = add(2, right=3)\nconst bad: String = add(2)\n",
	)]);
	assert!(hir.diagnostics.is_empty(), "{:?}", hir.diagnostics);

	let checked = check_project(&hir.project);
	assert_eq!(
		checked
			.diagnostics
			.iter()
			.filter(|diagnostic| diagnostic.code == "VTYPE001")
			.count(),
		1
	);
	assert!(!type_diagnostic(&checked, "VTYPE002"));
	assert!(!type_diagnostic(&checked, "VTYPE003"));
}

#[test]
fn propagates_exported_function_signatures_to_import_bindings() {
	let hir = resolve_project(vec![
		ModuleSource::new(
			"lib.voil",
			"export fn greet(name: String) -> String:\n\treturn name\n",
		),
		ModuleSource::new(
			"main.voil",
			"import greet from \"./lib.voil\"\nconst message: String = greet(\"Ada\")\nconst wrong: String = greet(1)\n",
		),
	]);
	assert!(hir.diagnostics.is_empty(), "{:?}", hir.diagnostics);

	let checked = check_project(&hir.project);
	let import = hir.project.modules[1]
		.imports
		.first()
		.expect("greet import");
	assert!(matches!(
		checked.modules[1].symbol_type(import.local_symbol),
		Some(Type::Function(_))
	));
	assert!(type_diagnostic(&checked, "VTYPE001"));
}

#[test]
fn accepts_none_for_optional_type_and_checks_builtin_generic_arity() {
	let valid_hir = resolve_project(vec![ModuleSource::new(
		"valid.voil",
		"state name: String? = none\n",
	)]);
	assert!(valid_hir.diagnostics.is_empty(), "{:?}", valid_hir.diagnostics);
	let valid = check_project(&valid_hir.project);
	assert!(valid.diagnostics.is_empty(), "{:?}", valid.diagnostics);

	let invalid_hir = resolve_project(vec![ModuleSource::new(
		"invalid.voil",
		"state names: List = none\n",
	)]);
	assert!(invalid_hir.diagnostics.is_empty(), "{:?}", invalid_hir.diagnostics);
	let invalid = check_project(&invalid_hir.project);
	assert!(type_diagnostic(&invalid, "VTYPE006"));
}

#[test]
fn reports_return_type_and_required_argument_errors() {
	let hir = resolve_project(vec![ModuleSource::new(
		"main.voil",
		"fn title(value: String) -> String:\n\treturn 1\n\ntitle()\n",
	)]);
	assert!(hir.diagnostics.is_empty(), "{:?}", hir.diagnostics);

	let checked = check_project(&hir.project);
	assert!(type_diagnostic(&checked, "VTYPE007"));
	assert!(type_diagnostic(&checked, "VTYPE002"));
}

#[test]
fn requires_bool_conditions_and_string_or_int_keys() {
	let hir = resolve_project(vec![ModuleSource::new(
		"main.voil",
		"state items: List<Int> = none\nif 1:\n\tconst value = 1\nfor item in items key true:\n\tconst copy = item\n",
	)]);
	assert!(hir.diagnostics.is_empty(), "{:?}", hir.diagnostics);

	let checked = check_project(&hir.project);
	assert!(type_diagnostic(&checked, "VTYPE001"));
	assert!(type_diagnostic(&checked, "VTYPE008"));
}
