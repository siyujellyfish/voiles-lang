# Voiles Component Model

Status: `Draft` with accepted core semantics.

This document defines the current component declaration, default export, instance scope and compiler reachability model.

## 1. Explicit component declaration

Status: `Accepted`.

Reusable UI is declared explicitly with `component`.

```voil
component UserBtn(
	text: String,
	disabled: Bool = false
):
	state count = 0

	fn click():
		count += 1

	container:
		std.button(
			disabled=disabled,
			onclick=click
		)
		std.p(text)
		std.p(count)
```

The compiler does not need to infer whether an arbitrary `.voil` module is a component from its render statements.

`fn` declares callable logic; `component` declares instantiable UI.

## 2. One default component per file

Status: `Accepted` for v0.1.

A `.voil` module may declare at most one top-level `component` in v0.1.

That component is automatically the module's default component export. No explicit `export` keyword is required.

Example file:

```text
ui/UserBtn.voil
```

```voil
import std from "@voiles/html-base"

component UserBtn(text: String):
	std.button(text)
```

Consumer:

```voil
import UserBtn from "../ui/UserBtn.voil"

main:
	UserBtn(text="Click")
```

The imported identifier is the local alias for the module's default component export.

The component name should normally match the filename for readability and diagnostics, but filename matching is not currently a parser requirement.

Named component exports and multiple top-level component declarations are deferred until a concrete requirement exists.

## 3. Component parameters are the public input surface

Status: `Accepted` baseline.

Component parameters are declared in the component header.

```voil
component UserBtn(
	text: String,
	disabled: Bool = false
):
	...
```

Semantics:

- `text` is required because it has no default value;
- `disabled` is optional because it has a default value;
- inputs are immutable inside the component unless a later explicit mechanism defines writable input semantics;
- component invocation uses named arguments with the current Draft `=` separator.

```voil
UserBtn(
	text="Delete",
	disabled=true
)
```

The exact named-argument grammar remains subject to the general call/struct grammar decision.

## 4. Component instance scope

Status: `Accepted`.

Every component invocation creates a new component instance scope.

```voil
UserBtn(text="A")
UserBtn(text="B")
```

If the component contains:

```voil
state count = 0
```

then each invocation owns an independent `count` storage cell.

```text
UserBtn A
	└─ count A

UserBtn B
	└─ count B
```

Ordinary `.voil` modules imported from inside a component instance also use that component instance as their importer scope, so ordinary `state` dependencies remain isolated per component instance.

Only module-top-level `shared` declarations intentionally cross component/module instance boundaries.

## 5. Scope inside and outside the component block

The `component ...:` block is a lexical and instance scope.

```voil
shared theme = "dark"
const module_name = "button"

component UserBtn(text: String):
	const label = text
	state count = 0
	...
```

Current interpretation:

- `shared` remains legal only at module top level and is application-global;
- `const` / `state` declared inside `component` belong to the component instance scope;
- ordinary module-level declarations outside `component` retain module-level lexical semantics.

Whether module-level mutable `state` should be permitted in a module that also declares a component remains open. Component-local mutable state should normally be declared inside the component block so instance isolation is explicit.

## 6. Default export and module API

The default component export does not imply that every module-level binding becomes public.

The exact export/access model for helper functions, types and bindings is still open.

For v0.1 component imports, the minimum required behavior is:

```voil
import UserBtn from "./UserBtn.voil"
```

which imports the module's single default component.

No `export component` syntax is required.

## 7. Unused component elimination

Status: `Accepted` compiler goal.

A component that is unreachable from application entry routes and reachable component/module references may be removed from production output.

Conceptually:

```text
route entrypoints
	↓
reachable imports/components
	↓
codegen set
```

An imported component that is never invoked can also be removed when removing it preserves module semantics.

However the compiler must not blindly remove an entire module only because its default component is unused. If module-top-level initialization can have observable side effects, those effects must be preserved unless effect analysis proves the module itself removable.

Therefore dead-code elimination is semantics-preserving, not merely name-based.

Long-term direction:

- prefer analyzable or effect-free top-level initialization;
- remove unreachable component bodies;
- remove unreachable helper functions/types/constants;
- remove entire modules only when no observable top-level behavior remains.

## 8. Initial grammar sketch

Descriptive only:

```text
componentDecl     := "component" PascalIdentifier parameters componentBlock
componentBlock    := ":" NEWLINE INDENT componentItem* DEDENT

componentItem     := bindingDecl
                  | functionDecl
                  | controlFlow
                  | structuralBlock
                  | containerBlock
                  | standardHtmlCall
                  | componentCall

componentCall     := PascalIdentifier callArguments childBlock?
```

`sharedDecl` is excluded from `componentItem` because `shared` is module-top-level only.

## 9. Remaining component decisions

- component children / slot model;
- callback/event parameter typing;
- named-argument separator finalization;
- mount/unmount and cleanup lifecycle;
- whether module-level mutable `state` is legal in component modules;
- helper function/type export rules;
- exact reachability/effect model used by tree-shaking;
- whether named or multiple component exports are ever added after v0.1.
