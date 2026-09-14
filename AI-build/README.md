# AI-build

`AI-build` 是 Voiles 的持續規劃與實作紀錄區。所有功能性調整都必須同步更新此處相關文件，避免規格只存在於討論紀錄。

## 目前階段

- 專案：Voiles
- 原始碼副檔名：`.voil`
- v0.1 language baseline：Accepted
- 階段：compiler bootstrap / Milestone 1A syntax AST + semantic HIR
- 工作 branch：`implementation/semantic-hir-bootstrap`
- Rust workspace：2024 edition，Cargo resolver 3
- 目前 compiler code：`crates/voiles-lexer`、`crates/voiles-syntax`、`crates/voiles-hir`
- 目前 bootstrap 不使用第三方 crate
- Milestone 0A lexer：Verified
- Milestone 0B lossless CST/parser：Verified
- Milestone 1A single-module lexical/name resolution：Verified code state；final documentation head CI 仍需保持全綠
- 未經明確授權不合併 `main`

## 文件

- [`language-spec.md`](./language-spec.md)：v0.1 語言主規格、grammar baseline、route/safety/HMR/compiler representation。
- [`syntax-decisions.md`](./syntax-decisions.md)：Accepted/Open/Deferred 設計決策 registry。
- [`state-model.md`](./state-model.md)：`const` / `state` / `shared`、closure、scoped module、component identity/lifecycle。
- [`component-model.md`](./component-model.md)：component declaration/export/parameters/callback/identity/lifecycle/slots/tree-shaking。
- [`compiler.md`](./compiler.md)：compiler pipeline、lexer/parser/AST/HIR implementation contract 與驗證 gate。
- [`type-system.md`](./type-system.md)：type checker implementation contract 與 HIR handoff。
- [`runtime.md`](./runtime.md)：reactivity、identity、module/component lifecycle runtime contract。
- [`routing.md`](./routing.md)：file routing 與 typed route implementation contract。
- [`security.md`](./security.md)：TrustedHtml/URL/foreign/resource safety implementation contract。
- [`known-issues.md`](./known-issues.md)：bootstrap limitation、未驗證項目與 prototype risk。
- [`todo.md`](./todo.md)：目前實作進度與下一個 milestone。

## 更新規則

1. 實作或規格變更前先完整確認相關文件現況。
2. 每個功能 branch 同步更新受影響的 `AI-build` 文件。
3. 已定案內容標記為 `Accepted`；仍可能改動者標記為 `Draft`；需要獨立討論/prototype 者標記為 `Open`；刻意延後者標記為 `Deferred`。
4. 語法變更需同步更新範例、grammar 與 compiler TODO。
5. 新增外部 dependency 前先查閱該套件官方最新文件並確認 project-compatible version。
6. 實作一律建立新 branch。
7. 合併至 `main` 一律使用 squash commit。
