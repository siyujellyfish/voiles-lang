# Voiles TODO

## Current phase: syntax specification

### Resolved syntax decisions

- [x] Use `:` for every block opener.
- [x] Use significant indentation for block hierarchy.
- [x] Use `#` as comment syntax only.
- [x] Require quoted import module specifiers.
- [x] Require `fn` for function declarations.
- [x] Allow semantic page structure directly at module level without a `view:` wrapper.
- [x] Use `std.*`-style standard HTML namespace to visually distinguish native HTML APIs from user components.
- [x] Use function-like PascalCase syntax for user component invocation.
- [x] Use lexical scope with nearest-binding resolution and shadowing.
- [x] Same importer + same resolved `.voil` path resolves to the same module instance.
- [x] Different import aliases in the same importer do not clone module state.
- [x] Declare cross-module-instance sharing on the variable with `shared`, not on the module/import.
- [x] `shared` is mutable by definition and does not require an additional `state` keyword.

### P0 — parser-blocking decisions

- [ ] Define exact `NEWLINE` / `INDENT` / `DEDENT` tokenization rules.
- [ ] Define multiline parenthesized expression continuation rules.
- [ ] Confirm named argument separator `=` across components, struct construction and function defaults.
- [ ] Define unambiguous grammar for structural HTML block vs ordinary block/call.
- [ ] Define user component child-block grammar.
- [ ] Define initial expression precedence table.
- [ ] Define syntax error recovery for malformed indentation and unfinished blocks.

### P0 — binding / state model

- [ ] Finalize `const` semantics as immutable runtime binding.
- [ ] Finalize `state` semantics as mutable scoped binding with compiler-generated reactivity when observed.
- [ ] Finalize `shared` semantics as mutable storage shared across declaring module instances.
- [ ] Decide whether v0.1 restricts `shared` to module top-level.
- [ ] Decide whether v0.1 removes `let` / `var` entirely.
- [ ] Evaluate whether function-local non-reactive mutation requires `mut`, or whether unobserved local `state` lowers to ordinary mutable storage.
- [ ] Define closure capture rules for `const` / `state`.
- [ ] Define module-instance creation and cleanup lifecycle.
- [ ] Define cyclic import initialization for scoped modules and `shared` bindings.
- [ ] Define component invocation state identity relative to imported module instance identity.

### P1 — standard HTML surface

- [ ] Define minimum `@voiles/html-base` API.
- [ ] Define canonical `std` alias policy: convention vs reserved language namespace.
- [ ] Define the boundary between syntax-level structural blocks (`main:`, `header:`) and `std.*` elements.
- [ ] Define typed HTML attributes.
- [ ] Define native event handler types.
- [ ] Define HTML escaping behavior.
- [ ] Define boolean attribute lowering.

### P1 — components

- [ ] Define component file/default export model.
- [ ] Finalize component prop declaration syntax.
- [ ] Define component children / slot model.
- [ ] Define component event/callback prop typing.
- [ ] Define whether multiple components may be declared in one `.voil` file.

### P1 — container and CSS integration

- [ ] Define `container` as concrete block/layout container and default lowering behavior.
- [ ] Decide whether `container(...)` accepts CSS-like arguments, typed layout arguments, or both.
- [ ] Decide whether styles are inline, separate `.voil` blocks, imported CSS, or a combination.
- [ ] Define component-local style scoping policy.
- [ ] Define CSS custom property support.
- [ ] Define pseudo class/element support.
- [ ] Define media query and container query support.
- [ ] Define animation/keyframe support.
- [ ] Define modern CSS forward-compatibility policy.
- [ ] Define raw/native CSS escape hatch.
- [ ] Define when compiler wrapper elimination is semantically safe.

### P1 — core language surface

- [ ] Decide whether `T?` is accepted shorthand for `Option<T>`.
- [ ] Finalize struct construction syntax.
- [ ] Finalize enum pattern syntax.
- [ ] Define function return inference policy.
- [ ] Define async/error syntax.
- [ ] Define exhaustive match diagnostics.
- [ ] Define array/indexing bounds behavior.

### P1 — routes

- [ ] Finalize static/dynamic route filename grammar.
- [ ] Define `param` conversion failure behavior.
- [ ] Define `_layout.voil`, `_404.voil`, `_error.voil` semantics.
- [ ] Define typed route-link API so generated URLs cannot omit required params.

### P1 — safety model

- [ ] Define `TrustedHtml` boundary.
- [ ] Define URL-related nominal types or standard-library wrappers.
- [ ] Define JS/npm import validation boundary.
- [ ] Define `Result<T, E>` and async/exception interoperability.

### P2 — module and ecosystem surface

- [ ] Finalize default/named/namespace import forms.
- [ ] Define project-root import convention without requiring config.
- [ ] Define npm package import semantics.
- [ ] Define native Web API exposure strategy.

## Compiler preparation

Do not begin compiler implementation until the active P0 syntax decisions are resolved or covered by an explicit parser experiment branch.

When implementation begins, add or expand planning documents for:

- `compiler.md`
- `type-system.md`
- `routing.md`
- `runtime.md`
- `security.md`
- `known-issues.md`

## Branch / merge policy

- Every implementation or specification change uses a dedicated branch.
- `main` merges use squash commit.
- Each implementation branch updates the affected `AI-build` documents before merge.
