# Voiles Component Model

Status: `Draft` with accepted core semantics.

This document defines component declaration, export, parameters, instance identity, lifecycle, slots and compiler reachability.

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

The compiler does not infer whether an arbitrary `.voil` module is a component from render statements.

```text
fn
	-> callable logic

component
	-> instantiable UI
```

## 2. Automatic component export

Status: `Accepted`.

Every top-level `component Name(...):` declaration is automatically part of the module component export surface. No explicit `export` keyword is required.

```voil
component UserBtn(text: String):
	...

component IconBtn(icon: String):
	...
```

Both declarations are automatically exportable and their names must be unique within the module.

Single import:

```voil
import UserBtn from "./buttons.voil"
```

Multiple component import:

```voil
import UserBtn, IconBtn from "./buttons.voil"
```

Each imported name resolves the component declaration with the same name.

Component import alias remains Draft. Current recommendation:

```voil
import UserBtn as PrimaryBtn, IconBtn as SmallIconBtn from "./buttons.voil"
```

Mixed aliasing would also be valid if accepted:

```voil
import UserBtn, IconBtn as SmallIconBtn from "./buttons.voil"
```

Non-component module/default/named/namespace import semantics remain a separate module-system decision.

## 3. Component parameters

Status: `Accepted`.

Component parameters are immutable public input bindings declared in the component header.

```voil
component UserCard(
	name: String,
	title: String = name,
	disabled: Bool = false
):
	...
```

Accepted semantics:

- no default -> required parameter;
- default present -> optional parameter;
- component parameters are immutable inside the component;
- component invocation is named-argument-only;
- positional component arguments are compile errors;
- defaults are evaluated independently for each component invocation;
- parameter/default initialization is left-to-right;
- a default may reference only parameters already initialized earlier in the parameter list.

Valid:

```voil
UserCard(
	name="Alice"
)
```

Invalid positional call:

```voil
UserCard("Alice") # compile error
```

Invalid reassignment:

```voil
component UserBtn(text: String):
	text = "changed" # compile error
```

Mutable component-owned data must be explicit `state`:

```voil
component Input(value: String):
	state current = value

	fn clear():
		current = ""
```

`state current = value` reads the parameter when the component instance is created. Later updates to `value` do not implicitly reset `current`.

Per-invocation defaults:

```voil
component Panel(id: String = create_id()):
	...

Panel()
Panel()
```

Each invocation evaluates `create_id()` independently.

Defaults may reference earlier parameters:

```voil
component Avatar(
	name: String,
	alt: String = name
):
	...
```

A later parameter is not visible to an earlier default:

```voil
component Avatar(
	alt: String = name, # compile error
	name: String
):
	...
```

Component calls use named `=` arguments. The broader function/struct named-argument grammar is tracked separately.

## 4. Component instance identity

Status: `Accepted`.

Every component invocation identity creates a new component instance scope when that identity first becomes active.

```voil
UserBtn(text="A")
UserBtn(text="B")
```

If the component owns:

```voil
state count = 0
```

then each invocation identity owns an independent `count` cell.

```text
UserBtn A
	└─ count A

UserBtn B
	└─ count B
```

A component instance is also an importer scope for ordinary scoped `.voil` dependencies. Different component instances therefore receive different ordinary stateful dependency module instances unless the dependency uses `shared`.

Within one component instance, importing the same resolved module path more than once still resolves to the same dependency module instance.

### 4.1 Reactive parameter updates preserve identity

Status: `Accepted`.

Reactive changes to caller expressions update the immutable parameter values on the existing component instance. A parameter change alone does not destroy/recreate the component.

```voil
state name = "Alice"

UserCard(
	name=name
)

name = "Bob"
```

The same `UserCard` instance receives `"Bob"` while preserving component-local `state` and scoped dependency instances.

```text
parameter update
	-> same component instance
	-> new immutable input value
	-> existing local state preserved
```

Component-local `state` initializers run only for a new component instance identity.

### 4.2 Structural identity and conditionals

Status: `Accepted`.

Outside repeated UI, component identity is derived from the stable structural invocation position in the compiled UI structure.

Conditional branches are lifetime boundaries.

```voil
if show_profile:
	UserCard(
		name=user.name
	)
```

Conceptually:

```text
false -> true
	-> create UserCard instance

true -> false
	-> unmount UserCard instance
	-> release ordinary instance-local state/dependencies

false -> true
	-> create a new UserCard instance
```

Branch removal does not keep hidden component state alive by default.

### 4.3 Keyed repeated UI

Status: `Accepted`.

Repeated UI that creates component instances must declare explicit iteration identity with `key`.

```voil
for user in users key user.id:
	UserCard(
		user=user
	)
```

Voiles does not silently use list position/index as component identity.

Therefore this is a compile error in v0.1:

```voil
for user in users:
	UserCard(
		user=user
	) # compile error: repeated component UI requires key
```

The key belongs to repeated iteration identity, not to the component parameter list.

Reorder example:

```text
before: A B C
after:  C A B
```

If keys remain `A`, `B`, and `C`, all component instances are preserved and only rendered ordering changes.

If a key disappears, its instance is unmounted. A new key creates a new instance. A changed key removes the old identity and creates a new one.

For v0.1, accepted key types are `String` and `Int`.

Duplicate keys within one repeated UI evaluation are invalid. Because uniqueness may depend on runtime data, the runtime must detect duplicates and fail deterministically rather than guessing which instance to reuse.

Ordinary algorithmic loops that do not instantiate repeated UI components do not require a key.

## 5. Component lifecycle and resource cleanup

Status: `Accepted` for v0.1.

Voiles uses a paired `mount` / nested `cleanup` lifecycle for resources owned by a component instance.

```voil
component Clock():
	state now = get_time()

	mount:
		const timer = start_timer():
			now = get_time()

		cleanup:
			timer.stop()

	std.p(now)
```

Accepted semantics:

- `mount:` runs exactly once when a new component instance becomes mounted/active;
- reactive parameter updates do not rerun `mount:`;
- ordinary component-local `state` updates do not rerun `mount:`;
- keyed reorder with an unchanged key preserves the instance and does not rerun `mount:`;
- `cleanup:` runs exactly once when that mounted component instance is unmounted;
- `cleanup:` may capture lexical bindings declared in its enclosing `mount:` block;
- `cleanup:` is legal only inside a `mount:` block;
- v0.1 does not introduce reactive `effect`, dependency arrays or automatic rerun semantics.

Resource example:

```voil
mount:
	const subscription = store.subscribe(update)

	cleanup:
		subscription.close()
```

The lexical relationship is intentional: the resource is created and destroyed in the same lifecycle block without forcing the resource handle into component `state`.

Conditional lifetime follows component identity rules:

```voil
if show_clock:
	Clock()
```

```text
false -> true
	-> create instance
	-> mount runs

true -> false
	-> cleanup runs
	-> unmount instance

false -> true
	-> create new instance
	-> mount runs again for the new instance
```

For keyed repeated UI:

- reorder with the same key -> no cleanup/remount;
- key removal -> cleanup then unmount;
- key replacement/change -> cleanup old instance, then mount new instance;
- new key -> mount new instance.

`cleanup` defines component-owned resource teardown. Cleanup rules for ordinary imported scoped modules remain a separate module-lifecycle decision.

## 6. Scope inside and outside component blocks

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

- `shared` is legal only at module top level and is application-global;
- component-local `const` / `state` belong to the component instance scope;
- ordinary module-level declarations outside `component` retain module lexical semantics.

Whether module-level mutable `state` should be permitted in a module that also declares components remains open. Component-local mutable state should normally be declared inside the component block so instance isolation is explicit.

## 7. Component children and slots

Status: `Accepted` baseline.

A component invocation opens child content with the normal `:` + indentation model.

```voil
Card(title="Profile"):
	std.p(user.name)
	std.p(user.email)
```

Ordinary child statements form default slot content.

Default outlet:

```voil
component Card(title: String):
	container:
		std.h2(title)
		slot
```

Named outlets:

```voil
component Modal():
	container:
		slot header
		slot
		slot footer
```

Caller-provided named content:

```voil
Modal():
	slot header:
		std.h2("Confirm")

	std.p("Delete this item?")

	slot footer:
		Button(text="Cancel")
		Button(text="Delete")
```

`slot` is a compiler-level UI insertion point, not an ordinary function call.

### 7.1 Slot lexical scope

Status: `Accepted`.

Slot content preserves caller lexical scope.

```voil
const username = "Alice"

Card():
	std.p(username)
```

`username` resolves where `Card()` is invoked, not where `component Card` is declared.

The component controls placement only. Caller slot content does not implicitly gain component-local bindings, and the component does not gain caller-local bindings through projection.

A future slot-parameter/scoped-slot feature must explicitly define values crossing this boundary.

### 7.2 Slot cardinality

Status: `Accepted` for v0.1.

All slot outlets are optional by default.

A component declaration may contain:

- at most one default `slot` outlet;
- at most one outlet for each named slot.

A component invocation may provide each named slot at most once.

Compile errors:

- duplicate default/named outlets;
- duplicate named-slot provisions;
- unknown named slot supplied by caller;
- ordinary/default children supplied when callee has no default outlet.

One slot provision may contain any number of child UI statements.

Required slots, repeated projection/cloning and scoped-slot parameters are not part of the v0.1 baseline.

## 8. Unused component elimination

Status: `Accepted` compiler goal.

A component unreachable from route/application entry points and reachable component/module references may be removed from production output.

```text
route entrypoints
	↓
reachable imports/components
	↓
component dependency graph
	↓
codegen set
```

An imported component that is never invoked can also be removed when removal preserves module semantics.

Component-local initialization and lifecycle work occur only when the component is instantiated. Therefore an unused component declaration must not create component state, run `mount`, or create render output merely because its containing module exists.

The compiler must not blindly remove an entire module solely because all component declarations are unused. Observable module-top-level side effects must be preserved unless effect analysis proves the module removable.

## 9. Initial grammar sketch

Descriptive only:

```text
componentDecl          := "component" PascalIdentifier parameters componentBlock
componentBlock         := ":" NEWLINE INDENT componentItem* DEDENT

componentItem          := bindingDecl
                       | functionDecl
                       | controlFlow
                       | structuralBlock
                       | containerBlock
                       | standardHtmlCall
                       | componentCall
                       | slotOutlet
                       | mountBlock

componentCall          := PascalIdentifier componentCallArguments childBlock?
componentCallArguments := "(" namedArgumentList? ")"
childBlock             := ":" NEWLINE INDENT childItem* DEDENT
childItem              := namedSlotBlock | uiStatement
namedSlotBlock         := "slot" identifier block
slotOutlet             := "slot" identifier?

mountBlock             := "mount" block
cleanupBlock           := "cleanup" block

keyedForUi             := "for" identifier "in" expression "key" expression block

componentImportDecl    := "import" componentImportItem ("," componentImportItem)* "from" stringLiteral
componentImportItem    := PascalIdentifier componentAlias?
componentAlias         := "as" PascalIdentifier
```

Semantic validation must:

- reject `sharedDecl` outside module top level;
- reject duplicate component declaration names;
- reject positional component arguments, missing required parameters, duplicate named arguments and unknown parameter names;
- evaluate component defaults per new instance, left-to-right, with references only to earlier parameters;
- preserve existing component instances across reactive parameter updates;
- reject component-instantiating repeated UI without explicit `key`;
- type-check v0.1 keys as `String` or `Int`;
- detect duplicate repeated-UI keys at runtime;
- preserve caller lexical scope through slot lowering;
- validate slot cardinality and unknown slots;
- permit `cleanup:` only as a nested lifecycle block inside `mount:`;
- preserve mount lexical bindings for cleanup capture;
- ensure `mount:` executes once per mounted instance identity and `cleanup:` once per unmount.

## 10. Remaining component decisions

- slot parameter / scoped-slot model, if needed;
- slot content type model;
- callback/event parameter typing;
- component import alias syntax finalization (`as` currently recommended);
- scoped module cleanup/lifetime semantics separate from component-owned `mount` resources;
- whether module-level mutable `state` is legal in modules that also declare components;
- helper function/type export rules;
- exact reachability/effect model used by tree-shaking.