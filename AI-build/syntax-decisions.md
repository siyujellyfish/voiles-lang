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

所有 block opener 以 `:` 結束 header，child hierarchy 由縮排決定。不使用 `{}` 作為一般 block delimiter，也不使用 HTML/XML closing tag。

```voil
fn click_btn():
	count += 1

main:
	container:
		std.h1("Hello")
```

### A-004：`#` 僅作為註解起始符

```voil
# comment
```

`#` 不承擔 HTML id shorthand 或其他語意。

### A-005：Import module specifier 必須使用字串

```voil
import std from "@voiles/html-base"
import UserBtn from "../ui/components/UserBtn.voil"
```

套件、標準模組與相對路徑都使用 quoted module specifier。

### A-006：Function declaration 使用 `fn`

```voil
fn click_btn():
	count += 1
```

`fn` 保留 declaration / invocation 的明確界線。

### A-007：Page 可直接使用 semantic HTML structural blocks

頁面不要求 `view:` wrapper 或第一行 `page` declaration：

```voil
header:
	...

main:
	...

footer:
	...
```

### A-008：Native HTML 與 user component 在 source 上明確區分

Native HTML API 經由標準 HTML module namespace 使用：

```voil
import std from "@voiles/html-base"
std.h1("Hello")
```

User component 使用 imported PascalCase symbol：

```voil
UserBtn(
	text="just click"
)
```

`std` 為官方文件與 formatter 建議的 canonical alias；是否提升為 reserved namespace 仍可後續評估。

### A-009：Component invocation 採 function-like syntax

User component 不使用 JSX/XML tag。Component children 使用 `:` 開啟 child block。

### A-010：Reactive mutation 必須明確

會觸發 UI dependency tracking 的資料必須使用明確 mutable binding；immutable binding 不因被 UI 引用就自動變成 reactive state。

### A-011：String 不等於可信 HTML

一般 `String` 插入 HTML API 一律做 text escaping。raw HTML 必須通過明確安全型別或 unsafe boundary。

### A-012：無 JavaScript 式 `undefined` 語意

可缺失值使用 `Option<T>` 或其語法糖。Voiles 原生型別系統不以 `undefined` 作一般值。

### A-013：Binding 使用 lexical scope 與 nearest-binding resolution

名稱由目前 lexical scope 向外解析，內層可 shadow 外層。相同 scope 重複宣告與 use-before-declaration 都是 compile error。

### A-014：同 importer scope + 同 resolved path 共用同一 module instance

```text
same importer scope + same resolved path
-> same module instance

different importer scope + same resolved path
-> different module instance
```

不同 alias 不會在相同 importer scope 內複製 module state。

### A-015：共享性宣告在變數，而非 module/import

```voil
shared i = 1
```

`shared` 本身代表 mutable shared binding，不使用 `shared import` 或 `shared state`。

### A-016：`shared` 只允許 module top-level

`shared` 視為 application-global storage，因此只能出現在 `.voil` module 最外層。Component、function、`if`、loop 等 nested scope 中的 `shared` 都是 compile error。

### A-017：每次 Component invocation identity 建立獨立 instance scope

不同 component invocation identity 各自擁有 component-local `state` 與普通 scoped `.voil` dependencies；只有 `shared` 跨 instance 共用。

### A-018：Component 使用顯式 `component` declaration

```voil
component UserBtn(
	text: String,
	disabled: Bool = false
):
	state count = 0
	...
```

Voiles 不依 render root 推論 component。`fn` 表示 callable logic；`component` 表示 instantiable UI。

### A-019：所有 Component declaration 自動 export，未使用元件可移除

所有 top-level `component Name(...):` 自動加入 module component export surface，不需要 `export component`。不可達 component 可由 compiler tree-shake；module top-level side effect 是否能一起移除仍由 effect policy 決定。

### A-020：多 Component import 使用逗號分隔

```voil
import A, B from "./c.voil"
```

每個名稱依 component declaration name 解析。Non-component import semantics 另行討論。

### A-021：Component children 使用 `slot` 模型

Ordinary child content 進入 default slot：

```voil
Card(title="Profile"):
	std.p(user.name)
```

Component 內使用 compiler-level `slot` outlet；named slot 使用 `slot Name` / `slot Name:`。

Slot content 保留 caller lexical scope；component 只控制插入位置。

### A-022：Slot cardinality 採單一 optional outlet/provision

v0.1 中 default/named slot 預設 optional。Component declaration 最多一個 default outlet、每個 named outlet 最多一個；caller 每個 named slot 最多提供一次。

Unknown named slot、duplicate outlet/provision，以及沒有 default outlet 卻傳 ordinary children 都是 compile error。v0.1 不加入 required slot、重複 projection/cloning 或 scoped-slot parameter。

### A-023：Component parameter 為 immutable named input，prop 更新保留 instance

```voil
component UserCard(
	name: String,
	title: String = name,
	disabled: Bool = false
):
	...
```

Accepted 規則：

- 無 default 為 required；有 default 為 optional；
- parameter 在 component 內不可 assignment；
- component invocation 只允許 named arguments；
- default expression 每次 invocation 各自 evaluate；
- parameter/default initialization 由左至右，default 只能引用前面的 parameter；
- mutable input copy 必須明確建立 `state`；
- reactive prop 改變時保留既有 component instance/local state，不因 prop 更新而 destroy/recreate。

```voil
component Input(value: String):
	state current = value
```

`current` 只在 instance initialization 取當下 `value`；後續 prop update 不自動覆寫它。

### A-024：Component identity 採 structural position；Repeated UI 必須 explicit `key`

非 repeated UI 中，component invocation identity 由穩定結構位置決定。該位置的 reactive values/props 改變時保留同一 component instance。

Conditional branch 是 lifetime boundary：branch 退出時其中 component instance unmount，ordinary local state 與 scoped dependency instance 釋放；重新進入 branch 時建立新 instance。

Repeated UI 若建立 component instance，必須宣告 explicit iteration key：

```voil
for user in users key user.id:
	UserCard(
		user=user
	)
```

Voiles v0.1 不使用隱式 index identity；component-instantiating UI loop 缺少 `key` 為 compile error。

同 key 在 reorder 後保留 component instance/state；key 消失則 unmount；新 key 建立新 instance；key 改變等同 identity replacement。

v0.1 key type 限定 `String` / `Int`。同一 repeated UI evaluation 中 duplicate key 為 runtime error。

普通不建立 repeated component UI 的 algorithmic loop 不要求 `key`。

### A-025：Component lifecycle 使用 `mount` + nested `cleanup`

Component-owned resource lifecycle 使用：

```voil
component Clock():
	state now = get_time()

	mount:
		const timer = start_timer():
			now = get_time()

		cleanup:
			timer.stop()

	std.p(now)
```

Accepted 規則：

- `mount:` 在每個新 mounted component instance 執行一次；
- reactive prop/state update 不重跑 `mount:`；
- keyed reorder 且 key 不變時不 cleanup/remount；
- `cleanup:` 只能出現在 `mount:` block 內；
- `cleanup:` 可 capture enclosing `mount:` lexical bindings；
- instance unmount 時 `cleanup:` 執行一次；
- conditional branch removal、key removal/replacement 都會觸發舊 instance cleanup + unmount；
- conditional re-entry / new key 建立新 instance 後重新執行 mount；
- v0.1 不提供 reactive `effect`、dependency array 或自動 rerun 語意。

這讓 timer、subscription、listener 等資源的建立與 teardown 保持同一 lexical lifecycle scope，不要求把 resource handle 存入 component `state`。

普通 scoped module instance 的 cleanup/lifetime 仍是獨立 module-system 決策。

## Draft

### D-001：變數模型縮減為 `const` / `state` / `shared`

目前偏好：

```voil
const title = "Voiles"
state count = 0
shared session = none
```

- `const`：immutable runtime binding。
- `state`：mutable scoped binding；被 UI/runtime dependency 觀察時由 compiler 產生 reactive update。
- `shared`：mutable application-global binding，只允許 module top-level。
- v0.1 不提供 `let` / `var` 的方向尚待最終確認。

### D-002：Named argument / component prop separator 使用 `=`

Component call 已採 named-only `=`：

```voil
UserBtn(
	text="just click",
	onclick=click_btn
)
```

普通 function named arguments 與 struct construction 是否統一使用 `=` 仍需 grammar 驗證。

### D-003：`container` 是具體的 block/layout container

`container` 概念上接近 `<div>`，預期預設 lowering 為 concrete block container。只有 compiler 能證明不改變 layout/style/event/accessibility/lifecycle 語意時才可消除 wrapper。

### D-004：Optional shorthand `T?` 等價於 `Option<T>`

```voil
String?
```

語意為 `Option<String>`，不是 nullable reference。

### D-005：Route param 可在頁面內宣告型別

```voil
param id: Int
```

Compiler 依 route segment 建立 typed binding；conversion failure 不得注入 unchecked invalid value。

### D-006：Component import alias 使用 item-local `as`

目前推薦：

```voil
import A as X, B as Y from "./c.voil"
```

混用亦可：

```voil
import A, B as SmallB from "./c.voil"
```

此語法尚未升為 Accepted。

## Open

### O-001：CSS / Voiles layout integration

需要定義 CSS property/value、typed layout API、component-local scope、pseudo selector、queries、custom property、animation 與 raw/native CSS escape hatch。

### O-002：一般非 reactive mutable local

需決定 function-local algorithmic mutation 是以 unobserved `state` lowering、額外 `mut`，或其他受限 construct 表達。

### O-003：Top-level UI block 的完整 grammar

需確認頁面 sibling structural roots 與 component block 可接受的 UI roots。

### O-004：Slot parameters / typing

Default/named slot syntax 與 v0.1 cardinality 已定案；slot parameter/scoped-slot 與 slot content type model 待 concrete use case 再設計。

### O-005：Event type model

需定義 native DOM event surface、handler signature 與 component callback typing。

### O-006：Standard HTML module surface

需定義 `@voiles/html-base` 的 elements、attributes、events、escaping、semantic block 邊界與 Web platform versioning。

### O-007：Non-component import symbol forms

Component 單/多 symbol import 已有基本語法；仍需定義 component alias、non-component default/named/namespace import 與 JS/npm interop grammar。

### O-008：Async/error syntax

需決定 `async fn`、`Result<T, E>` propagation 與 throwing JS API interop。

### O-009：Scoped module lifecycle / cycles

Component `mount`/`cleanup` 與 component identity lifetime 已定案；仍需定義 ordinary scoped module instance cleanup、cyclic imports 與 `shared` initialization order。

### O-010：Module top-level side effects / tree-shaking boundary

需定義 optimizer 何時必須保留 module initialization，以及何時能移除整個 module。

## Deferred

### X-001：Macro system

v0.1 不設計 general-purpose macro。

### X-002：Operator overloading

v0.1 不開放使用者自訂 operator overloading。

### X-003：完整 borrow checker

Web target 不引入 Rust 式 lifetime/borrow syntax。

### X-004：SSR-only 語法

SSR/SSG 尚未進入 MVP，不先污染 core syntax。