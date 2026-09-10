# Voiles State / Scope Model

Status: `Draft` with accepted core semantics.

This document defines the current direction for `const`, `state`, `shared`, lexical scope, module instancing, component parameters/instance state, component identity and slot lexical scope.

## 1. Lexical scope

`const`, `state` and `shared` names follow lexical scope for name visibility.

Name resolution searches from the innermost block outward. An inner declaration may shadow an outer declaration.

```voil
state i = 1

fn count():
	i += 1

fn count_2():
	state i = 5
	i += 1
	# local i == 6
```

In `count()`, `i` resolves to the outer binding. In `count_2()`, the local `state i` shadows the outer binding.

A declaration is not visible before its declaration point. Redeclaring the same name in the same lexical scope is a compile error.

## 2. Binding kinds

### 2.1 `const`

```voil
const title = "Voiles"
```

Semantics:

- immutable binding;
- can be initialized from a runtime expression;
- does not mean compile-time constant;
- follows ordinary lexical scope;
- cannot be reassigned.

### 2.2 `state`

```voil
state count = 0
```

Semantics:

- mutable binding;
- reactive when observed by generated UI/runtime dependencies;
- follows lexical scope;
- storage lifetime is tied to the owning scope/module/component instance;
- mutation keeps the declared/inferred type fixed.

A function-local state is recreated for each function invocation unless later captured by an explicitly supported closure/lifetime mechanism.

```voil
fn count_once() -> Int:
	state i = 5
	i += 1
	return i
```

Every independent call begins from `5` and returns `6`.

## 3. Scoped module instances

Voiles `.voil` imports are not assumed to behave like application-global JavaScript ESM singletons.

Accepted rule:

> The same importer scope + the same resolved `.voil` module path resolves to the same module instance.

Example:

```voil
# btn_state.voil
state i = 0
```

```voil
# a.voil
import btn_state from "./btn_state.voil"

btn_state.i += 1
# btn_state.i == 1
```

```voil
# b.voil
import btn_state from "./btn_state.voil"

btn_state.i += 1
# btn_state.i == 1
```

Conceptually:

```text
btn_state.voil definition
	├─ imported from a.voil -> instance A -> i = 1
	└─ imported from b.voil -> instance B -> i = 1
```

Different import aliases do not create extra instances when importer scope and resolved path are identical:

```voil
import a from "./store.voil"
import b from "./store.voil"
```

Within this importer scope, `a` and `b` refer to the same resolved module instance.

This identity rule prevents aliases or duplicate import statements from silently cloning mutable state.

## 4. `shared`

Sharing is declared on the variable, not on the module or import statement.

```voil
shared i = 1
```

`shared` does not require `state` because mutability is inherent in the declaration.

Accepted semantics:

- mutable;
- reactive when observed by UI/runtime dependencies;
- one shared storage cell per declaration identity across all instances of the declaring `.voil` module within the application runtime;
- ordinary imports do not need a `shared import` form;
- `shared` is only legal at module top level.

Example:

```voil
# store.voil
shared count = 0
```

```voil
# a.voil
import store from "./store.voil"
store.count += 1
# shared count == 1
```

```voil
# b.voil
import store from "./store.voil"
store.count += 1
# shared count == 2
```

The module instances remain different, but `shared count` intentionally points to the same shared storage cell.

Conceptually:

```text
store.voil declaration: shared count
	              │
	      shared storage cell
	              │
	      ┌───────┴───────┐
	      │               │
	instance A        instance B
	(imported by A)   (imported by B)
```

This keeps sharing explicit at the exact declaration that can create cross-scope mutation.

## 5. `shared` is top-level only

Status: `Accepted`.

`shared` represents application-global storage, so it is valid only at module top level.

Valid:

```voil
shared session = none

fn update_session(...):
	...
```

Invalid inside a component:

```voil
component Counter():
	shared i = 0
```

Invalid inside a function:

```voil
fn test():
	shared i = 0
```

Also invalid inside other nested blocks:

```voil
if ready:
	shared cache = none
```

The compiler must report these declarations as syntax/semantic errors.

This avoids introducing static-local semantics and avoids ambiguity around recursive calls, closures, async tasks and block lifetime.

## 6. Component declaration, parameters and instance scope

Status: `Accepted`.

A user component is explicitly declared with `component`:

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
			onclick=click
		)
		std.p(text)
		std.p(count)
```

Component parameters are immutable input bindings. A required parameter has no default; an optional parameter has a default. Component invocations are named-argument-only.

Defaults are evaluated independently for each component invocation and initialized left-to-right. A default may reference only parameters already initialized earlier in the declaration.

Every component invocation creates an independent component instance scope when that invocation identity first becomes active.

Usage:

```voil
UserBtn(text="A")
UserBtn(text="B")
```

Conceptually:

```text
UserBtn invocation A
	├─ text = "A"
	└─ state count = 0

UserBtn invocation B
	├─ text = "B"
	└─ state count = 0
```

After clicking only A:

```text
A.count = 1
B.count = 0
```

The two component instances must not share ordinary `state` merely because they originate from the same component declaration.

A component instance is also an importer scope for ordinary scoped `.voil` dependencies used by that component. Therefore, if the component imports another stateful module, each component instance receives its own dependency module instance unless the dependency uses `shared`.

Example:

```voil
# local_store.voil
state value = 0
```

```voil
# Counter.voil
import store from "./local_store.voil"

component Counter():
	...
```

Two `Counter()` invocations conceptually produce:

```text
Counter A
	└─ local_store instance A
		└─ state value

Counter B
	└─ local_store instance B
		└─ state value
```

Within one component instance, importing the same resolved module path more than once still resolves to the same dependency module instance.

`shared` remains the explicit exception: a `shared` declaration points to the same application-global storage regardless of which component instance reaches it.

### 6.1 Parameter updates preserve instance state

Status: `Accepted`.

A reactive change in a caller expression supplying a parameter updates that immutable parameter binding on the existing component instance. It does not recreate the component solely because the value changed.

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

the same `UserCard` instance observes the new input value and preserves its component-local `state` and ordinary imported module instances.

Component-local `state` initializers run when a new component instance is created, not every time a parameter changes.

Therefore:

```voil
component Input(value: String):
	state current = value
```

uses the initial `value` to initialize `current`; a later update to the `value` parameter does not implicitly overwrite `current`.

### 6.2 Structural identity, conditionals and keyed repeated UI

Status: `Accepted`.

Outside repeated UI, a component invocation uses its stable structural invocation position as its runtime identity. Reactive changes to values used at that position preserve the same component instance.

A conditional branch is an explicit lifetime boundary. When a branch becomes inactive, component instances reachable only from that branch are unmounted and their ordinary instance-local state/dependency instances are released. Re-entering the branch creates new instances.

```voil
if show_profile:
	UserCard(
		name=user.name
	)
```

Conceptually:

```text
false -> true
	-> create instance

true -> false
	-> unmount instance
	-> release local state/dependencies

false -> true
	-> create new instance
```

Repeated UI that creates component instances must define explicit iteration identity using `key`:

```voil
for user in users key user.id:
	UserCard(
		user=user
	)
```

Voiles does not silently use the iteration index/list position as component identity. A component-instantiating UI loop without `key` is therefore a compile error in v0.1.

Stable keys preserve instance state across reorders. If items keyed `A`, `B`, `C` reorder to `C`, `A`, `B`, the runtime preserves all three component instances and only changes rendered ordering.

If a key disappears, its instance is unmounted. If a new key appears, a new instance is created. If an item's key changes, that is an identity change: the old instance is removed and a new instance is initialized.

For v0.1, accepted key types are stable scalar `String` or `Int` values. Duplicate keys within one repeated UI evaluation are invalid. Because uniqueness may depend on runtime data, duplicate-key detection is a runtime validation error rather than an ambiguous reuse heuristic.

Ordinary algorithmic loops that do not instantiate repeated UI components do not require `key`.

## 7. Component export identity

Status: `Accepted`.

Every top-level `component Name(...):` declaration is automatically part of the module's component export surface. Source code does not need an `export` keyword.

A `.voil` file may contain multiple component declarations, and each declaration has its own component identity.

```voil
component UserBtn(...):
	...

component IconBtn(...):
	...
```

Component names must be unique within the module.

This rule is called automatic or implicit component export. It should not be modeled as JavaScript's single `default export`, because Voiles permits more than one automatically exportable component declaration per module.

Component definitions that are unreachable from application entry points may be removed by compiler tree-shaking. Their component-local state is never instantiated unless the component itself is invoked.

Component imports may select multiple declarations using a comma-separated list. Alias syntax remains a separate Draft decision.

## 8. Slot lexical scope

Status: `Accepted`.

Component child content is lexically owned by the caller, even though the callee decides where that content is rendered through `slot` outlets.

```voil
const username = "Alice"

Card():
	std.p(username)
```

If `Card` renders its default children with:

```voil
component Card():
	container:
		slot
```

then `username` still resolves in the caller scope where `Card()` appears. The slot body is not rebound into the lexical scope of the `Card` declaration.

The same rule applies to named slot blocks:

```voil
Modal():
	slot header:
		std.h2(username)
```

The component controls the insertion point but cannot implicitly access caller-local bindings through slot projection. Likewise slot content cannot implicitly access component-local `const`, `state` or helper bindings.

A future slot-parameter/scoped-slot feature, if added, must explicitly declare any values crossing from the component instance into caller-authored slot content.

## 9. State identity summary

```text
const
	immutable
	lexical scope

component parameter
	immutable input binding
	named-only at invocation
	default evaluated per component invocation
	parameter updates preserve component instance/local state

state
	mutable
	lexical scope
	storage belongs to current scope/module/component instance
	different importer scopes may receive independent module state
	different component invocation identities receive independent component state

shared
	mutable
	module top-level only
	storage is shared across module and component instances
	intended for explicit application-global state

slot content
	caller lexical scope
	callee controls placement only
```

## 10. Module / component identity summary

```text
same importer scope + same resolved path
	-> same module instance

different importer scope + same resolved path
	-> different module instance

state in that module
	-> independent per module instance

component declaration
	-> explicit UI component identity
	-> automatically exportable

stable structural invocation position
	-> same component instance across reactive value updates

conditional branch removal
	-> unmount component instance
	-> release ordinary local state/dependency instances

conditional branch re-entry
	-> new component instance

repeated UI
	-> explicit key required when components are instantiated
	-> same key preserves instance across reorder
	-> removed/changed key removes old identity
	-> new key creates new identity

parameter update on existing invocation identity
	-> same component instance
	-> local state and dependency instances retained

slot content
	-> caller lexical environment is retained across projection

shared in that module
	-> same shared storage across all instances
```

## 11. Remaining decisions

- Whether `shared` is always reactive or whether reactivity is generated only when an observer exists. Current preference: mutable declaration with compiler-generated reactivity only when observed.
- Component import alias syntax.
- Exact export/access syntax for non-component module bindings.
- Slot parameters/content typing if a concrete use case requires them.
- Cyclic import initialization rules for scoped module instances.
- Exact cleanup hook/API semantics when a component/importer/module instance becomes unreachable.
- Top-level side-effect policy and how it constrains whole-module tree-shaking.