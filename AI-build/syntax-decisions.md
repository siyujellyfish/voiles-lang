# Voiles Syntax Decisions

此文件記錄語法層級的設計決策。狀態定義：

- `Accepted`：現階段視為基礎規格，除非出現明確架構衝突才調整。
- `Draft`：已有偏好方案，但尚未鎖定。
- `Open`：需後續討論或 prototype 驗證。
- `Deferred`：刻意延後，不納入目前 MVP。

## Accepted

### A-001：`.voil` 是獨立語言

Voiles 不採 JSX/TSX template language 路線，也不把 TypeScript parser/type checker 當作語言核心。JS/TS 僅作為輸出目標、interop 邊界或工具鏈整合層。

### A-002：File-based routing 不要求 route 宣告 path

`app/` 內檔案路徑即 URL route。一般頁面不寫 `route "/path"`，避免 filesystem 與 source code 出現兩份 path 真相來源。

### A-003：沒有 closing tag

View syntax 不使用 HTML/XML closing tag。元素與 component 的 child hierarchy 由區塊結構表達。

### A-004：View control flow 使用語言本身的 control-flow constructs

條件、迭代與 pattern matching 不設計 `v-if`、`#each`、`{#if}` 等 template-only 第二套語法，而使用 `if`、`for`、`match`。

### A-005：Reactive state 必須明確標示

會觸發 UI dependency tracking 的可變資料使用 `state` 宣告。普通 local value 不因被 View 使用就自動變成 reactive state。

### A-006：String 不等於可信 HTML

一般 `String` 插入 View 一律做 text escaping。raw HTML 必須通過明確的安全型別或 unsafe boundary，不能因 interpolation 自動成為 HTML。

### A-007：無 `undefined` 語意

Voiles 原生型別系統不提供 JavaScript 式 `undefined`。可缺失值使用 `Option<T>` 或其語法糖。

### A-008：Style 由 compiler 解析，不當作任意字串

即使 style syntax 保留 CSS 接近度，property/value 仍進入 typed style IR，以支援 validation、dead-style elimination 與 codegen 最佳化。

## Draft

### D-001：使用 significant indentation 表達 block

偏好：

```voil
view
	main.page
		h1 "Hello"
		p "Welcome"
```

而不是：

```voil
view {
	main.page {
		h1("Hello")
		p("Welcome")
	}
}
```

理由：

- 與「最小標籤化、程式扁平化」方向一致。
- View 不需要 closing tag、brace 或 delimiter noise。
- formatter 可強制唯一結構風格。

需要驗證：

- multiline expression 與 block 的 ambiguity。
- comment/trivia preservation。
- formatter 對 tab/space 的 canonical policy。
- copy/paste 或 generated code 的可預測性。

### D-002：View node 採 `element.class#id(attrs) value` 形式

暫定：

```voil
button.primary(type: "button", disabled: busy) "Save"
```

而不是 JSX/XML attribute syntax。

理由：attribute grammar 集中於括號內，可避免 element 後方出現大量互相衝突的 token。

### D-003：Component file 採 implicit default component

一個 component `.voil` file 本身即一個 default component，不要求每個檔案重複：

```voil
component UserCard
```

預計透過 `prop` + `view` 定義：

```voil
prop user: User

view
	article.card
		h2 user.name
```

是否允許同檔 named component 尚未定案。

### D-004：Optional shorthand `T?` 等價於 `Option<T>`

偏好允許：

```voil
prop subtitle: String?
```

其語意仍為 explicit `Option<String>`，不是 nullable reference。

### D-005：Route param 可在頁面內宣告型別

例如 `app/users/[id].voil`：

```voil
param id: Int
```

compiler 依 route segment 產生 parser 與 typed binding。若 URL segment 無法轉成 `Int`，預設 route 不匹配或進入 route error，而不是把 invalid value 傳入頁面。

### D-006：Style 使用縮排式 CSS-compatible property syntax

暫定：

```voil
style .card
	display: grid
	gap: 1rem
	padding: 1rem
```

避免重新命名整套 CSS property，同時由 compiler 做 typed parsing。

### D-007：Layout primitive 可不產生 DOM wrapper

`stack`、`row`、`grid`、`layer` 視為 View/layout primitive，而不是固定對應 `<div>`。compiler 應依語意判斷是否需要實體 container。

## Open

### O-001：significant indentation 是否正式定案

這是目前最優先的語法決策，會直接決定 lexer/parser/CST/formatter 架構。

### O-002：block delimiter 是否需要 `:`

候選：

```voil
if ready
	...
```

或：

```voil
if ready:
	...
```

前者更乾淨，後者在 parser 與人類閱讀上更明確。

### O-003：attribute separator 使用 `:` 還是 `=`

候選：

```voil
button(type: "button", disabled: busy)
```

或：

```voil
button(type = "button", disabled = busy)
```

需與 named argument、struct literal 與 style declaration 一起考量，避免三套近似語法。

### O-004：事件語法

候選 A：

```voil
button "Save"
	on click save
```

候選 B：

```voil
button(on.click: save) "Save"
```

候選 C：

```voil
button "Save" on click save
```

目前偏好 A，因 child block 中可以自然容納多事件與 modifier，但需要確認是否造成過度 nesting。

### O-005：two-way binding 是否存在

需要決定是否提供：

```voil
input bind value: name
```

或堅持單向 value + event update，避免隱含 mutation。

### O-006：component import/module syntax

待確定 relative module、project-root module、npm/JS interop 是否共享同一個 `use` 語法。

### O-007：style scope 預設

候選：

1. `.voil` style 預設 component-scoped。
2. route/component style 預設局部，但需要 `global style` 才能跨 component。
3. 完全採 CSS 原生 cascade，不做 scope rewrite。

目前偏好 1/2，仍需考慮 CSS interoperability。

### O-008：raw CSS escape hatch

需設計不破壞 typed style parser 的 escape hatch，並明確標示 optimizer 無法提供完整保證的範圍。

### O-009：async/error syntax

需決定 `async fn`、`Result<T, E>` propagation、throwing JS API interop 的具體語法。安全模型要求不能把 JS exception 默默偽裝成 typed Result。

### O-010：named slot / child content 模型

需確定 component children 是否使用 `children`、slot 名稱或 typed child parameter，並避免重新引入大量 template tag。

## Deferred

### X-001：macro system

v0.1 不設計 general-purpose macro。

### X-002：operator overloading

v0.1 不開放使用者自訂 operator overloading。

### X-003：完整 borrow checker

Web target 不引入 Rust 式 lifetime/borrow syntax。資源與 unsafe interop 的 safety 由不同機制處理。

### X-004：SSR-only 語法

SSR/SSG 尚未進入 MVP，不先污染 core syntax。
