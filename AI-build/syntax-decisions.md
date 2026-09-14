# Voiles Syntax Decisions

此文件記錄語法與核心 runtime 層級的設計決策。狀態：

- `Accepted`：v0.1 基礎規格，除非出現明確架構衝突才調整。
- `Draft`：已有方向但尚未鎖定。
- `Open`：需要獨立設計或 prototype。
- `Deferred`：刻意不納入 v0.1 baseline。

## Accepted

### A-001：`.voil` 是獨立語言

Voiles 不採 JSX/TSX template language 路線；JS/TS 是輸出與 interop 邊界，不是語言核心 parser/type checker。

### A-002：File-based routing 不重複宣告 path

`app/` 內檔案路徑即 URL route，不在 source 中再維護第二份 route path。

### A-003：所有 block 使用 `:` + significant indentation

```voil
fn click_btn():
	count += 1

main:
	std.p(count)
```

一般 block 不使用 `{}` 或 HTML/XML closing tag。

### A-004：`#` 僅為 line comment

`#` 不承擔 HTML id shorthand 或其他語意。

### A-005：Import module specifier 必須是 quoted string

```voil
import std from "@voiles/html-base"
import UserBtn from "../ui/UserBtn.voil"
```

### A-006：Function declaration 使用 `fn`

`fn` 保留 declaration / invocation 的明確界線。

### A-007：Page 可直接使用 structural HTML blocks

頁面不需要 `page` 或 `view:` wrapper。

### A-008：Native HTML 與 user component source surface 分離

Native HTML 主要經 `std.*`；user component 使用 PascalCase invocation。

### A-009：Component invocation 採 function-like syntax

```voil
UserBtn(text="Save")
```

Children 使用 `:` + indentation。

### A-010：Mutable/reactive data 必須明確宣告

Immutable binding 不因被 UI 讀取就自動變成 mutable/reactive state。

### A-011：`String` 不等於 trusted HTML

一般字串進 HTML sink 一律 escaping；raw HTML 需要 `TrustedHtml` 或明確 unsafe boundary。

### A-012：沒有 JavaScript-style `undefined`

缺失值使用 `Option<T>` / `T?` 與 `none`。

### A-013：Lexical scope + nearest-binding resolution

內層可以 shadow 外層；same-scope redeclaration 與 use-before-declaration 都是 compile error。

### A-014：Scoped `.voil` module identity

```text
same importer scope + same resolved path
-> same module instance

different importer scope + same resolved path
-> different module instance
```

Alias 不會複製 module state。

### A-015：Sharing 宣告在 variable

```voil
shared count = 0
```

不使用 `shared import` 或 `shared state`。

### A-016：`shared` 只允許 module top-level

Component/function/control-flow nested scope 中宣告 `shared` 為 compile error。

### A-017：每個 component invocation identity 有獨立 instance scope

Component-local `state` 與 ordinary scoped dependencies 依 component identity 隔離；`shared` 是例外。

### A-018：Component 使用顯式 `component` declaration

```voil
component UserBtn(text: String):
	...
```

### A-019：Top-level component 自動 export；unused component 可 DCE

Component 不需要 `export component`。不可達 component declaration 可被 tree-shake。

### A-020：同 module 多 component import 使用逗號 list

```voil
import A, B from "./c.voil"
```

### A-021：Component children 使用 `slot`

普通 child content 進 default slot；`slot Name:` 提供 named slot；slot content 保留 caller lexical scope。

### A-022：v0.1 slot 為 optional single outlet/provision

一個 default outlet、每個 named outlet/provision 最多一次；unknown slot 或無 default outlet 卻傳普通 children 為 compile error。

### A-023：Component parameter 是 immutable named input

Component call named-only；default 每 instance evaluate、由左至右，只能引用前面已初始化 parameter。Reactive prop 更新保留既有 component identity/local state。

### A-024：Component identity 採 structural position；Repeated UI 必須 explicit `key`

非 repeated UI 由穩定 call-site structure 決定 identity。Conditional branch removal 會 unmount。Repeated component UI：

```voil
for user in users key user.id:
	UserCard(user=user)
```

v0.1 key 為 `String | Int`；不使用隱式 index identity；duplicate runtime key 是 deterministic runtime error。

### A-025：Component lifecycle 使用 `mount` + nested `cleanup`

`mount:` 每新 mounted instance 一次；prop/state update 與 unchanged-key reorder 不重跑。`cleanup:` 只能位於 `mount:` 內、可 capture mount locals，unmount 時執行一次。

### A-026：Scoped module lifecycle 使用 `init` + nested `cleanup`

Ordinary scoped module instance 由 importer scope 擁有。`init:` 每 instance 一次；同 importer + path reuse。Teardown dependency-first/post-order：dependency module cleanup -> importer module cleanup -> owner component cleanup。

Ordinary module cleanup 不 destroy/reset `shared` storage。

### A-027：Lexer indentation 採 Python-style logical-line stack model

Voiles 的 `NEWLINE` / `INDENT` / `DEDENT` 核心規則參照 Python lexical indentation：

- physical EOL normalize 為 logical newline；
- 非 continuation 的 logical line 結束產生 `NEWLINE`；
- blank / whitespace-only / comment-only logical line 不產生 `NEWLINE`、`INDENT`、`DEDENT`；
- indentation stack 初始為 `0`；較深 push 並產生一個 `INDENT`；較淺必須回到 stack 中既有 level，依 pop 數產生 `DEDENT`；
- EOF 對所有剩餘非零 indentation level 產生 `DEDENT`；
- indentation 不符合既有 stack level 為 lexer/parser diagnostic；
- tabs 使用 Python-style tab stop 計算（下一個 8-column boundary）；tabs/spaces 混用若造成 interpretation ambiguity 為 indentation error；
- formatter canonical output 使用 tab indentation。

### A-028：Implicit multiline continuation 採 bracket context

在 `(...)`、`[...]`、`{...}` expression context 內允許跨 physical line；中間 physical newline 不產生 logical `NEWLINE`，continuation indentation 不參與 block `INDENT/DEDENT`。Blank/comment lines 可存在於 continuation 中。

v0.1 不需要 Python-style backslash explicit line joining；formatter 以 bracketed continuation 為 canonical form。

### A-029：Binding model 正式定案為 `const / state / shared`

```voil
const title = "Voiles"
state count = 0
shared session = none
```

- `const`：immutable runtime binding；
- `state`：唯一 ordinary mutable binding，storage 依 lexical/instance scope；
- `shared`：module-top-level application-global mutable cell；
- v0.1 不提供 `let`、`var`、`mut`；
- function-local algorithmic mutation 一律使用 `state`；compiler 若證明沒有 observer，可 lower 成普通 mutable local，不建立 reactive machinery；
- `state` / `shared` 只有被 UI/runtime dependency 觀察時才生成 reactive tracking。

### A-030：Closure 支援 lexical capture

Voiles v0.1 支援 closure / nested function capture：

- `const` capture 保持 immutable；
- `state` capture 指向同一 mutable cell，而非 value copy；
- 同一 function invocation 建立的多個 closure capture 同一 local `state` 時共享該 cell；
- escaping closure 可延長 function-local captured environment lifetime，直到最後引用不可達；
- component/module-owned binding 的 capture 不得讓 closure 超越其 owner lifetime；compiler 必須拒絕可證明會逃逸到更長 lifetime 的 capture（例如寫入 `shared`）；
- owner unmount/cleanup 後不得存在可呼叫並存取已釋放 owner-local state 的 closure。

### A-031：Structural HTML block surface 使用 block/container-level tags；接受 attributes

Block/container-level structural HTML 名稱可直接作 block header，attributes 使用 named `=` arguments：

```voil
section(
	id="profile",
	class="panel"
):
	std.h2("Profile")
```

v0.1 初始 structural set：`html`, `body`, `header`, `main`, `footer`, `nav`, `section`, `article`, `aside`, `div`, `form`, `fieldset`, `figure`, `blockquote`, `ul`, `ol`, `table`, `details`, `dialog`。

較低層/leaf/content-oriented HTML element 使用 `std.*`。Structural set 由 `@voiles/html-base` metadata/version 維護，parser 將其視為 privileged structural names，不要求把所有 HTML tag 變成 lexer keyword。

### A-032：Import alias 定案為 item-local `as`

```voil
import A as X, B as Y from "./c.voil"
```

可混合 alias/non-alias item。

### A-033：Named argument separator 統一使用 `=`

Component、ordinary function named argument、struct construction、structural HTML attributes 都使用 `=`。

- component call：named-only；
- struct construction：named-only；
- ordinary function：可 positional，之後可接 named；一旦出現 named argument，後面不得再出現 positional；
- duplicate named、unknown named、missing required argument 都是 compile error；
- default parameter 同樣使用 `=`。

### A-034：Boolean operators 同時接受 word/symbol aliases

以下完全等義且 short-circuit：

```text
and == &&
or  == ||
not == !
```

Formatter canonical form 採 `and / or / not`，但 parser 永久接受兩組。

初始 precedence（高 -> 低）：member/index/call -> unary (`+ - not !`) -> `* / %` -> `+ -` -> comparison -> equality -> `and/&&` -> `or/||` -> assignment。

### A-035：`T?` 正式等價於 `Option<T>`

```voil
String? == Option<String>
```

`none` 是 empty option literal。Voiles 不提供 implicit nullable reference 或 implicit truthiness；Option 使用 pattern/match 或標準 Option API 解構。Optional chaining 可後續另行加入，不屬 v0.1 必要語法。

### A-036：Struct 採 immutable value model

```voil
struct User:
	id: Int
	name: String
	nickname: String? = none

const user = User(
	id=1,
	name="Ada"
)
```

- fields 預設且 v0.1 一律 immutable；
- construction named-only；
- field 可有 default；
- missing required / duplicate / unknown field 為 compile error；
- v0.1 不提供 mutable struct field；需要變更時建立新 value；
- copy/update sugar deferred，可先用 constructor 明確重建。

### A-037：Enum + exhaustive `match`

```voil
enum LoadState<T>:
	idle
	loading
	ready(T)
	failed(Error)
```

- enum case 在 enum 外使用 namespace qualification；
- 對已知 enum 的 `match` pattern 可省略 enum qualification；
- compiler 做 exhaustiveness 與 unreachable-pattern diagnostics；
- `_` 是 wildcard；
- payload pattern 支援 binding 與 nested enum/struct pattern；
- v0.1 不需要 pattern guard。

### A-038：Function/callback type 與 DOM event baseline

Function type：

```voil
fn(Int, String) -> Bool
```

Component callback parameter 使用 function type，例如：

```voil
component Button(
	onclick: fn() -> Void
):
	...
```

Native DOM handler 由 `@voiles/html-base` 提供具體 event types（如 `MouseEvent`, `InputEvent`, `KeyboardEvent`, `SubmitEvent`, `FocusEvent`, `PointerEvent`）。Handler signature 必須 type-check；optional callback 使用 `fn(...)->...?` / `Option<fn(...) -> ...>`。

### A-039：`@voiles/html-base` 為 metadata-driven Web surface；`std` 是 ordinary alias

`std` 只是官方 docs/formatter canonical alias，不是 reserved language namespace。

`@voiles/html-base` baseline：

- structural block metadata + `std.*` leaf/content elements；
- typed standard attributes/events；
- `class` 直接使用；HTML `for` 使用 `html_for`；
- `aria_*` / `data_*` 轉成 kebab-case attributes；
- boolean attribute `true` emit / `false` omit；
- normal text escaping；
- void element 拒絕 child block；
- browser-standard additions由 package metadata version 更新，不要求新增 language keyword。

### A-040：Scoped slot 與 first-class UI value 不進 v0.1

Slot parameter/scoped-slot Deferred。v0.1 不定義 `Ui`/`Node`/`Slot` first-class value type；UI structure 只存在於 compiler UI lowering context。

### A-041：Runtime `.voil` import cycle 在 v0.1 為 compile error

Ordinary runtime module graph 必須是 DAG，維持 deterministic init/cleanup ownership。Type-only dependency cycle 可以存在，但必須是無 runtime initialization edge 的 type import/reference。

### A-042：Observable module-top-level work 必須放在 `init:`

一般 top-level declaration initializer 必須可分析為無 observable side effect。Timer/socket/subscription/logging/DOM mutation 等 observable work 必須位於 module `init:`。

這使 whole-module tree-shaking 與 lifecycle ownership deterministic。

`shared` initializer 亦不得直接建立需要 cleanup 的 external resource。Application-lifetime resource 應由 application-root owned scoped module 的 `init:/cleanup:` 管理，`shared` 只存放可獨立存活的 shared data/cell。

### A-043：Non-component public API 使用 explicit `export`

Component 保持 automatic export；其他 module symbol 必須 explicit export：

```voil
export fn format_user(...):
	...

export const version = "1"
export state count = 0
export shared session = none
export struct User:
	...
export enum Status:
	...
```

Named symbol import 沿用同一 list + `as`：

```voil
import format_user, User as AccountUser from "./user.voil"
```

Namespace import：

```voil
import * as store from "./store.voil"
```

v0.1 不提供 default export。Mutable `state/shared` 只有 explicit export 時才成為 public mutation surface。

### A-044：JavaScript/npm interop 是 explicit unsafe/foreign boundary

Voiles package / `.voil` import 使用 ordinary `import`；原生 JS/npm module 使用 explicit foreign import marker：

```voil
extern import * as lib from "some-js-package"
```

Foreign value 預設是 opaque `JsValue`，除非標準 adapter/generated binding 提供 Voiles signature。`.d.ts` 可作 binding generation input，但不能繞過 `undefined` normalization、exception boundary 與 mutable object interop rules。v0.1 不提供 unconstrained `any`。

JS `undefined` 在 typed boundary 轉成 `none`/`Option`; JS throw / Promise rejection 必須轉成 explicit error result。

### A-045：Async/error 使用 `async fn` + `await` + `Result<T,E>` + `?`

```voil
async fn load_user(id: Int) -> Result<User, LoadError>:
	const response = await fetch_user(id)?
	return parse_user(response)?
```

- `?` 只在 compatible `Result`/Option-returning context 做 early propagation；
- v0.1 沒有一般 `throw` 作為原生 control flow；
- foreign JS throw/rejection 在 interop adapter 轉為 `Err`；
- async event callback 必須符合 declared async callback signature，不隱式丟棄 failure。

### A-046：List indexing 有 checked/optional 兩種 API

```voil
items[i]       # T, bounds failure -> deterministic runtime bounds error
items.get(i)   # Option<T>
```

Negative index 不具有 Python 式反向索引語意；負值視為 out-of-bounds。

### A-047：`container` 是 concrete generic block container

`container:` 預設 lowering 為 `<div>` 等 concrete generic block node，接受與 `div` 相同的 attributes/events。若 compiler 證明 layout/style/event/DOM/accessibility/lifecycle 完全等價才可 wrapper-eliminate。

Semantic tag 應使用對應 structural block，而不是 `container` semantic override。

### A-048：Route baseline / layout lifetime

File routing：

```text
index.voil          -> segment root
about.voil          -> /about
[id].voil           -> one dynamic segment
[...slug].voil      -> one-or-more catch-all
[[...slug]].voil    -> optional catch-all
```

Same path ambiguity/conflict 為 compile error。Reserved files：`_layout.voil`, `_404.voil`, `_error.voil`。

`param id: Int` 對 path segment 做 compile-known runtime conversion；conversion failure 視為該 typed route 不匹配並進入 nearest `_404` resolution，不注入 invalid value。Catch-all 預設 `List<String>`。

Nested `_layout.voil` instance 在同 subtree navigation 中保留 state，離開 subtree 時依一般 component/module lifecycle cleanup。`_error.voil` 採 nearest ancestor boundary；`_404.voil` 採 nearest segment fallback，最後回 root fallback。

Typed route-link API 必須能從 route schema 檢查 required params。

### A-049：HTML/URL safety 使用 nominal safe types

- `TrustedHtml` 不可由 plain `String` implicit conversion；
- runtime user content 必須經 sanitizer API 取得 `TrustedHtml`；unsafe raw conversion 若存在必須顯式標記；
- URL sinks 對 compile-time literal 做 scheme validation；runtime dynamic URL 使用 nominal `Url`/更窄 safe URL wrapper；
- safe URL parser 拒絕不允許的 dangerous scheme（例如 `javascript:`）；
- ordinary text sink 永遠 escaping。

### A-050：Compiler pipeline 採 lossless CST -> AST -> HIR -> typed HIR -> lowering IR

```text
source
-> tokens
-> lossless CST
-> syntax AST
-> resolved HIR
-> typed HIR
-> reactivity / identity / lifecycle lowering IR
-> HTML/CSS/JS codegen
```

CST 保留 comments/trivia/source spans；symbol resolution、module identity、component/slot semantics、type info 在 HIR；reactive dependency、keyed identity、mount/init ownership 在 lowering IR。

Diagnostics 至少包含 stable error code、primary span、optional secondary spans、expected/found/context，並支援 indentation/error recovery 以繼續分析後續 source。

### A-051：Dev HMR 以 declaration identity 相容性決定 state preservation

Vite integration 使用 framework/tooling HMR boundary，不把 Vite HMR API 暴露成 Voiles source 語法。

- component body-only compatible edit -> preserve instance/local state；
- component parameter signature、state declaration shape/type/order、identity/lifecycle shape 不相容 -> remount affected boundary；
- module `init`/dependency graph 改變需要先 cleanup 舊 owned module subtree，再 initialize replacement；
- `shared` cell 僅在 declaration identity + type compatible 時 preserve，否則 reset/reinitialize；
- compile error 時 dev server 保留 last successful module graph並顯示 diagnostic，修正後再套 update；
- HMR 僅為 dev semantics，不影響 production identity/lifecycle specification。

## Open

### O-001：CSS / Voiles layout integration details

保持 native CSS compatibility 為優先。需另行設計 style block/imported CSS、scoping/cascade、custom property、pseudo selector、media/container queries、animation/keyframes、typed subset 與 raw CSS escape hatch。此項不在本批次強行鎖 DSL。

### O-002：Exact foreign binding generator / npm package metadata

`extern import` safety baseline 已定案；`.d.ts`/schema 到 Voiles binding 的細節、package metadata 與 tooling 仍需 prototype。

### O-003：Application root/bootstrap exact surface

Application-lifetime resource 由 root-owned module 管理的原則已定；root bootstrap special file/API 的最終 surface 可與 routing/runtime implementation 一起決定。

### O-004：HMR compatibility hash implementation

HMR preservation semantics 已定；精確 declaration fingerprint、Vite plugin hook 與 invalidation propagation 留到 compiler/Vite implementation。

## Deferred

### X-001：Scoped slot / slot parameter

v0.1 不提供。

### X-002：First-class UI value type

v0.1 不提供 `Ui`/`Node`/`Slot` value model。

### X-003：CSS custom DSL

不在 v0.1 baseline 預先發明會限制 native CSS forward compatibility 的 DSL。

### X-004：General-purpose macro

v0.1 不提供。

### X-005：Operator overloading

v0.1 不提供 user-defined operator overloading。

### X-006：Rust-style borrow/lifetime syntax

不引入完整 borrow checker surface。

### X-007：SSR-only syntax

SSR/SSG 不先污染 v0.1 client-first core syntax。

### X-008：Ecosystem/package policy finalization

Project-root alias、Voiles package publishing、registry/package metadata、browser compatibility target 與最終 Vite plugin contract 等到 compiler prototype 後再鎖定。