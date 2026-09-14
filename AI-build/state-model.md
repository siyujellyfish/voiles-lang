# Voiles State / Scope Model

Status: `Accepted v0.1 baseline`.

此文件定義 `const` / `state` / `shared`、lexical scope、closure capture、scoped module identity/lifecycle、component identity/lifecycle 與 cleanup ownership。

## 1. Lexical scope

Name resolution 從目前 lexical scope 向外尋找最近 declaration。Inner declaration 可 shadow outer declaration。

```voil
state i = 1

fn count():
	i += 1

fn count_2():
	state i = 5
	i += 1
```

Same-scope redeclaration 與 use-before-declaration 都是 compile error。

## 2. Binding kinds

### 2.1 `const`

```voil
const title = "Voiles"
```

- immutable runtime binding；
- initializer 可以是 runtime expression；
- 不代表 compile-time constant；
- lexical scope；
- 不可 reassign。

### 2.2 `state`

```voil
state count = 0
```

`state` 是 v0.1 唯一 ordinary mutable binding：

- mutable；
- lexical scope；
- storage lifetime 綁定 owning function/module/component environment；
- mutation 後 type 固定；
- 被 UI/runtime dependency 觀察時 compiler 才建立 reactive tracking；
- 沒有 observer 的 function-local `state` 可 lower 成普通 mutable local。

Voiles v0.1 不提供 `let` / `var` / `mut`。Algorithmic local mutation 仍寫 `state`：

```voil
fn sum(items: List<Int>) -> Int:
	state total = 0

	for item in items:
		total += item

	return total
```

### 2.3 `shared`

```voil
shared session = none
```

- mutable；
- module top-level only；
- one application-global cell per declaration identity；
- across scoped module/component instances shared；
- 只有被觀察時才生成 reactive tracking；
- ordinary scoped-module cleanup 不 reset/destroy shared cell。

Invalid：

```voil
component Counter():
	shared i = 0
```

```voil
fn test():
	shared i = 0
```

## 3. Function invocation and closure capture

普通 function-local `state` 每次 function invocation 建立自己的 cell：

```voil
fn count_once() -> Int:
	state i = 5
	i += 1
	return i
```

每次獨立 call 都從 `5` 開始。

### 3.1 Closure capture

Voiles v0.1 支援 nested function/closure capture：

```voil
fn make_counter():
	state i = 0

	fn next() -> Int:
		i += 1
		return i

	return next
```

Rules：

- captured `const` 保持 immutable；
- captured `state` 指向原 cell，不做 value copy；
- 同一 invocation 產生的多個 closure 若 capture 同一 `state`，共享該 cell；
- escaping closure 會讓 function-local captured environment heap-promote，直到最後 closure reference unreachable；
- closure capture 不改變 declared/inferred type。

### 3.2 Owner-bound capture

Component/module-owned binding 不能因 closure escape 而延長 owner lifecycle。

例如 component callback 可以 capture component-local state：

```voil
component Counter():
	state count = 0

	fn click():
		count += 1

	std.button(onclick=click)
```

但把 `click` 寫入 application-global `shared` 或其他可證明比 component 活得更久的 storage 必須 compile error。

Compiler 必須做 owner-lifetime escape check：owner cleanup/unmount 後不得留下仍可呼叫且能存取已釋放 owner-local state 的 closure。

## 4. Scoped `.voil` module identity

Ordinary `.voil` module 不是 application-global ESM singleton。

```text
same importer scope + same resolved path
	-> same module instance

different importer scope + same resolved path
	-> different module instance
```

同 importer scope 中不同 alias 不 clone module instance：

```voil
import * as a from "./store.voil"
import * as b from "./store.voil"
```

`a` / `b` 指向同一 scoped instance。

## 5. Scoped module lifecycle

Ordinary scoped module instance 由 importer scope 擁有。Module-owned resource 使用 module-top-level `init:` + nested `cleanup:`：

```voil
state connected = false

init:
	const socket = open_socket("/chat")
	connected = true

	cleanup:
		socket.close()
```

Semantics：

```text
new module instance
	-> dependencies initialize
	-> init once

same importer + same path
	-> reuse instance
	-> init does not rerun

owner reactive update
	-> preserve instance

owner destruction
	-> dependency modules cleanup recursively
	-> this module cleanup once
	-> ordinary module-local state release
```

`cleanup:` 只能出現在該 module `init:` 內，可 capture `init:` lexical locals。

不同 alias 指向同一 instance 時，init/cleanup 都只執行一次。

### 5.1 Dependency-first teardown

```text
Component
└─ module A
   └─ module B
```

Teardown：

```text
module B cleanup
-> module A cleanup
-> Component cleanup
-> Component state release
```

所有 ordinary scoped dependency chain 都採 post-order teardown。

### 5.2 Runtime import cycle

v0.1 ordinary runtime module graph 必須是 DAG。

```text
A -> B -> C -> A
```

若 edge 需要 runtime initialization/state access，compiler 直接報 cyclic-runtime-import error。

Type-only dependency cycle 可存在，前提是不建立 runtime ownership/init edge。這維持 deterministic init/cleanup order。

## 6. Module top-level side effects

Observable top-level work 必須位於 `init:`：

```voil
init:
	const socket = open_socket("/chat")
	cleanup:
		socket.close()
```

普通 top-level declaration initializer 必須可分析為沒有 observable side effect。

Logging、timer、subscription、WebSocket、DOM mutation、network startup 等都不得偷偷發生在 ordinary top-level initializer。

這使 module tree-shaking 與 lifecycle ownership deterministic。

## 7. `shared` resource restriction

Ordinary `shared` cell 不負責 application-global external resource cleanup。

```voil
shared current_user = none
```

合法。

這類 v0.1 不允許作為 ordinary shared initializer：

```voil
shared socket = open_socket("/chat") # compile error: cleanup-requiring resource
```

Application-lifetime external resource 應由 application-root owned scoped module 的 `init:/cleanup:` 管理；`shared` 可以保存共享 data/reference，但不能繞過 resource owner lifecycle。

## 8. Component parameters and instance scope

Component parameters 是 immutable input bindings：

```voil
component UserBtn(
	text: String,
	disabled: Bool = false
):
	state count = 0
```

- no default -> required；
- default -> optional；
- component call named-only；
- defaults per new component instance evaluate；
- initialization left-to-right；
- default 只能引用前面已初始化 parameter。

每個 component invocation identity 有獨立 instance scope。

### 8.1 Reactive parameter update

```voil
state name = "Alice"

UserCard(name=name)
name = "Bob"
```

同一 component instance 收到新 immutable parameter value；component-local `state` 與 ordinary scoped dependency instances 保留。

```voil
component Input(value: String):
	state current = value
```

`current` 只在 new instance initialization 讀取當下 `value`；後續 prop update 不自動 reset。

## 9. Component identity

### 9.1 Structural identity

非 repeated UI 的穩定 invocation structural position 決定 identity。Reactive value change 不 recreate instance。

### 9.2 Conditional lifetime

```voil
if show_profile:
	UserCard(name=user.name)
```

```text
false -> true
	-> create instance
	-> initialize dependencies
	-> mount

true -> false
	-> dependencies cleanup post-order
	-> component cleanup
	-> release local state

false -> true
	-> new instance
```

Branch inactive 時不 hidden-retain local state。

### 9.3 Repeated UI

```voil
for user in users key user.id:
	UserCard(user=user)
```

- repeated component UI 必須 explicit key；
- no implicit index identity；
- v0.1 key type `String | Int`；
- same key reorder -> preserve instance；
- removed/changed key -> destroy old identity；
- new key -> new instance；
- duplicate key in one render evaluation -> deterministic runtime error。

Algorithmic loop 沒有 component identity 時不要求 `key`。

## 10. Component mount/cleanup

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

- new mounted instance -> `mount` once；
- prop/state update -> no remount；
- same-key reorder -> no remount；
- `cleanup` only inside `mount`；
- cleanup captures mount locals；
- unmount -> cleanup once；
- owned scoped modules cleanup before component cleanup。

v0.1 不提供 reactive `effect` / dependency array。

## 11. Module-level `state` in component-declaring modules

合法。

```voil
state module_counter = 0

component A():
	...

component B():
	...
```

`module_counter` 屬於 containing scoped module instance，不是自動 component-local，也不是 application-global。若需要 per-component instance state，必須在 `component` block 內宣告；若需要跨 module instance application-global state，使用 `shared`。

## 12. Slot lexical scope

Slot content 由 caller lexical environment 擁有；callee 只決定 insertion point。

```voil
const username = "Alice"

Card():
	std.p(username)
```

`username` 在 caller scope resolution。Caller slot content 不會隱式看到 component locals；component 也不會透過 projection 取得 caller locals。

Scoped-slot/slot parameter Deferred。

## 13. Summary

```text
const
	immutable lexical binding

state
	ordinary mutable lexical binding
	reactive only when observed
	no separate mut/let/var

shared
	module-top-level application-global mutable cell
	ordinary module cleanup does not destroy it
	cannot directly own cleanup-requiring external resource

function closure
	capture const/state lexically
	state captured by cell
	function-local captured env may outlive call
	owner-bound captures cannot escape owner lifetime

scoped module
	owned by importer
	init once
	DAG runtime dependencies
	dependency-first cleanup

component
	structural/key identity
	owned scoped modules
	mount once / cleanup once

slot
	caller lexical scope
```

## Remaining runtime work

Core v0.1 semantics above are Accepted。Implementation/prototype 尚需驗證：

- precise closure escape-analysis diagnostics；
- resource-type/effect classification used to reject cleanup-requiring `shared` initializers；
- application bootstrap special surface；
- HMR fingerprint/invalidation implementation；
- foreign JS binding generator details。