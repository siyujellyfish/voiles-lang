# Voiles State / Scope Model

Status: `Draft` with accepted core semantics.

This document defines the current direction for `const`, `state`, `shared`, lexical scope, module instancing and component instance state.

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
# btn.voil
state i = 0
```

```voil
# a.voil
import btn from "./btn.voil"

btn.i += 1
# btn.i == 1
```

```voil
# b.voil
import btn from "./btn.voil"

btn.i += 1
# btn.i == 1
```

Conceptually:

```text
btn.voil definition
	├─ imported from a.voil -> btn instance A -> i = 1
	└─ imported from b.voil -> btn instance B -> i = 1
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

Invalid:

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

## 6. Component instance scope

Status: `Accepted`.

Every user component invocation creates an independent component instance scope.

Example component:

```voil
# UserBtn.voil
state count = 0

fn click():
	count += 1

container():
	std.button(
		onclick=click
	)
```

Usage:

```voil
UserBtn()
UserBtn()
```

Conceptually:

```text
UserBtn invocation A
	└─ state count = 0

UserBtn invocation B
	└─ state count = 0
```

After clicking only A:

```text
A.count = 1
B.count = 0
```

The two component instances must not share ordinary `state` merely because they originate from the same imported component definition.

A component instance is also an importer scope for ordinary scoped `.voil` dependencies used by that component. Therefore, if the component imports another stateful module, each component instance receives its own dependency module instance unless the dependency uses `shared`.

Example:

```voil
# local_store.voil
state value = 0
```

```voil
# Counter.voil
import store from "./local_store.voil"
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

## 7. State identity summary

```text
const
	immutable
	lexical scope

state
	mutable
	lexical scope
	storage belongs to current scope/module/component instance
	different importer scopes may receive independent module state
	different component invocations receive independent component state

shared
	mutable
	module top-level only
	storage is shared across module and component instances
	intended for explicit application-global state
```

## 8. Module / component identity summary

```text
same importer scope + same resolved path
	-> same module instance

different importer scope + same resolved path
	-> different module instance

state in that module
	-> independent per module instance

component invocation
	-> new component instance scope
	-> independent component-local state
	-> independent ordinary imported module state

shared in that module
	-> same shared storage across all instances
```

## 9. Remaining decisions

- Whether `shared` is always reactive or whether reactivity is generated only when an observer exists. Current preference: mutable declaration with compiler-generated reactivity only when observed.
- Exact export/access syntax for module bindings.
- How the compiler identifies the renderable component surface of a `.voil` module without requiring explicit file-role declarations.
- Cyclic import initialization rules for scoped module instances.
- Cleanup/lifetime rules when a component/importer/module instance becomes unreachable.
