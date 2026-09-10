# Voiles State / Scope Model

Status: `Draft` with accepted core semantics.

This document defines the current direction for `const`, `state`, `shared`, lexical scope, scoped modules, component parameters/identity/lifecycle and slot lexical scope.

## 1. Lexical scope

`const`, `state` and `shared` names follow lexical scope for visibility.

Name resolution searches from the innermost block outward. An inner declaration may shadow an outer declaration.

```voil
state i = 1

fn count():
	i += 1

fn count_2():
	state i = 5
	i += 1
```

In `count()`, `i` resolves to the outer binding. In `count_2()`, the local `state i` shadows it.

A declaration is not visible before its declaration point. Redeclaration in the same lexical scope is a compile error.

## 2. Binding kinds

### 2.1 `const`

```voil
const title = "Voiles"
```

Semantics:

- immutable runtime binding;
- may be initialized from a runtime expression;
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
- storage lifetime is tied to its owning scope/module/component instance;
- mutation keeps the declared/inferred type fixed.

Function-local state is recreated for each function invocation unless later captured by an explicitly supported closure/lifetime mechanism.

```voil
fn count_once() -> Int:
	state i = 5
	i += 1
	return i
```

Each independent call begins from `5` and returns `6`.

## 3. Scoped `.voil` module instances

Voiles `.voil` imports are not application-global JavaScript ESM-style singletons by default.

Accepted rule:

```text
same importer scope + same resolved .voil path
	-> same module instance

different importer scope + same resolved .voil path
	-> different module instance
```

Example:

```voil
# btn_state.voil
state i = 0
```

```voil
# a.voil
import btn_state from "./btn_state.voil"
btn_state.i += 1
# i == 1 in A's module instance
```

```voil
# b.voil
import btn_state from "./btn_state.voil"
btn_state.i += 1
# i == 1 in B's module instance
```

Different aliases do not create extra instances in the same importer scope:

```voil
import a from "./store.voil"
import b from "./store.voil"
```

`a` and `b` resolve to the same module instance in that importer scope.

## 4. `shared`

Sharing is declared on the variable, not on the module or import statement.

```voil
shared i = 1
```

Accepted semantics:

- mutable by definition;
- reactive when observed by UI/runtime dependencies;
- one shared storage cell per declaration identity across all instances of the declaring `.voil` module within the application runtime;
- no `shared import` form is required;
- `shared` is legal only at module top level.

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

The ordinary module instances remain different while `shared count` points to the same shared storage cell.

Invalid nested declarations:

```voil
component Counter():
	shared i = 0 # compile error
```

```voil
fn test():
	shared i = 0 # compile error
```

```voil
if ready:
	shared cache = none # compile error
```

This avoids implicit static-local semantics and ambiguous recursive/closure/async lifetimes.

## 5. Component parameters and instance scope

Status: `Accepted`.

Component parameters are immutable input bindings.

```voil
component UserBtn(
	text: String,
	disabled: Bool = false
):
	state count = 0
	...
```

Rules:

- no default -> required parameter;
- default present -> optional parameter;
- component invocation is named-argument-only;
- defaults are evaluated independently for every new component instance;
- parameter/default initialization is left-to-right;
- a default may reference only earlier initialized parameters.

Every component invocation identity creates an independent component instance scope when that identity first becomes active.

```voil
UserBtn(text="A")
UserBtn(text="B")
```

Conceptually:

```text
UserBtn A
	├─ text = "A"
	└─ state count A

UserBtn B
	├─ text = "B"
	└─ state count B
```

A component instance is also an importer scope for ordinary scoped `.voil` dependencies. Therefore ordinary stateful dependencies are isolated per component instance unless they use `shared`.

### 5.1 Reactive parameter updates preserve instance state

Status: `Accepted`.

A reactive caller expression updates the immutable parameter value on the existing component instance instead of recreating the component.

```voil
state name = "Alice"

UserCard(
	name=name
)

name = "Bob"
```

The same `UserCard` instance receives `"Bob"`; component-local `state` and ordinary imported module instances remain alive.

Component-local state initializers run only for a new component instance identity.

```voil
component Input(value: String):
	state current = value
```

`current` receives the initial `value` once. Later updates to the `value` parameter do not implicitly overwrite it.

## 6. Component identity and lifetime

Status: `Accepted`.

### 6.1 Structural identity

Outside repeated UI, a stable structural invocation position defines component identity.

Reactive value/parameter changes at the same position preserve the same instance.

### 6.2 Conditional lifetime

Conditional branch removal is an unmount boundary.

```voil
if show_profile:
	UserCard(
		name=user.name
	)
```

```text
false -> true
	-> create instance

true -> false
	-> unmount instance
	-> release ordinary local state/dependency instances

false -> true
	-> create new instance
```

Voiles does not implicitly keep branch-local component state hidden after the branch becomes inactive.

### 6.3 Keyed repeated UI

Repeated UI that creates components requires explicit iteration identity:

```voil
for user in users key user.id:
	UserCard(
		user=user
	)
```

Voiles does not silently use iteration index/list position as identity.

A component-instantiating repeated UI loop without `key` is a compile error in v0.1.

Stable keys preserve component instances across reorder. Removed keys unmount their instances; new keys create new instances; changed keys replace identity.

v0.1 keys are `String` or `Int`.

Duplicate keys in the same repeated UI evaluation are runtime errors because uniqueness can depend on runtime data.

Ordinary algorithmic loops that do not create repeated component UI do not require keys.

## 7. Component mount/cleanup lifetime

Status: `Accepted` for v0.1.

Component-owned resources use `mount:` with an optional nested `cleanup:` block.

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

Lifecycle semantics:

```text
new mounted instance
	-> mount runs exactly once

reactive parameter/state update
	-> same instance
	-> mount does not rerun

keyed reorder with same key
	-> same instance
	-> mount does not rerun
	-> cleanup does not run

instance unmount
	-> cleanup runs exactly once
	-> instance-local state/dependencies released
```

`cleanup:` is valid only inside `mount:` and may capture lexical bindings from the enclosing mount block.

```voil
mount:
	const subscription = store.subscribe(update)

	cleanup:
		subscription.close()
```

This keeps resource creation and teardown in one lexical lifecycle scope without requiring the resource handle to be stored in component `state`.

Conditional and keyed identity interact with lifecycle as follows:

- conditional branch removal -> cleanup then unmount;
- conditional re-entry -> new instance, then mount;
- keyed reorder with unchanged key -> no mount/cleanup;
- key removal/replacement -> cleanup old instance;
- new key -> mount new instance.

v0.1 does not define reactive `effect`, dependency arrays or automatic effect reruns.

## 8. Scoped module initialization and cleanup

Status: `Accepted` for v0.1 baseline.

Ordinary scoped `.voil` module instances are owned by their importer scope. Module-owned resources use module-top-level `init:` with an optional nested `cleanup:` block.

```voil
# chat.voil
state connected = false

init:
	const socket = open_socket("/chat")
	connected = true

	cleanup:
		socket.close()
```

Accepted lifecycle semantics:

```text
new scoped module instance
	-> init runs once

same importer scope + same resolved path
	-> reuse the same module instance
	-> init does not rerun

owner reactive update
	-> existing module instance is preserved
	-> init does not rerun

owner/importer scope destruction
	-> owned dependency modules cleanup recursively
	-> this module cleanup runs once
	-> ordinary module-local state is released
```

`cleanup:` in a module lifecycle is legal only inside the enclosing module `init:` block and may capture lexical bindings declared by that `init:` block.

Different aliases that resolve to the same scoped module instance do not duplicate lifecycle execution: the instance initializes once and cleans up once.

### 8.1 Dependency teardown order

Status: `Accepted`.

Scoped module ownership is torn down in dependency-first/post-order sequence.

```text
Component
└─ module A
   └─ module B
```

Teardown order:

```text
module B cleanup
-> module A cleanup
-> Component cleanup
-> Component unmount/state release
```

The same rule applies recursively through deeper ordinary scoped-module dependency chains. This makes dependency lifetime strictly nested inside owner lifetime.

Dependencies initialize before owner lifecycle code can rely on them. Cyclic import initialization/cleanup remains a separate unresolved case because a cycle does not form a simple ownership tree.

### 8.2 `shared` lifetime is independent

Ordinary scoped-module cleanup does not destroy or reset `shared` storage.

```voil
# store.voil
shared user = none
state local_cache = none
```

Destroying one ordinary `store.voil` instance releases that instance's `local_cache`, but the application-global `shared user` cell remains alive for other module/component instances.

A `shared` binding that owns an application-global external resource requires a separate application-global resource-lifetime policy. v0.1 ordinary module `cleanup:` must not infer ownership of such shared resources.

## 9. Slot lexical scope

Status: `Accepted`.

Component child content is lexically owned by the caller even though the callee chooses the insertion point through `slot` outlets.

```voil
const username = "Alice"

Card():
	std.p(username)
```

If `Card` renders:

```voil
component Card():
	container:
		slot
```

then `username` still resolves in the caller scope.

The same applies to named slots:

```voil
Modal():
	slot header:
		std.h2(username)
```

The component cannot implicitly access caller-local bindings through slot projection, and slot content cannot implicitly access component-local bindings.

Any future scoped-slot parameters must explicitly define values crossing that boundary.

## 10. State and identity summary

```text
const
	immutable
	lexical scope

component parameter
	immutable input
	named-only invocation
	default evaluated per new instance
	parameter update preserves instance/local state

state
	mutable
	lexical scope
	storage belongs to owning function/module/component instance

shared
	mutable
	module top-level only
	one application-global cell per declaration identity
	lifetime is independent from ordinary scoped-module cleanup

scoped module instance
	owned by importer scope
	init once per new instance
	cleanup once when owner lifetime ends
	dependencies cleanup before importer/owner

stable structural component position
	-> preserve instance across reactive updates

conditional removal
	-> cleanup owned scoped modules dependency-first
	-> cleanup component-owned mount resources
	-> unmount
	-> release ordinary instance-local state/dependencies

conditional re-entry
	-> new instance
	-> initialize dependencies
	-> mount

repeated UI
	-> explicit String/Int key required for component identity
	-> same key preserves instance across reorder
	-> removed/changed key unmounts old identity
	-> new key creates/mounts new identity

slot content
	caller lexical scope
	callee controls placement only
```

## 11. Remaining decisions

- Whether `shared` is always reactive or reactivity is generated only when an observer exists. Current preference: compiler-generated reactivity only when observed.
- Component import alias syntax.
- Exact export/access syntax for non-component module bindings.
- Slot parameter/content typing if a concrete use case requires it.
- Cyclic import initialization/cleanup for scoped module instances and `shared` bindings.
- Application-global resource lifecycle for resources intentionally stored in `shared` bindings.
- Module top-level side-effect policy and whole-module tree-shaking boundary.