# Voiles Known Issues / Bootstrap Limitations

Status: `Active implementation notes`.

## Lexer implementation limitations

The first `voiles-lexer` slice intentionally implements only the grammar needed to validate block/token architecture.

Current known limitations:

- identifier classification uses Rust Unicode `is_alphabetic` / `is_alphanumeric`; exact Unicode identifier/XID and normalization policy is not yet specified;
- numeric literals currently cover decimal integer/float/exponent scanning only; radix literals and stricter underscore validation are not implemented;
- string scanning currently supports single/double quoted single-line strings and escape skipping, but no raw/multiline/interpolated string model;
- Python-style form-feed behavior in leading indentation is not implemented;
- invalid-dedent recovery is provisional and is intended to reduce diagnostic cascades, not define valid syntax;
- delimiter mismatch recovery is provisional;
- source line/column indexing is not yet implemented; spans are byte offsets.

## Parser / syntax limitations

The lossless CST/parser and typed AST façade exist, but syntax tooling remains bootstrap-level:

- parser recovery heuristics are deterministic but still provisional for heavily malformed nested blocks;
- typed AST wrappers currently cover the declarations/control-flow nodes needed by semantic bootstrap rather than every CST node;
- direct UI head syntax does not by itself distinguish user component vs html-base structural element; that identity remains a later semantic/metadata decision;
- no formatter implementation exists yet despite CST retaining formatter-relevant trivia.

## Semantic HIR limitations

Milestone 1A implements single-module lexical/name resolution only.

Implemented now:

- lexical scopes and stable symbol/reference identities;
- nearest-binding resolution and shadowing;
- same-scope duplicate/use-before/unresolved diagnostics;
- direct binding mutability validation;
- `shared` lexical-owner validation;
- left-to-right parameter defaults;
- closure capture discovery;
- lifecycle lexical-owner checks.

Still intentionally unresolved:

- cross-module import/export target identity;
- runtime dependency DAG/cycle validation;
- type-reference resolution and type checking;
- owner-bound closure escape/lifetime analysis;
- effect/resource classification for top-level/shared initializer validation;
- component call/slot API validation;
- html-base structural/event metadata resolution.

Closure capture discovery must not be confused with completed closure lifetime safety: 1A records capture relationships, but a later owner-lifetime escape pass must still reject values that outlive component/module owners.

## Verification status

The local implementation container does not provide the Rust toolchain used for executable gates, so GitHub Actions remains the authoritative execution environment for implementation branches.

Recorded fully green implementation states:

```text
Milestone 0A
run    34813462150
commit 7b7485fb035e870e0816306dc2c113f04289a2df

Milestone 0B final head
run    34816974670
commit 9da619178c9ce2d5f337a057fabba732f485121a

Milestone 1A implementation code
run    34819797481
commit 4551b961b89a69ead482e49fd7117d6064d65e43
```

Each recorded run passed:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

The final 1A documentation head must also keep these gates green before the milestone branch is considered closed.

## Future prototype risks

- cross-module source identity/resolution and runtime-vs-type dependency graph classification;
- closure owner-lifetime escape analysis;
- effect/resource classification for `init` and `shared` validation;
- exact parser recovery behavior for pathological malformed nesting;
- exact foreign binding generation and async callback ABI;
- html-base metadata generation/versioning;
- HMR compatibility fingerprint/invalidation graph;
- CSS scoping/cascade design.
