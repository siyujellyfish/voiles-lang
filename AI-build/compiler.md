# Voiles Compiler Implementation

Status: `Implementation started`.

Branch: `implementation/lossless-cst-parser`

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
crates/voiles-lexer      # active, Milestone 0A verified
crates/voiles-syntax     # active, Milestone 0B parser/CST
crates/voiles-hir        # later
crates/voiles-types      # later
crates/voiles-lowering   # later
crates/voiles-codegen    # later
crates/voiles-cli        # later
```

Only crates needed by the current milestone should be created.

## 3. Milestone 0A — lexer

Implemented contract:

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

## 5. Milestone 0B — lossless CST/parser

Current implementation on `implementation/lossless-cst-parser` adds `crates/voiles-syntax` without third-party dependencies.

Implemented parser/CST surface:

- lossless `SyntaxNode` / `SyntaxElement` tree retaining lexer tokens, trivia and source spans;
- source round-trip reconstruction from CST tokens;
- module/items, bindings, imports/exports, namespace imports and `extern import`;
- `fn` / `async fn`, typed/default parameters and return types;
- Pratt expression parser with accepted precedence, postfix calls/member/index/`?`, assignment and named/positional argument ordering;
- `if/else`, `for/in/key`, `return` and lifecycle blocks;
- `component`, child blocks, structural UI blocks and `slot` syntax;
- `struct`, generic `enum`, payload cases, `match` arms and patterns;
- typed route `param name: Type` declarations;
- deterministic error nodes and line/block recovery while retaining source tokens;
- CST descendant count/span helpers used by parser/tooling tests.

Parser responsibility remains syntactic only. Name/component/slot resolution, scope legality, component API validation, type checking and HTML metadata validation stay in later HIR/type-checking stages.

Reserved keywords remain lexer keywords and are not accepted as ordinary identifiers. Parser fixtures must therefore use non-keyword names; for example, a match function parameter uses `status` rather than the reserved binding keyword `state`.

## 6. Verification gate

Required workspace verification:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Milestone 0A lexer code was verified successfully on GitHub Actions run `34813462150` at commit `7b7485fb035e870e0816306dc2c113f04289a2df`. The run completed all four gates successfully, including the lexer unit-test suite.

Milestone 0B verification is in progress on `implementation/lossless-cst-parser`. Prior runs have confirmed parser compilation and most parser tests; the final branch head must pass all four gates before 0B is considered verified.

The local execution container used during implementation still has no Rust toolchain. GitHub Actions is therefore the current executable verification environment for this branch.

Any later implementation commit must keep the same CI gates green before the branch is considered verified.

## 7. Non-goals for the current parser bootstrap

Not implemented in Milestone 0B:

- syntax AST facade beyond the raw lossless CST;
- semantic name resolution;
- type checking;
- reactive dependency lowering;
- component/runtime lowering;
- browser code generation;
- Vite integration;
- CSS integration.
