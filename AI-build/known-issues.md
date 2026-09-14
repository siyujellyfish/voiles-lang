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

The implementation environment used for this bootstrap did not have Rust installed, and external DNS prevented installing official stable Rust through rustup. Therefore the new Rust source has not yet been compiled or executed in this session.

Before merge/progression beyond lexer bootstrap, run:

```text
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Any compiler error or failed test from that verification is a blocking issue and must be corrected on the implementation branch.

## Future prototype risks

- closure owner-lifetime escape analysis;
- effect/resource classification for `init` and `shared` validation;
- CST recovery after malformed indentation/incomplete blocks;
- exact foreign binding generation and async callback ABI;
- html-base metadata generation/versioning;
- HMR compatibility fingerprint/invalidation graph;
- CSS scoping/cascade design.
