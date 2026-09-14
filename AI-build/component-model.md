# Voiles Component Model

Status: `Accepted v0.1 baseline`.

此文件定義 component declaration/export/import、parameter/callback surface、instance identity/lifecycle、slot、module interaction 與 compiler reachability。

## 1. Explicit component declaration

Reusable UI 使用顯式 `component`：

```voil
component UserBtn(
	text: String,
	disabled: Bool = false
):
	state count = 0

	fn click():
		count += 1

	container:
		std.button(
			disabled=disabled,
			onclick=click
		)
		std.p(text)
		std.p(count)
```

`fn` = callable logic；`component` = instantiable UI。Compiler 不依 top-level render statement 猜 component。

## 2. Automatic component export

所有 top-level component 自動 export：

```voil
component UserBtn(...):
	...

component IconBtn(...):
	...
```

不需要 `export component`。同 module component name 必須唯一。

Single/multi import：

```voil
import UserBtn from "./buttons.voil"
import UserBtn, IconBtn from "./buttons.voil"
```

Alias 使用 item-local `as`：

```voil
import UserBtn as PrimaryBtn, IconBtn as SmallIconBtn from "./buttons.voil"
```

可混用 aliased/non-aliased items。

Component automatic export 不代表其他 module binding 自動 public；non-component symbol 必須 explicit `export`。

## 3. Component parameters

Component declaration header 是 public input surface：

```voil
component UserCard(
	name: String,
	title: String = name,
	disabled: Bool = false
):
	...
```

Rules：

- no default -> required；
- default -> optional；
- parameter immutable；
- component invocation named-only；
- positional component arg compile error；
- defaults 每個 new instance 獨立 evaluate；
- parameter/default initialization left-to-right；
- default 只能引用前面已初始化 parameter。

Valid：

```voil
UserCard(name="Alice")
```

Invalid：

```voil
UserCard("Alice")
```

Mutable copy 必須 explicit `state`：

```voil
component Input(value: String):
	state current = value
```

後續 `value` prop update 不自動 reset `current`。

## 4. Callback parameters

Function type syntax：

```voil
fn(Int, String) -> Bool
```

Component callback：

```voil
component Button(
	text: String,
	onclick: fn() -> Void
):
	std.button(onclick=onclick)
```

Optional callback：

```voil
component Button(
	onclick: (fn() -> Void)? = none
):
	...
```

Handler value 必須 type-compatible。Async handler 只有 parameter signature 明確允許 async callback 時才合法；async failure 不可被 runtime silently discard。

Native DOM event callback type 由 `@voiles/html-base` metadata 提供，例如 `MouseEvent`, `InputEvent`, `KeyboardEvent`, `SubmitEvent`, `FocusEvent`, `PointerEvent`。

## 5. Component instance identity

每個 component invocation identity 第一次 active 時建立獨立 instance scope。

```voil
UserBtn(text="A")
UserBtn(text="B")
```

Component-local `state` 不共享。Component instance 同時是 ordinary `.voil` dependency 的 importer scope。

Same component instance 中 same resolved module path -> same dependency module instance。

### 5.1 Reactive parameter update preserves identity

```voil
state name = "Alice"
UserCard(name=name)
name = "Bob"
```

同一 `UserCard` instance 接收新 parameter value；local state/dependencies 保留。Component-local initializer 不重跑。

### 5.2 Structural identity

非 repeated UI 由 compiled UI tree 中 stable structural invocation position 決定 identity。

Conditional branch inactive -> branch-local instance unmount；re-entry -> new instance。

### 5.3 Keyed repeated UI

```voil
for user in users key user.id:
	UserCard(user=user)
```

- repeated UI 中建立 component 必須有 `key`；
- key type `String | Int`；
- same key reorder preserve instance；
- removed/changed key destroys old identity；
- new key creates new instance；
- duplicate key runtime error；
- 不用 implicit index identity。

Ordinary algorithmic loop 不建立 component identity 時不要求 key。

## 6. Component lifecycle

Component-owned resource：

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

Semantics：

- `mount:` once per new mounted instance；
- prop/state update 不重跑；
- same-key reorder 不 cleanup/remount；
- `cleanup:` only nested in `mount:`；
- cleanup 可 capture mount locals；
- unmount 時 cleanup once；
- v0.1 沒有 reactive `effect` / dependency array。

Owned scoped module dependency 有自己的 `init:/cleanup:`，teardown 時 dependency modules post-order cleanup，再執行 component cleanup。

## 7. Component lexical / module state interaction

`component ...:` 是 lexical + instance scope。

```voil
state module_counter = 0

component A():
	state local = 0
	...
```

- module-level `state` 合法，屬於 containing scoped module instance；
- component-local `state` 屬於 component instance；
- `shared` 仍只能 module top-level；
- module-level state 不會因 module 中存在 component 就變成 per-component state。

若需要 per-instance mutable state，宣告在 component block；若需要 application-global cell，使用 `shared`。

## 8. Children and slots

Default children：

```voil
Card(title="Profile"):
	std.p(user.name)
```

Callee outlet：

```voil
component Card(title: String):
	container:
		std.h2(title)
		slot
```

Named slot：

```voil
component Modal():
	container:
		slot header
		slot
		slot footer
```

Caller：

```voil
Modal():
	slot header:
		std.h2("Confirm")

	std.p("Content")

	slot footer:
		Button(text="OK")
```

`slot` 是 compiler-level insertion point，不是 ordinary function call。

### 8.1 Slot lexical scope

Slot content 保留 caller lexical environment；callee 只決定 placement。

Caller-authored content 不會隱式取得 component locals；component 也不會透過 slot 取得 caller locals。

### 8.2 Slot cardinality

v0.1：

- outlet optional；
- default outlet 最多一個；
- 每個 named outlet 最多一個；
- caller 每個 named slot 最多 provision 一次；
- duplicate outlet/provision compile error；
- unknown named slot compile error；
- callee 沒有 default outlet 卻傳 ordinary children compile error。

### 8.3 Scoped slot Deferred

Slot parameter/scoped-slot 不在 v0.1。

例如未來可能考慮：

```voil
List(items=users):
	slot item(user):
		std.p(user.name)
```

但此 syntax/semantics 目前不成立。

### 8.4 UI value type Deferred

v0.1 不定義 `Ui`/`Node`/`Slot` first-class value，不允許以一般 variable/function return 任意搬運 compiler UI tree。UI structure 先保持 compiler-lowered structural context。

## 9. Native HTML interaction

Native HTML 主要透過普通 alias：

```voil
import std from "@voiles/html-base"
```

`std` 不是 reserved namespace。

Block/container-level structural names可直接使用 block syntax；leaf/content-oriented element 使用 `std.*`。

Structural block 可接受 named attributes：

```voil
section(
	id="profile",
	class="panel"
):
	std.h2("Profile")
```

Native event handler 的 exact type 來自 html-base metadata。

## 10. Unused component elimination

Unreachable component declaration 是 production DCE candidate：

```text
route entrypoints
-> reachable module/component symbols
-> component dependency graph
-> codegen set
```

Unused component 本身不能建立 state、run mount 或 render output。

Whole module removal 另外受 module `init:`/export reachability 限制。因 observable top-level work 必須放在 `init:`，effect boundary 可被 compiler 明確追蹤。

## 11. Grammar baseline

```text
componentDecl          := "component" PascalIdentifier componentParameters componentBlock
componentParameters    := "(" componentParameterList? ")"
componentParameter     := identifier typeAnnotation ("=" expression)?
componentBlock         := ":" NEWLINE INDENT componentItem* DEDENT

componentItem          := statement
                       | slotOutlet
                       | mountBlock

componentCall          := PascalIdentifier componentCallArguments childBlock?
componentCallArguments := "(" namedArgumentList? ")"

childBlock             := ":" NEWLINE INDENT childItem* DEDENT
childItem              := namedSlotBlock | uiStatement
namedSlotBlock         := "slot" identifier block
slotOutlet             := "slot" identifier?

mountBlock             := "mount" block
cleanupBlock           := "cleanup" block
keyedForUi             := "for" identifier "in" expression "key" expression block

componentImportItem    := PascalIdentifier importAlias?
importAlias             := "as" identifier
```

Semantic validation 必須：

- reject duplicate component names；
- component arguments named-only；
- reject missing/duplicate/unknown component args；
- parameter defaults left-to-right；
- preserve same instance on reactive prop update；
- repeated component UI require key；
- key type-check + duplicate runtime validation；
- slot caller lexical scope preservation；
- slot cardinality/unknown-slot validation；
- mount/cleanup nesting/lifetime validation；
- dependency module cleanup before component cleanup；
- callback signature type-check；
- owner-bound closure escape check。

## 12. Remaining component-adjacent work

Core component v0.1 semantics 已定。後續 implementation/prototype work：

- exact native event metadata generation；
- async callback ABI；
- HMR component compatibility fingerprint；
- CSS/style scoping design；
- scoped slot / UI value model 僅在後續有 concrete use case 時重新開啟。