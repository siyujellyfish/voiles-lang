# AI-build

`AI-build` 是 Voiles 的持續規劃與實作紀錄區。所有功能性調整應同步更新此處相關文件，避免規格只存在於討論紀錄。

## 目前階段

- 專案：Voiles
- 原始碼副檔名：`.voil`
- 階段：v0.1 language baseline 已接受；compiler/runtime implementation 尚未開始
- 工作 branch：`planning/syntax-spec`
- P0 lexer/parser/binding/module/component baseline 已足以進入 parser/compiler prototype
- 真正開始實作前必須建立新的 implementation branch 並先查閱所使用外部套件的官方最新文件
- 未經明確授權不合併 `main`

## 文件

- [`language-spec.md`](./language-spec.md)：v0.1 語言主規格、grammar baseline、route/safety/HMR/compiler representation。
- [`syntax-decisions.md`](./syntax-decisions.md)：Accepted/Open/Deferred 設計決策 registry。
- [`state-model.md`](./state-model.md)：`const` / `state` / `shared`、closure、scoped module、component identity/lifecycle。
- [`component-model.md`](./component-model.md)：component declaration/export/parameters/callback/identity/lifecycle/slots/tree-shaking。
- [`todo.md`](./todo.md)：已解決項、implementation/prototype work、CSS design phase 與 implementation gate。

## 更新規則

1. 實作或規格變更前先完整確認相關文件現況。
2. 每個功能 branch 同步更新受影響的 `AI-build` 文件。
3. 已定案內容標記為 `Accepted`；仍可能改動者標記為 `Draft`；需要獨立討論/prototype 者標記為 `Open`；刻意延後者標記為 `Deferred`。
4. 語法變更需同步更新範例、grammar 與 compiler TODO。
5. 實作一律建立新 branch。
6. 合併至 `main` 一律使用 squash commit。