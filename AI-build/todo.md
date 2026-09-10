# Voiles TODO

## Current phase: syntax specification

### P0 — parser-blocking decisions

- [ ] Decide whether indentation is syntax-significant.
- [ ] Decide whether block openers require `:`.
- [ ] Decide attribute/named-argument separator (`:` vs `=`).
- [ ] Define unambiguous grammar for native View node vs component invocation.
- [ ] Define tokenization rules for indentation, newline and multiline expression continuation.
- [ ] Define initial expression precedence table.
- [ ] Define syntax error recovery goals for malformed View/Style blocks.

### P1 — core language surface

- [ ] Finalize `let` / `var` / `const` / `state` semantics.
- [ ] Decide whether `T?` is accepted shorthand for `Option<T>`.
- [ ] Finalize struct construction syntax.
- [ ] Finalize enum pattern syntax.
- [ ] Define function return inference policy.
- [ ] Define event handler syntax.
- [ ] Decide whether two-way binding exists.
- [ ] Define component children / slot model.
- [ ] Define dynamic class syntax.

### P1 — routes

- [ ] Finalize static/dynamic route filename grammar.
- [ ] Define `param` conversion failure behavior.
- [ ] Define `_layout.voil`, `_404.voil`, `_error.voil` semantics.
- [ ] Define typed route-link API so generated URLs cannot omit required params.

### P1 — style language

- [ ] Decide default component style scoping.
- [ ] Define selector grammar supported by typed Style IR.
- [ ] Define property/value parser boundaries.
- [ ] Define raw CSS escape hatch.
- [ ] Define modern CSS compatibility policy for new properties/functions/at-rules.
- [ ] Decide how `stack` / `row` / `grid` / `layer` attach layout styles and when wrappers may be removed.

### P1 — safety model

- [ ] Define `TrustedHtml` boundary.
- [ ] Define URL-related nominal types or standard-library wrappers.
- [ ] Define JS/npm import validation boundary.
- [ ] Define `Result<T, E>` and async/exception interoperability.
- [ ] Define array/indexing bounds behavior.
- [ ] Define exhaustive match diagnostics.

### P2 — module and ecosystem surface

- [ ] Finalize `.voil` import syntax.
- [ ] Define project-root import convention without requiring config.
- [ ] Define npm package import syntax.
- [ ] Define native Web API exposure strategy.

## Compiler preparation

Do not begin compiler implementation until P0 syntax decisions are resolved or covered by an explicit parser experiment branch.

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
