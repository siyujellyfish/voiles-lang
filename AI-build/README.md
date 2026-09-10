# AI-build

`AI-build` 是 Voiles 的持續規劃與實作紀錄區。所有功能性調整應同步更新此處相關文件，避免規格只存在於討論紀錄。

## 目前階段

- 專案：Voiles
- 原始碼副檔名：`.voil`
- 階段：語言語法與 compiler 前置規格
- 工作 branch：`planning/syntax-spec`
- 本階段不包含 compiler/runtime 實作

## 文件

- [`language-spec.md`](./language-spec.md)：`.voil` 語法規格草案與完整範例。
- [`syntax-decisions.md`](./syntax-decisions.md)：已接受原則、暫定方案、待決策項目。
- [`todo.md`](./todo.md)：目前與下一階段工作項目。

## 更新規則

1. 實作或規格變更前先確認相關文件現況。
2. 每個功能 branch 同步更新受影響的 `AI-build` 文件。
3. 已定案內容標記為 `Accepted`；仍可能改動者標記為 `Draft`；需要討論者標記為 `Open`。
4. 語法變更需同步更新範例、grammar 與 compiler TODO。
5. 合併至 `main` 時使用 squash commit。
