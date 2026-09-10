# Voiles Component Model

Status: `Draft` with accepted core semantics.

This document defines the current component declaration, automatic export, instance scope and compiler reachability model.

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

The compiler does not infer whether an arbitrary `.voil` module is a component from its render statements.

`fn` declares callable logic; `component` declares instantiable UI.

## 2. Automatic component export

Status: `Accepted`.

Every top-level `component Name(...):` declaration is automatically part of the module's component export surface. No explicit `export` keyword is required.

A `.voil` module may contain multiple component declarations:

```voil
component UserBtn(text: String):
	std.button(text)

component IconBtn(icon: String):
	std.button(icon)
```

Both components are automatically exportable. Their names must be unique within the module.

This is intentionally called automatic or implicit component export rather than JavaScript-style `default export`: JavaScript permits only one default export per module, while Voiles allows multiple automatically exportable component declarations.

A single component may be imported by declaration name:

```voil
import UserBtn from "./buttons.voil"
```

Multiple components from the same `.voil` module use a comma-separated import list:

```voil
import UserBtn, IconBtn from "./buttons.voil"
```

Each imported name resolves the automatically exported component declaration with the same name.

Component import alias syntax remains Draft. The current recommendation is an item-local `as` form:

```voil
import UserBtn as PrimaryBtn, IconBtn as SmallIconBtn from "./buttons.voil"
```

Mixed aliased and non-aliased items would therefore be valid if this alias form is accepted:

```voil
import UserBtn, IconBtn as SmallIconBtn from "./buttons.voil"
```

Non-component module/default/named/namespace import semantics remain a separate module-system decision.

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

Whether module-level mutable `state` should be permitted in a module that also declares components remains open. Component-local mutable state should normally be declared inside the component block so instance isolation is explicit.

## 6. Component export and module API

Automatic component export does not imply that every module-level binding becomes public.

The exact export/access model for helper functions, types and bindings is still open.

For component imports, declaration-name resolution is accepted:

```voil
# UserBtn.voil
component UserBtn(text: String):
	...
```

```voil
import UserBtn from "./UserBtn.voil"
```

No `export component` syntax is required.

For a multi-component module:

```voil
# buttons.voil
component UserBtn(...):
	...

component IconBtn(...):
	...
```

both declarations are public component symbols and may be imported together:

```voil
import UserBtn, IconBtn from "./buttons.voil"
```

The recommended, but not yet accepted, alias form is:

```voil
import UserBtn as PrimaryBtn, IconBtn as CompactBtn from "./buttons.voil"
```

## 7. Unused component elimination

Status: `Accepted` compiler goal.

A component that is unreachable from application entry routes and reachable component/module references may be removed from production output.

Conceptually:

```text
route entrypoints
	↓
reachable imports/components
	↓
component dependency graph
	↓
codegen set
```

An imported component that is never invoked can also be removed when removing it preserves module semantics.

Component-local initialization occurs only when the component is instantiated. Therefore an unused component declaration must not create component state, lifecycle work or render output merely because its containing module exists.

However the compiler must not blindly remove an entire module only because all of its component declarations are unused. If module-top-level initialization has observable side effects, those effects must be preserved unless effect analysis proves the module itself removable.

Therefore dead-code elimination is semantics-preserving, not merely name-based.

Long-term direction:

- prefer analyzable or effect-free top-level initialization;
- remove unreachable component bodies;
- remove unreachable helper functions/types/constants when safe;
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

componentImportDecl := "import" componentImportItem ("," componentImportItem)* "from" stringLiteral
componentImportItem := PascalIdentifier componentAlias?
componentAlias      := "as" PascalIdentifier
```

`sharedDecl` is excluded from `componentItem` because `shared` is module-top-level only.

The module symbol table must reject duplicate component declaration names.

The comma-separated component import list is Accepted. `componentAlias` remains Draft until the alias form is explicitly accepted.

## 9. Remaining component decisions

- component children / slot model;
- callback/event parameter typing;
- named-argument separator finalization;
- component import alias syntax finalization (`as` currently recommended);
- mount/unmount and cleanup lifecycle;
- whether module-level mutable `state` is legal in modules that also declare components;
- helper function/type export rules;
- exact reachability/effect model used by tree-shaking.