# Voiles TODO

## Current phase

`v0.1 language baseline accepted; compiler bootstrap Milestone 1A semantic HIR implementation verified.`

Current branch: `implementation/semantic-hir-bootstrap`.

任何 merge 到 `main` 都使用 squash commit；未經明確授權不合併 `main`。

## Completed — Milestone 0A lexer bootstrap

- [x] Create implementation branch from `planning/syntax-spec`.
- [x] Initialize Rust 2024 virtual workspace with Cargo resolver 3.
- [x] Add `crates/voiles-lexer` with zero third-party dependencies.
- [x] Add byte source `Span` model.
- [x] Add stable lexical `Diagnostic` model.
- [x] Add keyword/operator/trivia token model.
- [x] Preserve whitespace/comment/non-structural physical line breaks for lossless CST handoff.
- [x] Implement Python-style indentation stack and synthetic `INDENT` / `DEDENT`.
- [x] Implement alternate-column tabs/spaces ambiguity diagnostics.
- [x] Implement bracket continuation for `()`, `[]`, `{}`.
- [x] Implement synthetic final logical `NEWLINE` + EOF dedent behavior.
- [x] Implement initial identifier, decimal number, string, punctuation/operator lexing.
- [x] Implement delimiter mismatch/unclosed diagnostics.
- [x] Author lexer unit tests for indentation, blank/comment lines, continuation, EOF, tab ambiguity, invalid dedent, trivia preservation, unclosed delimiter.
- [x] Run `cargo fmt --all -- --check` on GitHub Actions.
- [x] Run `cargo check --workspace` on GitHub Actions.
- [x] Run `cargo test --workspace` on GitHub Actions.
- [x] Run `cargo clippy --workspace --all-targets -- -D warnings` on GitHub Actions.
- [x] Fix the initial rustfmt CI failure and obtain a fully green compiler CI run.

Verification: GitHub Actions run `34813462150` at `7b7485fb035e870e0816306dc2c113f04289a2df` passed fmt/check/test/clippy. The local execution container still lacks Rust, so CI remains the executable verification environment for implementation branches.

## Completed — Milestone 0B lossless CST/parser

Implementation is present on `implementation/lossless-cst-parser` and the parser/CST verification gate has passed.

- [x] Create `crates/voiles-syntax` after re-reading the current compiler/grammar contracts.
- [x] Keep the parser/CST bootstrap zero-dependency; no external parser crate was required.
- [x] Parse module/items while retaining lexer trivia/source spans.
- [x] Parse bindings, imports/exports, `fn`, component declarations/calls.
- [x] Parse accepted expression precedence and named/positional call rules.
- [x] Parse `if/else`, `for/in/key`, struct/enum/match baseline.
- [x] Parse structural HTML blocks and attributes.
- [x] Parse `mount/init/cleanup` lifecycle blocks.
- [x] Parse slot outlet/provision syntax.
- [x] Parse typed route `param name: Type` declarations.
- [x] Add source round-trip and CST descendant span/count helpers for tests/tooling.
- [x] Define deterministic recovery tests for malformed indentation/incomplete blocks.
- [x] Correct parser fixtures so reserved keywords such as `state` are not used as ordinary identifiers.
- [x] Run final 0B fmt/check/test/clippy gates.

Verification: final 0B GitHub Actions run `34816974670` at `9da619178c9ce2d5f337a057fabba732f485121a` passed fmt/check/test/clippy.

## Completed — Milestone 1A syntax AST + semantic HIR

Implementation is present on `implementation/semantic-hir-bootstrap`. The implementation-code state was verified before documentation closeout.

- [x] Add zero-copy typed AST façade over lossless CST.
- [x] Add direct CST child/token traversal for semantic passes.
- [x] Create zero-third-party `crates/voiles-hir`.
- [x] Add stable scope/symbol/reference/capture model with `ScopeId` / `SymbolId`.
- [x] Resolve nearest lexical binding and preserve shadowing semantics.
- [x] Diagnose same-scope duplicate declarations (`VHIR001`).
- [x] Diagnose use-before-declaration with pending declaration activation (`VHIR002`).
- [x] Diagnose unresolved lexical names (`VHIR003`).
- [x] Diagnose direct assignment to immutable bindings (`VHIR004`).
- [x] Enforce `shared` as module-top-level-only (`VHIR005`).
- [x] Resolve function/component parameters left-to-right so defaults may only see earlier parameters.
- [x] Model function/component/block/loop/match-arm/lifecycle scopes.
- [x] Discover nested function closure captures without claiming escape analysis is complete.
- [x] Validate `init` / `mount` / nested `cleanup` lexical ownership (`VHIR006`).
- [x] Defer direct UI head component-vs-html identity while resolving lexical names in arguments/body.
- [x] Add 9 semantic HIR behavior tests.
- [x] Pass implementation-code fmt/check/test/clippy gates.

Verification: GitHub Actions run `34819797481` at `4551b961b89a69ead482e49fd7117d6064d65e43` passed fmt/check/test/clippy. Final documentation commits must retain the same gates before the milestone branch is considered closed.

## Next — Milestone 1B module graph + type checker bootstrap

- [ ] Create explicit module source identity used by semantic graph passes.
- [ ] Extract automatic component exports and explicit non-component exports.
- [ ] Resolve named and namespace `.voil` imports against module export tables.
- [ ] Distinguish runtime dependency edges from type-only dependency edges.
- [ ] Reject runtime dependency cycles while allowing valid type-only cycles.
- [ ] Resolve type-reference identity against local/imported/generic type namespaces.
- [ ] Create `crates/voiles-types` only when the HIR module/type identity contract is ready.
- [ ] Type-check primitive literals and explicit primitive references.
- [ ] Type-check lexical binding initializers/assignments using existing resolved `SymbolId`.
- [ ] Type-check first function signatures/calls.
- [ ] Keep component/slot/html metadata validation out until imported/type identities are available.
- [ ] Add cross-module and first type-checker test fixtures.
- [ ] Pass fmt/check/test/clippy gates on the final 1B branch head.

## Resolved language baseline — lexical / parser

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

## Resolved language baseline — binding / scope / lifetime

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

## Resolved language baseline — type/core language

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

## Resolved language baseline — component

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

## Resolved language baseline — HTML / container

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

## Resolved language baseline — module API / interop

- [x] Component export automatic; non-component export explicit with `export`.
- [x] Named imports share list + `as` syntax.
- [x] Namespace import syntax: `import * as name from "..."`.
- [x] No default export in v0.1.
- [x] JS/npm foreign modules use explicit `extern import` boundary.
- [x] Opaque default foreign type is `JsValue`.
- [x] No unconstrained `any` in v0.1.
- [x] JS `undefined` normalizes to Option/none at typed boundary.
- [x] JS throw/Promise rejection normalize to explicit error results.

## Resolved language baseline — route

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

## Resolved language baseline — safety

- [x] Plain `String` never equals trusted HTML.
- [x] `TrustedHtml` nominal boundary.
- [x] Sanitizer path for runtime HTML.
- [x] Unsafe raw conversion, if exposed, must be explicit.
- [x] Dynamic URL sinks use nominal safe URL type.
- [x] Compile-time URL literals are scheme-validated.
- [x] Dangerous URL schemes rejected by safe parser.

## Resolved language baseline — compiler architecture / HMR semantics

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

These are accepted semantic requirements whose exact implementation still requires prototype work:

- [ ] Owner-bound closure escape-analysis algorithm/diagnostics. Capture discovery is implemented in 1A; lifetime escape rejection is not.
- [ ] Effect/resource classification for top-level initializer and shared-resource validation.
- [ ] Cross-module runtime-vs-type dependency edge classification and cycle diagnostics.
- [ ] Exact parser recovery heuristics for pathological malformed nesting.
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

## Branch / merge gate

- compiler implementation is active only on a dedicated implementation branch;
- consult official documentation before adding every external package/dependency;
- use current project-compatible dependency versions;
- do not merge to `main` without explicit authorization;
- any merge to `main` must use squash commit.
