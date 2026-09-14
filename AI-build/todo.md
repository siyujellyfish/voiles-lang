# Voiles TODO

## Current phase

`v0.1 language baseline accepted; compiler implementation not started.`

本 branch 仍是 `planning/syntax-spec`。開始 compiler/runtime/Vite 實作前必須建立新的 implementation branch；任何 merge 到 `main` 都使用 squash commit。

## Resolved — lexical / parser baseline

- [x] `:` + significant indentation block model.
- [x] Python-style logical line / indentation-stack `NEWLINE` / `INDENT` / `DEDENT` semantics.
- [x] Blank/comment-only lines do not emit structural indentation tokens.
- [x] Bracket-context implicit multiline continuation for `()`, `[]`, `{}`.
- [x] Continuation indentation is non-structural.
- [x] v0.1 does not require backslash explicit line joining.
- [x] `#` line comments only.
- [x] Quoted module specifiers.
- [x] `fn` function declarations.
- [x] Initial expression precedence.
- [x] `and/or/not` and `&&/||/!` accepted as equivalent short-circuit operators.
- [x] General named argument separator is `=`.
- [x] Function calls permit positional then named; no positional after named.
- [x] Component calls named-only.
- [x] Struct construction named-only.
- [x] Import alias uses item-local `as`.
- [x] Structural HTML block baseline + attributes.
- [x] Component declaration/import/slot/key/lifecycle grammar baselines.
- [x] Module `init` / nested `cleanup` grammar baseline.
- [x] Source-span/error-recovery requirements defined for compiler diagnostics.

## Resolved — binding / scope / lifetime

- [x] Finalize binding model as `const / state / shared`.
- [x] Remove `let / var / mut` from v0.1.
- [x] Use `state` for all ordinary mutation; unobserved local state may lower to plain mutable storage.
- [x] Generate reactive tracking only when a `state/shared` observer exists.
- [x] Lexical nearest-binding lookup + shadowing.
- [x] Same-scope redeclaration/use-before-declaration errors.
- [x] Support closure capture.
- [x] Captured `state` is shared by cell among closures from the same invocation.
- [x] Escaping function-local closures extend captured function environment lifetime.
- [x] Reject owner-bound component/module closure escape beyond owner lifetime.
- [x] `shared` module-top-level only.
- [x] Scoped module identity = importer scope + resolved path.
- [x] Same-scope aliases do not clone module instances.
- [x] Scoped module `init/cleanup` lifecycle.
- [x] Dependency-first/post-order scoped-module teardown.
- [x] Runtime `.voil` import cycles are compile errors in v0.1.
- [x] Type-only dependency cycles may exist without runtime ownership edges.
- [x] Observable module-top-level work must live in `init:`.
- [x] `shared` initializer cannot directly own cleanup-requiring external resources.

## Resolved — type/core language baseline

- [x] `T? == Option<T>`.
- [x] `none` as empty option literal.
- [x] No implicit nullable reference / implicit Option truthiness.
- [x] Immutable struct fields in v0.1.
- [x] Struct named construction + defaults + required/unknown/duplicate validation.
- [x] Enum declaration + payload cases.
- [x] Exhaustive `match` + wildcard `_` + nested patterns.
- [x] Pattern guards deferred from v0.1.
- [x] Function type syntax `fn(...) -> T`.
- [x] Async/error baseline: `async fn`, `await`, `Result<T,E>`, `?` propagation.
- [x] No ordinary `throw` as core v0.1 control flow.
- [x] List `[]` checked indexing + `.get()` optional indexing.
- [x] Negative index is out-of-bounds, not Python reverse indexing.

## Resolved — component baseline

- [x] Explicit `component Name(...):` declarations.
- [x] Multiple components per module.
- [x] Automatic component export; no `export component`.
- [x] Component import list and item-local `as` alias.
- [x] Immutable component parameters.
- [x] Named-only component invocation.
- [x] Per-instance left-to-right component parameter defaults.
- [x] Reactive prop update preserves component identity/local state.
- [x] Structural component identity outside repeated UI.
- [x] Conditional branch exit unmounts; re-entry creates a new instance.
- [x] Repeated component UI requires explicit `String | Int` key.
- [x] No implicit index identity.
- [x] Duplicate runtime key is deterministic runtime error.
- [x] `mount:` + nested `cleanup:` lifecycle.
- [x] Owned module dependencies teardown before component cleanup.
- [x] Module-level `state` remains legal in modules that declare components.
- [x] Default/named slot syntax + caller lexical scope.
- [x] v0.1 optional/single slot cardinality.
- [x] Component callback/function type baseline.
- [x] DOM event types supplied by html-base metadata.
- [x] Scoped slot deferred.
- [x] First-class UI value type deferred.

## Resolved — HTML / container baseline

- [x] `std` is ordinary canonical alias, not reserved namespace.
- [x] Structural block/container-level HTML names use direct block surface.
- [x] Lower-level/leaf/content elements use `std.*`.
- [x] Structural blocks accept named attributes.
- [x] html-base metadata provides typed attributes/events.
- [x] `class` direct; `html_for` maps to HTML `for`.
- [x] `aria_*` / `data_*` attribute mapping.
- [x] Boolean attribute true/false lowering.
- [x] Text escaping baseline.
- [x] Void element child rejection.
- [x] `container` is concrete generic block node, default div-like lowering.
- [x] Wrapper elimination only with full semantic-preservation proof.

## Resolved — module API / interop baseline

- [x] Component export automatic; non-component export explicit with `export`.
- [x] Named imports share list + `as` syntax.
- [x] Namespace import syntax: `import * as name from "..."`.
- [x] No default export in v0.1.
- [x] JS/npm foreign modules use explicit `extern import` boundary.
- [x] Opaque default foreign type is `JsValue`.
- [x] No unconstrained `any` in v0.1.
- [x] JS `undefined` normalizes to Option/none at typed boundary.
- [x] JS throw/Promise rejection normalize to explicit error results.

## Resolved — route baseline

- [x] File-based static route grammar.
- [x] `[id]` dynamic segment.
- [x] `[...slug]` catch-all.
- [x] `[[...slug]]` optional catch-all.
- [x] Route conflicts are compile errors.
- [x] `_layout.voil`, `_404.voil`, `_error.voil` reserved files.
- [x] `param name: Type` typed path conversion.
- [x] Typed path-conversion failure enters nearest 404 resolution.
- [x] Catch-all default type `List<String>`.
- [x] Nested layout instance preserved inside subtree, cleaned when leaving subtree.
- [x] Nearest `_error` boundary and nearest `_404` fallback semantics.
- [x] Typed route-link API requirement.

## Resolved — safety baseline

- [x] Plain `String` never equals trusted HTML.
- [x] `TrustedHtml` nominal boundary.
- [x] Sanitizer path for runtime HTML.
- [x] Unsafe raw conversion, if exposed, must be explicit.
- [x] Dynamic URL sinks use nominal safe URL type.
- [x] Compile-time URL literals are scheme-validated.
- [x] Dangerous URL schemes rejected by safe parser.

## Resolved — compiler architecture / HMR semantics

- [x] Pipeline: tokens -> lossless CST -> AST -> resolved HIR -> typed HIR -> lowering IR -> HTML/CSS/JS.
- [x] CST preserves comments/trivia/source spans.
- [x] HIR owns symbol/type/module/component resolution.
- [x] Lowering IR owns reactivity/identity/lifecycle semantics.
- [x] Diagnostic requirements: stable code + primary/secondary spans + recovery.
- [x] HMR compatible body edit preserves component state.
- [x] Incompatible parameter/state/lifecycle shape remounts affected boundary.
- [x] Module init/dependency graph HMR change cleans old subtree before replacement init.
- [x] Compatible `shared` declaration identity/type may preserve cell across HMR.
- [x] Compile-error HMR keeps last successful graph running while surfacing diagnostics.
- [x] HMR stays dev-only semantics.

## Open — implementation/prototype decisions

These are no longer language-surface blockers but require prototype work before production implementation is considered stable:

- [ ] Exact closure escape-analysis algorithm/diagnostics.
- [ ] Effect/resource classification for top-level initializer and shared-resource validation.
- [ ] Exact parser recovery heuristics for malformed indentation/incomplete block.
- [ ] Foreign binding generator from `.d.ts`/schema and `JsValue` conversion APIs.
- [ ] Exact async callback ABI and browser Promise adapter.
- [ ] Application bootstrap/root special surface.
- [ ] HMR declaration fingerprint/compatibility hash and Vite invalidation propagation.
- [ ] HTML metadata generation/version policy.

## Open — CSS integration (separate design phase)

Native CSS compatibility remains priority; do not start a custom CSS DSL without a separate design pass.

- [ ] Native `.css` import model.
- [ ] `.voil` local style block surface, if any.
- [ ] Component-local scoping while preserving cascade.
- [ ] Custom properties.
- [ ] Pseudo classes/elements.
- [ ] Media/container queries.
- [ ] Keyframes/animations.
- [ ] Modern browser property forward compatibility.
- [ ] Typed CSS subset boundary.
- [ ] Raw/native CSS escape hatch.

## Deferred beyond v0.1 baseline

- [ ] Scoped slot / slot parameter.
- [ ] First-class `Ui` / `Node` / `Slot` values.
- [ ] Struct copy/update syntax sugar.
- [ ] Match guards.
- [ ] General-purpose macros.
- [ ] User operator overloading.
- [ ] Rust-style borrow/lifetime syntax.
- [ ] SSR-only syntax.
- [ ] Ecosystem/package policy finalization: project-root alias, Voiles package publication/metadata, final browser target, final Vite plugin contract.

## Compiler implementation gate

Language P0 design is now sufficiently specified to begin a parser/compiler prototype **after explicit authorization**.

When implementation begins:

- create a new implementation branch;
- add/update `AI-build/compiler.md`, `type-system.md`, `routing.md`, `runtime.md`, `security.md`, `known-issues.md` as implementation contracts;
- consult official documentation for every external package before selecting/using its version;
- use versions compatible with the project and current official releases;
- do not merge to `main` without explicit authorization;
- any merge to `main` must use squash commit.