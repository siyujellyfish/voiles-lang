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

### A-003：Block 使用 `:` + significant indentation

所有會開啟 block 的語法以 `:` 明確結束 header，child hierarchy 由縮排決定。

```voil
fn click_btn():
	count += 1

main:
	container:
		std.h1("Hello")
```

不使用 `{}` 作為一般 block delimiter，也不使用 HTML/XML closing tag。

### A-004：`#` 僅作為註解起始符

```voil
# comment
```

`#` 不再承擔 HTML id shorthand 等其他語意，避免 lexer/context ambiguity。

### A-005：Import module specifier 必須使用字串

```voil
import std from "@voiles/html-base"
import UserBtn from "../ui/components/UserBtn.voil"
```

套件、Voiles 標準模組與相對路徑都使用 quoted module specifier，避免把 `@`、`/`、`.`、`-` 引入一般 identifier grammar。

### A-006：Function declaration 使用 `fn`

```voil
fn click_btn():
	count += 1
```

`fn` 保留 declaration / invocation 的明確界線，避免 `name():` 同時可能代表 function declaration 或 component/block invocation。

### A-007：Page 可直接使用 semantic HTML structural blocks

頁面不要求額外 `view:` wrapper 或第一行 `page` declaration。結構型 HTML node 可以直接作為 block：

```voil
header:
	...

main:
	...

footer:
	...
```

這些 block 對應其 HTML semantic element。

### A-008：Native HTML 與 user component 在 source 上明確區分

Native HTML leaf/component API 經由標準 HTML module namespace 使用：

```voil
import std from "@voiles/html-base"

std.h1("Hello")
std.p(count)
```

User component 使用 imported PascalCase symbol：

```voil
UserBtn(
	text="just click",
	onclick=click_btn
)
```

`std` 為官方文件與 formatter 建議的 canonical alias；是否進一步保留成語言級 namespace 仍可在 module system 階段評估。

### A-009：Component invocation 採 function-like syntax

User component 不使用 JSX/XML tag：

```voil
UserBtn(
	text="just click",
	onclick=click_btn
)
```

Component children 使用 `:` 開啟 child block，而不是 closing tag。

### A-010：Reactive mutation 必須明確

會觸發 UI dependency tracking 的資料必須使用明確的 mutable binding；普通 immutable binding 不因被 UI 引用就自動變成 reactive state。

### A-011：String 不等於可信 HTML

一般 String 插入 HTML API 一律做 text escaping。raw HTML 必須通過明確的安全型別或 unsafe boundary。

### A-012：無 JavaScript 式 `undefined` 語意

可缺失值使用 `Option<T>` 或其語法糖。Voiles 原生型別系統不以 `undefined` 作一般值。

### A-013：Binding 使用 lexical scope 與 nearest-binding resolution

名稱依 lexical scope 解析，由目前區塊向外尋找最近的 declaration。內層可 shadow 外層名稱。

```voil
state i = 1

fn count():
	i += 1

fn count_2():
	state i = 5
	i += 1
```

`count()` 修改外層 `i`；`count_2()` 修改自己的 local `i`。

同一 lexical scope 重複宣告同名 identifier 為 compile error；宣告前不可使用。

### A-014：同 importer scope + 同 resolved path 共用同一 module instance

普通 `.voil` import 採 scoped module instance 模型。

```text
same importer scope + same resolved path
-> same module instance
```

即使使用不同 alias：

```voil
import a from "./store.voil"
import b from "./store.voil"
```

在同一 importer scope 內仍指向同一個 resolved module instance，不允許藉由 alias 或重複 import 隱式複製 mutable state。

不同 importer scope 對同一 `.voil` path 預設取得不同 module instance，因此普通 `state` 不會自然變成 application-global singleton。

### A-015：共享性宣告在變數，而非 module/import

Voiles 不採 `shared import` 或 shared-module 作為主要共享狀態模型。

共享變數直接寫：

```voil
shared i = 1
```

`shared` 本身代表 mutable shared binding，因此不寫 `shared state i = 1`。

同一 declaration 的 `shared` storage 會跨 declaring module 的不同 scoped module instances 共用。普通 `state` 則保留 module-instance 隔離。

完整 scope/state 模型記錄於 [`state-model.md`](./state-model.md)。

### A-016：`shared` 只允許 module top-level

`shared` 視為 application-global storage，因此只能出現在 `.voil` module 最外層。

允許：

```voil
shared session = none
```

禁止：

```voil
component Counter():
	shared i = 0
```

```voil
fn test():
	shared i = 0
```

也禁止出現在 `if`、`for`、component child block 等任何 nested scope。這避免引入 static-local、recursive invocation、closure、async task 等額外生命週期語意。

### A-017：每次 Component invocation 建立獨立 instance scope

元件復用時，每次 invocation 都必須建立新的 component instance scope。

```voil
UserBtn()
UserBtn()
```

若 `UserBtn` 內有：

```voil
state count = 0
```

則兩個 invocation 各自擁有獨立的 `count`，修改其中一個不影響另一個。

Component instance 同時視為 importer scope。因此元件內 import 的普通 scoped `.voil` module 也會依 component instance 分離；只有 `shared` declaration 會跨 component instances 共用。

### A-018：Component 使用顯式 `component` declaration

Voiles 不再依 top-level render root 推論一個檔案是否為 component。元件由 declaration 明確表示：

```voil
component UserBtn(
	text: String,
	disabled: Bool = false
):
	state count = 0
	...
```

Component declaration header 同時定義 component inputs/props，因此目前不需要額外 `input` / `prop` keyword。

`fn` 與 `component` 的角色保持對稱：

```text
fn
-> callable logic

component
-> instantiable UI
```

一個 `.voil` module 可包含多個 component declarations，名稱在同一 module 內必須唯一。

### A-019：所有 Component declaration 自動 export，未使用元件可移除

所有 top-level `component Name(...):` declaration 自動加入該 module 的 component export surface，不需要額外：

```voil
export component ...
```

規格使用 automatic/implicit component export 這個概念，而不是 JavaScript 的 single `default export`，因為一個 `.voil` module 可以有多個自動可匯入的 component。

Compiler 會建立 component dependency graph；從 application entry points 不可達的 component declarations 可進行 tree-shaking / dead-code elimination。

Component-local initialization 只在 invocation 時發生，因此未被使用的 component 不應建立 component state 或 render output。

若 module 另有獨立 top-level side effects，是否能連同整個 module 移除仍由後續 module side-effect policy 決定。

### A-020：多 Component import 使用逗號分隔

同一 `.voil` module 的多個自動 export component 直接寫在同一 import statement，以逗號分隔：

```voil
import A, B from "./c.voil"
```

每個名稱依 component declaration name 解析。例如：

```voil
# buttons.voil
component UserBtn(...):
	...

component IconBtn(...):
	...
```

```voil
import UserBtn, IconBtn from "./buttons.voil"
```

此規則只定案 component symbol list 的表面語法；non-component module/default/named/namespace import semantics 仍分開討論。

### A-021：Component children 使用 `slot` 模型

Component invocation 使用既有 `:` + indentation 傳遞 children：

```voil
Card(title="Profile"):
	std.p(user.name)
```

未包在 named slot block 的 child content 進入 default slot。Component 內使用 compiler-level `slot` outlet 決定插入位置：

```voil
component Card(title: String):
	container:
		std.h2(title)
		slot
```

Named slot 使用同一關鍵字：

```voil
component Modal():
	container:
		slot header
		slot
		slot footer
```

Caller 以 `slot Name:` 提供 named slot content：

```voil
Modal():
	slot header:
		std.h2("Confirm")

	std.p("Delete this item?")

	slot footer:
		Button(text="Cancel")
		Button(text="Delete")
```

`slot` 不是 ordinary function call，而是 compiler-level UI insertion point。

Slot content 保留 caller lexical scope，不會因為被插入 component outlet 而改用 callee/component lexical scope。Component 也不會因此取得 caller local bindings；若未來需要把 component-local 值傳入 slot，需另外設計明確的 slot-parameter 機制。

## Draft

### D-001：變數模型縮減為 `const` / `state` / `shared`

目前偏好：

```voil
const title = "Voiles"
state count = 0
shared session = none
```

語意候選：

- `const`：immutable binding；值可以在 runtime 初始化，不等於 compile-time constant。
- `state`：mutable binding；storage 跟隨 lexical/module/component instance scope；被 UI/runtime dependency 觀察時由 compiler 產生 reactive update。
- `shared`：mutable shared binding；跨 declaring module/component instances 共用 storage，且只允許 module top-level。
- v0.1 不提供 `let` / `var`。

尚需處理一般函式中的「非 reactive mutable local」需求。若確實必要，優先考慮之後加入限定於 function-local 的 `mut`，而不是增加多套一般變數模型。

### D-002：Named argument / component prop separator 使用 `=`

目前範例統一採：

```voil
UserBtn(
	text="just click",
	onclick=click_btn
)
```

需要與 assignment、default parameter、struct construction grammar 一起驗證後再升為 Accepted。

### D-003：`container` 是具體的 block/layout container

`container` 不是純抽象、預設 wrapperless 的 layout primitive；概念上更接近 `<div>`，負責建立區塊性布局範圍：

```voil
container:
	...
```

預期預設 lowering 可使用 `<div>` 或同等 block container。只有 compiler 能證明移除 wrapper 不改變 layout、style、event、accessibility 等語意時，才允許最佳化消除。

`container(...)` 內的 layout/style 參數仍未定案。

### D-004：Optional shorthand `T?` 等價於 `Option<T>`

```voil
String?
```

其語意仍為 `Option<String>`，不是 nullable reference。

### D-005：Route param 可在頁面內宣告型別

例如 `app/users/[id].voil`：

```voil
param id: Int
```

compiler 依 route segment 建立 typed binding；轉換失敗不得把 invalid value 傳入頁面。

### D-006：Component import alias 使用 item-local `as`

目前推薦：

```voil
import A as X, B as Y from "./c.voil"
```

alias 綁在個別 import item 上，因此混用亦可：

```voil
import A, B as SmallB from "./c.voil"
```

選擇 `as` 的理由是避免與 assignment / named argument 的 `=` 混淆，也不需要引入 `{ ... }` import wrapper。此語法尚未升為 Accepted。

## Open

### O-001：CSS / Voiles layout integration

CSS 能力與語法範圍過大，暫不把 `container(display=...)` 或自訂 style DSL 視為已定案。

需要分別討論：

1. Voiles 是否直接接受 CSS property/value。
2. layout primitive 是否建立較高階的 typed layout API。
3. component-local scope 與 native cascade 如何共存。
4. pseudo selector、media/container query、custom property、animation 等如何保留完整 CSS 能力。
5. raw/native CSS escape hatch 是否需要，以及 optimizer 能提供哪些保證。

### O-002：一般非 reactive mutable local

若主要只有 `const` / `state` / `shared`，algorithmic function 內的 accumulator、loop-local mutation 等需求如何處理仍需確認。

候選：

- 直接允許 function-local `state`，由 compiler 在無 observer 時降低成普通 mutable local。
- 提供 function-local `mut`。
- 使用其他受限 mutation construct。

### O-003：Top-level UI block 的完整 grammar

需要確認頁面是否允許多個 sibling structural root，例如 `header:` + `main:` + `footer:`，以及 component block 內可接受哪些 UI roots。

### O-004：Slot cardinality / typing

Default/named slot syntax 已定案，仍需定義：

- slot 是 required 或 optional 的宣告方式；
- 同一 named slot 是否可提供多次；
- 同一 slot outlet 是否可重複出現，以及重複時是 clone 還是 move；
- slot parameter / scoped-slot 是否需要；
- slot content 的型別模型。

### O-005：Event type model

`onclick=click_btn` 已作為 component prop 語法可用，但 native DOM event 要由 `std` API 如何暴露、handler signature 如何檢查，仍需設計。

### O-006：Standard HTML module surface

需要定義 `@voiles/html-base` 實際提供哪些 API：

- leaf HTML elements；
- attributes；
- events；
- escaping；
- semantic block 與 `std.*` API 的邊界；
- Web platform versioning。

### O-007：Non-component import symbol forms

Component 單/多 symbol import 已有基本語法；仍需定義：

- component alias 最終語法（目前 `as` 為 Draft）；
- non-component default/named/namespace import；
- JS/npm interop import grammar。

### O-008：Async/error syntax

需決定 `async fn`、`Result<T, E>` propagation、throwing JS API interop 的具體語法。

### O-009：Scoped module lifecycle / cycles

需要定義：

- scoped module/component instance 何時建立與釋放；
- cyclic imports 的 initialization 順序；
- `shared` declaration 在 cycle 中的初始化規則。

### O-010：Module top-level side effects / tree-shaking boundary

Component declaration 本身可 tree-shake；但若 `.voil` module 允許任意 top-level side effects，需要定義 optimizer 是否必須保留 module initialization，以及何時可移除整個 module。

## Deferred

### X-001：Macro system

v0.1 不設計 general-purpose macro。

### X-002：Operator overloading

v0.1 不開放使用者自訂 operator overloading。

### X-003：完整 borrow checker

Web target 不引入 Rust 式 lifetime/borrow syntax。資源與 unsafe interop 的 safety 由不同機制處理。

### X-004：SSR-only 語法

SSR/SSG 尚未進入 MVP，不先污染 core syntax。
