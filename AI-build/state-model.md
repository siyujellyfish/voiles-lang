# Voiles State / Scope Model

Status: `Draft` with accepted core semantics.

This document defines the current direction for `const`, `state`, `shared`, lexical scope and module instancing.

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
- storage lifetime is tied to the owning scope/module instance;
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

> The same importer + the same resolved `.voil` module path resolves to the same module instance.

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

Different import aliases do not create extra instances when importer and resolved path are identical:

```voil
import a from "./store.voil"
import b from "./store.voil"
```

Within this importer, `a` and `b` refer to the same resolved module instance.

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

## 6. State identity summary

```text
const
	immutable
	lexical scope

state
	mutable
	lexical scope
	storage belongs to current scope/module instance
	different importer scopes may receive independent module state

shared
	mutable
	module top-level only
	storage is shared across module instances
	intended for explicit application-global state
```

## 7. Module identity summary

```text
same importer + same resolved path
	-> same module instance

different importer + same resolved path
	-> different module instance

state in that module
	-> independent per module instance

shared in that module
	-> same shared storage across those instances
```

## 8. Remaining decisions

- Whether `shared` is always reactive or whether reactivity is generated only when an observer exists. Current preference: mutable declaration with compiler-generated reactivity only when observed.
- Exact export/access syntax for module bindings.
- Component invocation state identity relative to module instance identity.
- Cyclic import initialization rules for scoped module instances.
- Cleanup/lifetime rules when an importer/module instance becomes unreachable.
