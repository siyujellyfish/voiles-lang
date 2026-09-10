# Voiles Component Model

Status: `Draft` with accepted core semantics.

This document defines the current component declaration, automatic export, instance scope, parameter model, slot model and compiler reachability model.

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

Status: `Accepted`.

Component parameters are declared in the component header.

```voil
component UserBtn(
	text: String,
	disabled: Bool = false
):
	...
```

Accepted semantics:

- a parameter with no default is required;
- a parameter with a default is optional;
- component parameters are immutable inside the component;
- component invocation is named-argument-only;
- positional component arguments are compile errors;
- default expressions are evaluated independently for each component invocation;
- parameter/default initialization proceeds left-to-right;
- a default expression may reference parameters declared before it, but not parameters declared after it.

Example:

```voil
component UserCard(
	name: String,
	title: String = name,
	disabled: Bool = false
):
	...
```

Valid invocation:

```voil
UserCard(
	name="Alice"
)
```

Invalid positional invocation:

```voil
UserCard("Alice") # compile error
```

A parameter cannot be reassigned inside the component:

```voil
component UserBtn(text: String):
	text = "changed" # compile error
```

When mutable component-owned data is required, it must be copied explicitly into `state`:

```voil
component Input(value: String):
	state current = value

	fn clear():
		current = ""
```

`state current = value` reads the parameter during component-instance initialization. Later updates to the `value` parameter do not implicitly reset `current`.

Default expressions are evaluated per component invocation:

```voil
component Panel(id: String = create_id()):
	...

Panel()
Panel()
```

Each invocation evaluates `create_id()` independently.

Defaults may reference preceding parameters:

```voil
component Avatar(
	name: String,
	alt: String = name
):
	...
```

Referencing a later parameter is invalid because parameter initialization is left-to-right:

```voil
component Avatar(
	alt: String = name, # compile error
	name: String
):
	...
```

The surface separator for named arguments remains `=` in all current component syntax. The broader function/struct named-argument grammar is tracked separately, but component calls themselves are now defined as named-only.

## 4. Component instance scope

Status: `Accepted`.

Every component invocation creates a new component instance scope when that invocation first becomes active.

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

### 4.1 Reactive parameter updates preserve component identity

Status: `Accepted`.

If a caller expression supplying a component parameter changes reactively, the existing component instance receives the new immutable parameter value. The component is not destroyed and recreated solely because a parameter value changed.

```voil
state name = "Alice"

UserCard(
	name=name
)
```

After:

```voil
name = "Bob"
```

the same `UserCard` instance observes `name` changing from `"Alice"` to `"Bob"` while preserving its component-local `state` and ordinary scoped dependency instances.

This distinction is intentional:

```text
parameter update
	-> same component instance
	-> new immutable input value
	-> existing local state preserved

new component invocation identity
	-> new component instance
	-> component-local state initialized again
```

Changing a parameter therefore does not rerun component-local `state` initializers. In the earlier `Input(value)` example, a later `value` update does not overwrite `state current` unless component logic explicitly does so.

The remaining identity question is how invocation identity is tracked when components appear in conditional branches or repeated/list UI.

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

## 7. Component children and slots

Status: `Accepted` baseline.

A component invocation opens child content with the same `:` + indentation model used by the rest of Voiles.

```voil
Card(title="Profile"):
	std.p(user.name)
	std.p(user.email)
```

Ordinary child statements that are not inside a named-slot block form the default slot content.

A component renders the default slot with the compiler-level `slot` outlet:

```voil
component Card(title: String):
	container:
		std.h2(title)
		slot
```

`slot` is not an ordinary function call. It marks a UI insertion point owned by the component declaration.

Named slots use the same keyword with a slot name. The component declares insertion points:

```voil
component Modal():
	container:
		slot header
		slot
		slot footer
```

The caller supplies named content using `slot Name:` blocks while unwrapped child content still targets the default slot:

```voil
Modal():
	slot header:
		std.h2("Confirm")

	std.p("Delete this item?")

	slot footer:
		Button(text="Cancel")
		Button(text="Delete")
```

### 7.1 Slot lexical scope

Status: `Accepted`.

Slot content preserves the caller's lexical scope.

```voil
const username = "Alice"

Card():
	std.p(username)
```

`username` resolves where the `Card()` invocation is written. Moving that content into the component's `slot` outlet does not make the content part of the component declaration's lexical scope.

The component controls placement, but it does not gain lexical access to caller-local names merely because it renders their slot content. Likewise caller slot content cannot implicitly access component-local `const`, `state` or helper functions unless those values are explicitly exposed through a later slot-parameter mechanism.

### 7.2 Slot cardinality

Status: `Accepted` for v0.1.

All declared slot outlets are optional by default. If the caller provides no content for an outlet, that outlet renders nothing.

```voil
component Card():
	container:
		slot

Card() # valid; default slot is empty
```

A component declaration may contain at most one default `slot` outlet and at most one outlet for each named slot.

Invalid duplicate outlet:

```voil
component Modal():
	container:
		slot header
		slot header # compile error
```

A component invocation may provide each named slot at most once.

Invalid duplicate provision:

```voil
Modal():
	slot header:
		std.h2("A")

	slot header:
		std.h2("B") # compile error
```

A named slot provided by the caller must match a declared named outlet. Unknown named slots are compile errors.

Likewise, ordinary child content requires the callee to declare a default `slot` outlet. Passing default children to a component without a default outlet is a compile error.

One slot provision may contain any number of child UI statements. Cardinality applies to the slot provision/outlet itself, not to the number of nodes inside it.

Required slots, repeated projection/cloning and scoped-slot parameters are not part of the v0.1 baseline.

## 8. Unused component elimination

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

However the compiler must not blindly remove an entire module only because all of its component declarations are unused. If module-top-level initialization can have observable side effects, those effects must be preserved unless effect analysis proves the module itself removable.

Therefore dead-code elimination is semantics-preserving, not merely name-based.

Long-term direction:

- prefer analyzable or effect-free top-level initialization;
- remove unreachable component bodies;
- remove unreachable helper functions/types/constants when safe;
- remove entire modules only when no observable top-level behavior remains.

## 9. Initial grammar sketch

Descriptive only:

```text
componentDecl       := "component" PascalIdentifier parameters componentBlock
componentBlock      := ":" NEWLINE INDENT componentItem* DEDENT

componentItem       := bindingDecl
                    | functionDecl
                    | controlFlow
                    | structuralBlock
                    | containerBlock
                    | standardHtmlCall
                    | componentCall
                    | slotOutlet

componentCall       := PascalIdentifier componentCallArguments childBlock?
componentCallArguments := "(" namedArgumentList? ")"
childBlock          := ":" NEWLINE INDENT childItem* DEDENT
childItem           := namedSlotBlock | uiStatement
namedSlotBlock      := "slot" identifier block
slotOutlet          := "slot" identifier?

componentImportDecl := "import" componentImportItem ("," componentImportItem)* "from" stringLiteral
componentImportItem := PascalIdentifier componentAlias?
componentAlias      := "as" PascalIdentifier
```

`sharedDecl` is excluded from `componentItem` because `shared` is module-top-level only.

The module symbol table must reject duplicate component declaration names.

The comma-separated component import list is Accepted. `componentAlias` remains Draft until the alias form is explicitly accepted.

Component parameters form immutable bindings. Component invocation validation must reject positional arguments, missing required parameters, duplicate named arguments and unknown parameter names.

Component parameter defaults are evaluated per invocation from left to right and may only reference parameters already initialized earlier in the parameter list.

Slot child content must retain the caller lexical environment through lowering rather than being rebound as if it were declared inside the callee component.

Semantic validation must also reject duplicate slot outlets, duplicate named-slot provisions, unknown named-slot provisions and default child content passed to a component with no default slot outlet.

## 10. Remaining component decisions

- component identity in conditional/repeated UI and any key mechanism;
- slot parameter / scoped-slot model, if needed;
- slot content type model;
- callback/event parameter typing;
- component import alias syntax finalization (`as` currently recommended);
- mount/unmount and cleanup lifecycle;
- whether module-level mutable `state` is legal in modules that also declare components;
- helper function/type export rules;
- exact reachability/effect model used by tree-shaking.
