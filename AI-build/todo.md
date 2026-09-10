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
- [x] Same importer scope + same resolved `.voil` path resolves to the same module instance.
- [x] Different import aliases in the same importer scope do not clone module state.
- [x] Declare cross-module-instance sharing on the variable with `shared`, not on the module/import.
- [x] `shared` is mutable by definition and does not require an additional `state` keyword.
- [x] Restrict `shared` to module top-level only.
- [x] Every component invocation creates an independent component instance scope.
- [x] Component-local ordinary `state` is isolated per invocation identity.
- [x] Ordinary scoped modules imported by a component are also isolated per component instance; `shared` remains global.
- [x] Declare components explicitly with `component Name(...):`.
- [x] Allow multiple component declarations in one `.voil` module.
- [x] Automatically expose every top-level component declaration without requiring `export` syntax.
- [x] Treat unreachable component declarations as compiler tree-shaking / dead-code-elimination candidates.
- [x] Import multiple component symbols with a comma-separated list: `import A, B from "./c.voil"`.
- [x] Use component invocation child blocks as default slot content.
- [x] Use compiler-level `slot` outlets inside component declarations.
- [x] Support named slot provision with `slot Name:` blocks.
- [x] Preserve caller lexical scope for default and named slot content.
- [x] Make slot outlets optional by default in v0.1.
- [x] Permit at most one default outlet and one outlet per named slot in a component.
- [x] Permit each named slot to be provided at most once per invocation.
- [x] Reject unknown named slots and default children when no default outlet exists.
- [x] Make component parameters immutable input bindings.
- [x] Make component invocation named-argument-only.
- [x] Evaluate component parameter defaults per invocation from left to right.
- [x] Allow parameter defaults to reference only earlier parameters.
- [x] Preserve the same component instance and local state when reactive parameter values change.
- [x] Treat `state local = parameter` as instance initialization rather than automatic two-way synchronization.
- [x] Use structural invocation position as component identity outside repeated UI.
- [x] Treat conditional branch removal as component unmount/state release; re-entry creates a new instance.
- [x] Require explicit `key` for repeated UI that instantiates components.
- [x] Do not use implicit list index/position as component identity.
- [x] Preserve keyed component instances across reorder.
- [x] Treat removed/changed keys as identity removal/replacement.
- [x] Restrict v0.1 component keys to `String` / `Int`.
- [x] Treat duplicate runtime keys as deterministic runtime errors.
- [x] Use `mount:` once per mounted component instance.
- [x] Use nested `cleanup:` for component-owned resource teardown.
- [x] Permit cleanup to capture enclosing mount lexical bindings.
- [x] Do not rerun mount/cleanup for reactive prop/state updates or unchanged-key reorder.
- [x] Run cleanup on conditional removal and keyed identity removal/replacement.
- [x] Defer reactive effect/dependency-array semantics beyond the v0.1 lifecycle baseline.

### P0 — parser-blocking decisions

- [ ] Define exact `NEWLINE` / `INDENT` / `DEDENT` tokenization rules.
- [ ] Define multiline parenthesized expression continuation rules.
- [ ] Confirm named argument separator `=` across ordinary functions and struct construction; component invocation already uses named-only `=` arguments.
- [ ] Define unambiguous grammar for structural HTML block vs ordinary block/call.
- [x] Define component declaration grammar baseline: `component PascalName(parameters):`.
- [x] Define component import-list grammar baseline: `import A, B from "..."`.
- [ ] Finalize component import alias grammar; current recommendation: item-local `as`.
- [x] Define component child-block/default-slot grammar baseline.
- [x] Define named slot block / slot outlet grammar baseline.
- [x] Define slot cardinality validation baseline.
- [x] Define component call arguments as named-only.
- [x] Define keyed repeated-component UI grammar baseline: `for item in items key expression:`.
- [x] Define component lifecycle block baseline: `mount:` with nested `cleanup:`.
- [ ] Define initial expression precedence table.
- [ ] Define syntax error recovery for malformed indentation and unfinished blocks.

### P0 — binding / state model

- [ ] Finalize `const` semantics as immutable runtime binding.
- [ ] Finalize `state` semantics as mutable scoped binding with compiler-generated reactivity when observed.
- [ ] Finalize `shared` semantics as mutable storage shared across declaring module/component instances.
- [x] Restrict `shared` to module top-level.
- [x] Define component invocation state identity as independent per invocation identity.
- [x] Treat each component instance as an importer scope for ordinary scoped `.voil` dependencies.
- [x] Preserve caller lexical scope through component slot projection.
- [x] Preserve component instance/local state across reactive parameter updates.
- [x] Define structural component identity for non-repeated UI.
- [x] Define conditional branch removal/re-entry component lifetime.
- [x] Define keyed repeated-component identity and duplicate-key behavior.
- [x] Define component `mount` / nested `cleanup` lifecycle semantics.
- [ ] Decide whether v0.1 removes `let` / `var` entirely.
- [ ] Evaluate whether function-local non-reactive mutation requires `mut`, or whether unobserved local `state` lowers to ordinary mutable storage.
- [ ] Define closure capture rules for `const` / `state`.
- [ ] Define scoped module cleanup when importer/component instances become unreachable.
- [ ] Define cyclic import initialization for scoped modules and `shared` bindings.

### P1 — standard HTML surface

- [ ] Define minimum `@voiles/html-base` API.
- [ ] Define canonical `std` alias policy: convention vs reserved language namespace.
- [ ] Define the boundary between syntax-level structural blocks (`main:`, `header:`) and `std.*` elements.
- [ ] Define typed HTML attributes.
- [ ] Define native event handler types.
- [ ] Define HTML escaping behavior.
- [ ] Define boolean attribute lowering.

### P1 — components

- [x] Define explicit component declaration model.
- [x] Define automatic component export model with no `export` keyword.
- [x] Allow multiple component declarations per `.voil` module.
- [x] Define unused component declarations as tree-shaking candidates.
- [x] Define multi-component import list syntax.
- [x] Define default/named slot surface syntax.
- [x] Define slot lexical-scope ownership as caller-side.
- [x] Define v0.1 slot cardinality: optional, single outlet/provision, unknown-slot rejection.
- [x] Finalize component parameter / prop semantics: immutable, named-only, per-invocation defaults, left-to-right initialization.
- [x] Define reactive prop updates as same-instance updates that preserve local state.
- [x] Define structural identity for static component call sites.
- [x] Define conditional unmount/re-entry instance behavior.
- [x] Require explicit keys for repeated component UI and preserve instances by key.
- [x] Define v0.1 key type and duplicate-key runtime validation.
- [x] Define component mount/cleanup API semantics.
- [ ] Finalize component import alias syntax (`as` currently recommended).
- [ ] Define slot parameter / scoped-slot model if needed.
- [ ] Define slot content type model.
- [ ] Define component event/callback prop typing.
- [ ] Define module top-level side-effect rules so whole-module tree-shaking behavior is deterministic.

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

- [ ] Finalize component alias form.
- [ ] Finalize non-component default/named/namespace import forms.
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