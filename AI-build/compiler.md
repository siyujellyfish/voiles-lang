# Voiles Compiler Implementation

Status: `Implementation started`.

Branch: `implementation/compiler-bootstrap`

## 1. Toolchain baseline

The compiler is implemented as a Rust workspace using Rust 2024 edition semantics. The virtual workspace uses Cargo resolver `3`.

Current implementation intentionally uses no third-party crates. External dependencies must not be added until their current official documentation has been reviewed and their version is confirmed compatible with the project.

## 2. Planned compiler pipeline

```text
.voil source
-> voiles-lexer
-> lossless CST parser
-> syntax AST
-> resolved HIR
-> typed HIR
-> reactivity / identity / lifecycle lowering IR
-> HTML / CSS / JavaScript codegen
```

Initial crate direction:

```text
crates/voiles-lexer      # active
crates/voiles-syntax     # next: CST/parser
crates/voiles-hir        # later
crates/voiles-types      # later
crates/voiles-lowering   # later
crates/voiles-codegen    # later
crates/voiles-cli        # later
```

Only crates needed by the current milestone should be created.

## 3. Milestone 0A — lexer

Current implementation target:

- UTF-8 source accepted as Rust `&str`;
- byte-offset `Span` for every token/diagnostic;
- comments and whitespace preserved as tokens;
- structural `NEWLINE`, synthetic `INDENT` / `DEDENT`, and non-structural physical `LineBreak` are distinct;
- Python-style indentation stack with 8-column tab stops;
- alternate indentation column tracking detects ambiguous tabs/spaces;
- blank/comment-only lines do not affect block indentation;
- bracket continuation for `()`, `[]`, `{}` suppresses structural newline/indentation;
- EOF emits a synthetic logical newline when needed and closes remaining indentation levels;
- current v0.1 keywords/operators are tokenized;
- lexical diagnostics have stable `VLEXxxx` codes.

Current lexer diagnostic codes:

```text
VLEX001  dedent does not match an outer indentation level
VLEX002  ambiguous tabs/spaces indentation
VLEX003  unexpected character
VLEX004  unterminated string literal
VLEX005  unexpected/mismatched closing delimiter
VLEX006  unclosed delimiter
```

## 4. Lossless token contract

Source bytes must be reconstructable from source-backed tokens plus synthetic structural tokens:

- `Whitespace`, `Comment`, `LineBreak`, ordinary lexical tokens use non-empty source spans where applicable;
- `Indent` / `Dedent` may use zero-width spans at the first code byte of a logical line;
- synthetic EOF `Newline` and `Eof` use zero-width EOF spans;
- CST must never depend on discarding comments/trivia in the lexer.

`LineBreak` exists specifically so blank lines and continuation newlines can remain lossless without pretending they are parser-significant `Newline` tokens.

## 5. Parser handoff contract

The next parser crate should consume lexer output without re-deriving indentation. Parser recovery may continue after a lexer diagnostic, but must not reinterpret invalid indentation as valid source silently.

First parser slice should cover:

```text
module
binding declarations
fn declarations
imports / exports
basic expressions + accepted precedence
if / else
for / in / key
component declaration/call
structural blocks
mount/init/cleanup
slot syntax
```

The parser should produce a lossless CST before semantic resolution begins.

## 6. Verification gate

Before this branch is proposed for merge, a Rust-capable environment must run at minimum:

```text
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

This implementation session could not execute those commands because the available execution container had no Rust toolchain and external DNS prevented installing official stable Rust. Therefore repository code must be treated as source-reviewed but not yet toolchain-verified until the commands above pass.

## 7. Non-goals for this bootstrap

Not implemented in the current lexer slice:

- CST/parser;
- semantic name resolution;
- type checking;
- reactive dependency lowering;
- component/runtime lowering;
- browser code generation;
- Vite integration;
- CSS integration.
