# Voiles Runtime Implementation Contract

Status: `Implementation contract; runtime not started`.

## 1. Runtime goals

The runtime must preserve compiler-defined identity and ownership rather than emulate whole-component rerendering.

Core responsibilities:

- fine-grained observed `state/shared` updates;
- component structural/key identity;
- scoped module instances;
- deterministic init/mount/cleanup ordering;
- slot projection placement without caller-scope rebinding;
- deterministic runtime validation such as duplicate keys and bounds errors.

## 2. Scoped module ownership

```text
same importer scope + resolved path
-> same ordinary module instance
```

Different importer scopes receive different ordinary instances. `shared` cells are application-global per declaration identity.

Runtime `.voil` dependency graph is acyclic in v0.1.

Initialization:

```text
dependencies
-> module init
-> owner lifecycle code
```

Teardown is post-order:

```text
owned dependency cleanup
-> importer module cleanup
-> component cleanup
-> owner-local state release
```

## 3. Component identity

Non-repeated UI uses stable compiled structural position. Repeated component UI uses explicit `String | Int` iteration key.

- prop change at same identity preserves local state;
- conditional removal unmounts;
- conditional re-entry creates a new instance;
- same-key reorder preserves instance;
- removed/changed key tears old identity down;
- duplicate key in one evaluation is a runtime error.

## 4. Lifecycle blocks

Component resources:

```voil
mount:
	...
	cleanup:
		...
```

Scoped module resources:

```voil
init:
	...
	cleanup:
		...
```

Cleanup captures enclosing lifecycle locals and executes once for the corresponding owner destruction.

## 5. Reactivity

`state/shared` are mutable declarations; reactive machinery is generated only for observed cells. Unobserved function-local state may compile to plain mutable storage.

The lowering IR, not the source AST, should own observer edges and update operations.

## 6. HMR constraints

Development HMR may preserve compatible component/shared identity, but incompatible declaration shape remounts or reinitializes the affected boundary. HMR behavior must never redefine production lifetime semantics.
