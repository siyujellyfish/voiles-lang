# Voiles

Voiles is a work-in-progress front-end language using the `.voil` source extension.

The v0.1 language baseline is documented under [`AI-build`](./AI-build/). Compiler bootstrap implementation currently lives on dedicated implementation branches; the first Rust crate is `crates/voiles-lexer`.

## Compiler workspace

Use the current stable Rust toolchain selected by `rust-toolchain.toml`.

```text
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

No merge to `main` is performed without explicit authorization; project merges to `main` use squash commits.
