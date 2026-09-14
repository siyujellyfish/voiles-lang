# Voiles Compiler Implementation

Status: `Implementation active; Milestone 1A semantic HIR bootstrap implemented`.

Branch: `implementation/semantic-hir-bootstrap`

## 1. Toolchain baseline

The compiler is implemented as a Rust workspace using Rust 2024 edition semantics. The virtual workspace uses Cargo resolver `3`.

Current implementation intentionally uses no third-party crates. External dependencies must not be added until their current official documentation has been reviewed and their version is confirmed compatible with the project.

## 2. Planned compiler pipeline

```text
.voil source
-> voiles-lexer
-> lossless CST parser
-> syntax AST façade
-> resolved HIR
-> typed HIR
-> reactivity / identity / lifecycle lowering IR
-> HTML / CSS / JavaScript codegen
```

Current crate direction:

```text
crates/voiles-lexer      # Milestone 0A verified
crates/voiles-syntax     # Milestone 0B parser/CST verified; AST façade active
crates/voiles-hir        # Milestone 1A single-module semantic resolution active
crates/voiles-types      # Milestone 1B planned
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

`crates/voiles-syntax` provides the zero-dependency lossless CST/parser baseline.

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

Parser responsibility remains syntactic only. Name/component/slot resolution, scope legality, component API validation, type checking and HTML metadata validation stay in later semantic/type stages.

Reserved keywords remain lexer keywords and are not accepted as ordinary identifiers.

## 6. Milestone 1A — syntax AST façade + semantic HIR

The current branch adds a zero-copy typed AST façade over the CST and `crates/voiles-hir` for single-module lexical/name resolution.

Implemented AST/HIR surface:

- typed CST wrappers for modules, declarations, parameters, blocks, lifecycle blocks and name expressions;
- direct CST child/token traversal without losing lossless source representation;
- explicit scope graph and stable `ScopeId` / `SymbolId` identities;
- nearest lexical binding resolution and shadowing;
- same-scope duplicate declaration diagnostics;
- use-before-declaration diagnostics using pending declaration activation points;
- unresolved-name diagnostics;
- mutability validation for direct assignment to immutable bindings;
- `shared` declaration module-top-level validation;
- left-to-right function/component parameter default resolution;
- function/component/loop/match-arm/block/lifecycle scopes;
- nested function closure capture discovery;
- lifecycle lexical-owner validation;
- semantic references/captures retained in HIR model for later passes;
- direct UI head identity remains deferred to component/html metadata resolution while lexical names in arguments and child bodies are resolved now.

Semantic diagnostics introduced by 1A:

```text
VHIR001  duplicate declaration in same scope
VHIR002  use before declaration
VHIR003  unresolved name
VHIR004  assignment to immutable binding
VHIR005  shared declaration outside module top level
VHIR006  lifecycle block in invalid lexical owner
```

Milestone 1A intentionally does not claim the following are implemented:

- cross-module import/export symbol identity;
- runtime import DAG/cycle analysis;
- owner-bound closure escape analysis;
- type resolution or type checking;
- component argument/slot API validation;
- html-base structural/event metadata resolution;
- reactive/runtime lowering.

These boundaries belong to 1B or later passes.

## 7. Verification gate

Required workspace verification:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Milestone 0A lexer code was verified successfully on GitHub Actions run `34813462150` at commit `7b7485fb035e870e0816306dc2c113f04289a2df`.

Milestone 0B final parser/CST branch head was verified successfully on GitHub Actions run `34816974670` at commit `9da619178c9ce2d5f337a057fabba732f485121a`.

Milestone 1A implementation code was verified successfully on GitHub Actions run `34819797481` at commit `4551b961b89a69ead482e49fd7117d6064d65e43`; fmt, check, test and clippy all passed. The final documentation head must keep the same four gates green before the branch is considered closed.

The local execution container used during implementation still has no Rust toolchain. GitHub Actions remains the executable verification environment for implementation branches.

## 8. Next — Milestone 1B

Recommended next slice:

1. represent module source identity and runtime/type dependency edges;
2. resolve named/namespace imports against exported symbol tables;
3. validate component automatic exports and explicit non-component exports;
4. reject runtime `.voil` dependency cycles while allowing type-only cycles;
5. establish resolved type references over HIR;
6. bootstrap primitive/literal/binding/function type checking in `voiles-types`;
7. preserve current stable diagnostic + CI gates.

Component/slot/html metadata validation can follow once cross-module symbol and type identity are available.
