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
- source line/column indexing is not yet implemented; spans are byte offsets;
- no CST/parser exists yet.

## Verification status

The local implementation container did not have Rust installed and external DNS prevented installing official stable Rust through rustup. Executable verification was therefore performed on GitHub Actions.

Run `34813462150` at commit `7b7485fb035e870e0816306dc2c113f04289a2df` passed:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

The lexer unit-test suite passed in that run. Subsequent changes must keep the same CI gates green; a later failure is blocking even if this recorded run was successful.

## Future prototype risks

- closure owner-lifetime escape analysis;
- effect/resource classification for `init` and `shared` validation;
- CST recovery after malformed indentation/incomplete blocks;
- exact foreign binding generation and async callback ABI;
- html-base metadata generation/versioning;
- HMR compatibility fingerprint/invalidation graph;
- CSS scoping/cascade design.
