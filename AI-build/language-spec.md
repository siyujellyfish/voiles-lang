# Voiles Language Specification

Status: `v0.1 baseline accepted; implementation not started`.

Voiles source uses `.voil`. The language targets front-end applications compiled to native HTML/CSS/JavaScript while preserving routing, type safety, fine-grained reactivity, component identity and deterministic resource lifecycle in the compiler/runtime model.

---

## 1. Core source shape

```voil
import std from "@voiles/html-base"
import UserBtn from "../ui/UserBtn.voil"

state count = 0

fn click_btn():
	count += 1

header:
	std.h1("Voiles")

main:
	UserBtn(
		text="Click",
		onclick=click_btn
	)
	std.p(count)
```

Core rules：

1. `:` opens blocks.
2. Significant indentation defines hierarchy.
3. `#` is a line comment.
4. Module specifiers are quoted strings.
5. Functions use `fn`.
6. Components use `component`.
7. No JSX/XML closing tags.
8. Mutation uses `state` or top-level `shared`; no `let/var/mut` in v0.1.
9. Native HTML uses structural blocks or `std.*`.
10. User components use PascalCase function-like invocation.
11. Components/modules/resources have explicit compiler-known lifetimes.

---

## 2. Lexical lines and indentation

Voiles follows Python-style logical-line / indentation-stack behavior for the block lexer.

### 2.1 Logical line

A non-continuation logical line ends with `NEWLINE`.

Blank, whitespace-only and comment-only logical lines are ignored for block structure and do not emit `NEWLINE`, `INDENT` or `DEDENT`.

Physical EOL forms are normalized before tokenization.

### 2.2 INDENT / DEDENT stack

The lexer begins with indentation stack `[0]`.

For each structural logical line：

- equal indentation -> no indentation token；
- greater indentation -> push new level + emit one `INDENT`；
- lower indentation -> target must be an existing stack level; pop and emit one `DEDENT` per removed level；
- EOF -> emit `DEDENT` until only zero remains。

Tabs use Python-style tab stops to the next multiple-of-eight column. Ambiguous tabs/spaces mixing is an indentation error. Formatter output uses tabs for block indentation.

### 2.3 Multiline continuation

Inside `(...)`, `[...]`, `{...}` expressions may span physical lines without structural newline/indentation tokens.

```voil
UserCard(
	name=user.name,
	disabled=(
		user.loading
		or user.deleted
	)
)
```

Continuation indentation is presentation only. Blank/comment lines are allowed inside continuation contexts.

v0.1 does not require backslash explicit line joining.

---

## 3. Comments

```voil
# comment
```

`#` has no HTML id shorthand meaning. Multiline comment syntax is not required for v0.1.

---

## 4. Bindings

```voil
const title = "Voiles"
state count = 0
shared session = none
```

### 4.1 `const`

Immutable runtime lexical binding. Runtime initializer allowed; not necessarily compile-time constant.

### 4.2 `state`

Ordinary mutable binding：

- lexical/instance scoped；
- type remains fixed after initialization；
- compiler generates reactivity only when observed；
- unobserved function-local state may lower to a plain mutable local。

All v0.1 ordinary mutation uses `state`：

```voil
fn sum(items: List<Int>) -> Int:
	state total = 0
	for item in items:
		total += item
	return total
```

### 4.3 `shared`

Application-global mutable cell per declaration identity：

- module-top-level only；
- shared across scoped module/component instances；
- reactive only when observed；
- ordinary module cleanup does not reset it。

A `shared` initializer may not directly create a cleanup-requiring external resource. Such resources need an owned module `init:/cleanup:` lifetime.

---

## 5. Scope and closure

Names resolve from innermost lexical scope outward. Inner declaration may shadow outer declaration. Same-scope redeclaration and use-before-declaration are compile errors.

### 5.1 Closure capture

```voil
fn make_counter():
	state i = 0

	fn next() -> Int:
		i += 1
		return i

	return next
```

- `const` capture remains immutable；
- captured `state` uses the same mutable cell；
- closures from one invocation share captured cells；
- escaping closure heap-promotes function-local captured environment；
- component/module-owner bindings cannot escape into a longer-lived storage than the owner；compiler rejects such escape。

---

## 6. Functions and arguments

```voil
fn add(a: Int, b: Int = 0) -> Int:
	return a + b
```

Ordinary functions support positional arguments followed by named arguments：

```voil
add(1, b=2)
```

After the first named argument, positional arguments are not allowed.

Duplicate named, unknown named and missing required arguments are compile errors.

Named/default separator is consistently `=` across functions, components, struct construction and structural HTML attributes.

### 6.1 Function types

```voil
fn(Int, String) -> Bool
```

Function/callback values are type-checked like other values.

---

## 7. Boolean operators and precedence

Equivalent aliases：

```text
and == &&
or  == ||
not == !
```

Both forms short-circuit. Formatter canonical output is `and / or / not`.

Initial precedence, high -> low：

```text
member / index / call
unary (+ - not !)
* / %
+ -
comparison (< <= > >=)
equality (== !=)
and / &&
or / ||
assignment
```

---

## 8. Primitive/core types

Initial core types：

```text
Void
Bool
Int
Float
String
List<T>
Map<K,V>
Option<T>
Result<T,E>
```

No JavaScript-style `undefined` in native Voiles.

### 8.1 Optional shorthand

```voil
String? == Option<String>
```

`none` is the empty option literal. Option is not implicitly truthy/falsy.

### 8.2 List indexing

```voil
items[i]      # T, checked; out-of-bounds runtime error
items.get(i)  # Option<T>
```

Negative index is out-of-bounds; it does not mean reverse indexing.

---

## 9. Struct

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

v0.1：

- fields immutable；
- constructor named-only；
- fields may have defaults；
- missing required / duplicate / unknown field -> compile error；
- mutable struct fields are not part of v0.1；
- copy/update sugar deferred。

---

## 10. Enum and match

```voil
enum LoadState<T>:
	idle
	loading
	ready(T)
	failed(Error)
```

```voil
match state:
	idle:
		std.p("Idle")
	loading:
		Spinner()
	ready(data):
		Results(data=data)
	failed(error):
		ErrorView(error=error)
```

Compiler performs exhaustiveness and unreachable-pattern diagnostics. `_` is wildcard. Payload/nested struct/enum patterns are allowed. Pattern guards are deferred.

Enum case references outside an enum-known match use namespace qualification.

---

## 11. Control flow

```voil
if user.isAdmin:
	show_admin()
else:
	show_user()
```

```voil
for item in items:
	process(item)
```

Control flow may appear directly inside UI hierarchy.

Repeated UI that creates components requires key identity：

```voil
for user in users key user.id:
	UserCard(user=user)
```

---

## 12. Module import/export

Module specifiers are quoted strings.

### 12.1 Component exports

Every top-level component automatically exports by declaration name.

```voil
component A():
	...

component B():
	...
```

```voil
import A, B as SmallB from "./components.voil"
```

Alias syntax is item-local `as`.

### 12.2 Non-component exports

Other public symbols require explicit `export`：

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

Named import：

```voil
import format_user, User as AccountUser from "./user.voil"
```

Namespace import：

```voil
import * as store from "./store.voil"
```

v0.1 has no default export.

### 12.3 Scoped module identity

```text
same importer scope + same resolved path
-> same module instance

different importer scope + same resolved path
-> different module instance
```

Aliases do not clone module instances.

### 12.4 Runtime cycle

Runtime `.voil` import graph must be acyclic in v0.1. A runtime dependency cycle is a compile error.

Type-only cycles are allowed if they do not create runtime initialization/ownership edges.

---

## 13. Module lifecycle and side effects

Module-owned resources use top-level `init:` with nested `cleanup:`：

```voil
state connected = false

init:
	const socket = open_socket("/chat")
	connected = true

	cleanup:
		socket.close()
```

- `init` once per new scoped module instance；
- same importer+path reuses instance；
- owner reactive update does not rerun init；
- cleanup only nested inside init；
- cleanup captures init locals；
- owner destruction tears dependencies down post-order；
- module-local state releases after module cleanup。

Dependency tree：

```text
Component
└─ module A
   └─ module B
```

Teardown：

```text
B cleanup
-> A cleanup
-> Component cleanup
-> Component local state release
```

Observable module-top-level work must live in `init:`. Ordinary declaration initializers must be effect-free/analyzably non-observable.

---

## 14. Components

### 14.1 Declaration

```voil
component UserBtn(
	text: String,
	disabled: Bool = false
):
	state count = 0
	...
```

Component parameter surface：

- immutable；
- no default -> required；
- default -> optional；
- call named-only；
- default per new instance；
- initialization left-to-right；
- default only references earlier initialized parameters。

### 14.2 Callback prop

```voil
component Button(
	text: String,
	onclick: fn() -> Void
):
	std.button(onclick=onclick)
```

Optional callback may use `Option<fn(...) -> ...>` / `(...)?`.

Native DOM event handler types come from `@voiles/html-base`, e.g. `MouseEvent`, `InputEvent`, `KeyboardEvent`, `SubmitEvent`, `FocusEvent`, `PointerEvent`.

### 14.3 Instance identity

Each invocation identity creates an independent instance scope. Component-local `state` is isolated.

Reactive prop update preserves the existing instance and local state.

Non-repeated invocation identity comes from stable structural position.

Conditional branch exit destroys branch-local identity; re-entry creates a new instance.

### 14.4 Repeated identity

```voil
for user in users key user.id:
	UserCard(user=user)
```

- key required when repeated UI creates components；
- key type `String | Int`；
- same key reorder preserves instance；
- removed/changed key removes old identity；
- duplicate runtime key is deterministic runtime error；
- no implicit index identity。

### 14.5 Mount lifecycle

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

`mount` once per new instance. `cleanup` once per unmount and only legal nested in mount. Prop/state update or same-key reorder does not remount.

Owned module dependencies clean up before component cleanup.

### 14.6 Module-level state in component module

Legal：

```voil
state module_counter = 0

component A():
	state local = 0
```

`module_counter` belongs to scoped module instance; `local` belongs to component instance.

---

## 15. Slots

Default children：

```voil
Card(title="Profile"):
	std.p(user.name)
```

Outlet：

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

```voil
Modal():
	slot header:
		std.h2("Confirm")

	std.p("Content")

	slot footer:
		Button(text="OK")
```

Slot content preserves caller lexical scope.

v0.1 cardinality：

- outlets optional；
- one default outlet；
- one outlet per named slot；
- caller one provision per named slot；
- duplicate/unknown slot compile error；
- ordinary child content requires default outlet。

Scoped slot/slot parameter is deferred. v0.1 does not define first-class `Ui`/`Node`/`Slot` values.

---

## 16. Native HTML surface

```voil
import std from "@voiles/html-base"
```

`std` is ordinary alias, only canonical by docs/formatter convention.

### 16.1 Structural blocks

Block/container-oriented structural set：

```text
html
body
header
main
footer
nav
section
article
aside
div
form
fieldset
figure
blockquote
ul
ol
table
details
dialog
```

Structural block accepts named attributes：

```voil
section(
	id="profile",
	class="panel"
):
	std.h2("Profile")
```

Structural set is maintained by html-base metadata/version rather than making all HTML tags permanent lexer keywords.

### 16.2 `std.*`

Leaf/content-oriented elements use std API：

```voil
std.h1("Hello")
std.p(user.name)
std.img(src=user.avatar, alt=user.name)
```

html-base baseline：

- typed standard attrs/events；
- `class` direct；
- `html_for` -> HTML `for`；
- `aria_*` / `data_*` -> kebab-case HTML attrs；
- Bool attrs true emit / false omit；
- text escaping；
- void elements reject child blocks；
- browser HTML evolution handled through html-base metadata package versions。

---

## 17. Container

`container:` is generic concrete block container, default div-like lowering：

```voil
container:
	std.p("content")
```

It accepts div-like attributes/events.

Semantic markup should use structural tags. Compiler may eliminate wrapper only if layout/style/event/DOM/accessibility/lifecycle behavior is proven unchanged.

CSS/style parameter surface is not locked here.

---

## 18. CSS integration status

CSS is a separate design phase. Requirements are fixed but exact syntax is Open：

- preserve native CSS compatibility/cascade；
- support native `.css` usage；
- component-local scope without blocking normal cascade；
- custom properties；
- pseudo classes/elements；
- media/container queries；
- keyframes/animations；
- forward compatibility with new browser CSS；
- optional compiler-checkable subset；
- raw/native CSS escape hatch。

v0.1 baseline deliberately does not invent a restrictive custom CSS DSL before this design pass.

---

## 19. Routes

Filesystem routing baseline：

```text
app/index.voil              -> /
app/about.voil              -> /about
app/users/[id].voil         -> /users/:id
app/docs/[...slug].voil     -> one-or-more catch-all
app/docs/[[...slug]].voil   -> optional catch-all
```

Conflicting route patterns are compile errors.

Reserved route files：

```text
_layout.voil
_404.voil
_error.voil
```

### 19.1 Typed params

```voil
param id: Int
```

Dynamic path segment is converted before page receives it. Conversion failure means this typed route does not match and enters nearest 404 fallback, never injects invalid `Int`.

Catch-all default type is `List<String>`.

### 19.2 Layout/error lifetime

Nested layout instance remains mounted while navigation stays inside the same route subtree and preserves local state. Leaving subtree tears it down using normal component/module lifecycle.

Nearest ancestor `_error.voil` handles route/component boundary failures. Nearest segment `_404.voil` handles no-match/typed-param failure; root fallback is last resort.

Typed route-link API must validate required params against generated route schema.

---

## 20. HTML / URL safety

Plain `String` is never trusted HTML.

`TrustedHtml` is nominal. Runtime user content must pass sanitizer API before becoming `TrustedHtml`. Any unsafe raw conversion must be explicit/visibly unsafe.

Compile-time URL literals are scheme-validated. Dynamic URL values for sensitive sinks use nominal `Url`/safe wrapper produced by a safe parser; dangerous schemes such as `javascript:` are rejected.

Ordinary text sinks always escape.

---

## 21. JavaScript/npm interop

Native JS/npm module is an explicit foreign boundary：

```voil
extern import * as lib from "some-js-package"
```

Foreign values default to opaque `JsValue` unless generated/standard bindings provide typed signatures.

- no unconstrained native `any`；
- `.d.ts` may be binding-generation input but does not bypass runtime boundary rules；
- JS `undefined` -> Option/none at typed boundary；
- JS throw/Promise rejection -> explicit `Err` through adapter；
- mutable external objects require explicit wrapper semantics。

Exact binding-generator/tooling details remain implementation work.

---

## 22. Async/error

```voil
async fn load_user(id: Int) -> Result<User, LoadError>:
	const response = await fetch_user(id)?
	return parse_user(response)?
```

v0.1 baseline：

- `async fn`；
- `await`；
- `Result<T,E>`；
- `?` early propagation for compatible Result/Option contexts；
- no ordinary native `throw` control flow；
- foreign exceptions/rejections normalized to explicit error results；
- async callback must match declared async callback type and may not silently drop failure。

---

## 23. Compiler representation

Pipeline：

```text
source
-> tokens
-> lossless CST
-> syntax AST
-> resolved HIR
-> typed HIR
-> reactivity/identity/lifecycle lowering IR
-> HTML/CSS/JS codegen
```

### 23.1 CST

Preserves：

- comments/trivia；
- exact source spans；
- lexical line/indentation structure；
- formatter-relevant syntax。

### 23.2 HIR

Owns：

- symbol resolution；
- type resolution；
- import/export identity；
- component/slot binding；
- module instance ownership；
- closure capture/lifetime constraints。

### 23.3 Lowering IR

Owns：

- reactive dependencies；
- component structural/key identity；
- module init/cleanup ownership；
- mount/cleanup；
- route lowering；
- safe HTML/URL sink lowering。

### 23.4 Diagnostics

Diagnostics need stable code, primary span, optional secondary spans, expected/found/context and recovery where safe. Parser should continue after malformed indentation/incomplete block when a deterministic recovery point exists.

---

## 24. Dev HMR semantics

Voiles uses Vite/framework tooling HMR boundaries underneath; no `import.meta.hot` syntax is exposed in `.voil` source.

- compatible component body edit -> preserve instance/local state；
- incompatible component parameter/state declaration/lifecycle shape -> remount affected boundary；
- module init/dependency graph change -> cleanup old owned subtree then init replacement；
- `shared` cell may preserve only when declaration identity + type remain compatible；
- compiler error -> keep last successful dev module graph running and show diagnostic；
- HMR state preservation is dev-only and does not redefine production lifecycle semantics。

Exact compatibility hash/fingerprint and Vite invalidation propagation are implementation work.

---

## 25. Initial grammar sketch

```text
module                 := moduleItem* EOF

moduleItem             := importDecl
                        | exportDecl
                        | typeDecl
                        | bindingDecl
                        | functionDecl
                        | componentDecl
                        | paramDecl
                        | initBlock
                        | structuralBlock

block                  := ":" NEWLINE INDENT statement* DEDENT

importDecl             := "import" importItems "from" stringLiteral
                        | "import" "*" "as" identifier "from" stringLiteral
                        | externImportDecl
importItems            := importItem ("," importItem)*
importItem             := identifier ("as" identifier)?

externImportDecl       := "extern" "import" ... "from" stringLiteral

exportDecl             := "export" (functionDecl | constDecl | stateDecl | sharedDecl | structDecl | enumDecl)

functionDecl           := asyncModifier? "fn" identifier parameters returnType? block
componentDecl          := "component" PascalIdentifier componentParameters componentBlock

bindingDecl            := constDecl | stateDecl | sharedDecl
constDecl              := "const" identifier typeAnnotation? "=" expression
stateDecl              := "state" identifier typeAnnotation? "=" expression
sharedDecl             := "shared" identifier typeAnnotation? "=" expression

structDecl             := "struct" identifier block
enumDecl               := "enum" identifier genericParams? block

componentCall          := PascalIdentifier "(" namedArgumentList? ")" childBlock?
childBlock             := ":" NEWLINE INDENT childItem* DEDENT
namedSlotBlock         := "slot" identifier block
slotOutlet             := "slot" identifier?

mountBlock             := "mount" block
initBlock              := "init" block
cleanupBlock           := "cleanup" block

keyedForUi             := "for" identifier "in" expression "key" expression block

structuralBlock        := structuralName callArguments? block
standardHtmlCall       := identifier "." identifier callArguments
namedArgument          := identifier "=" expression
```

Exact expression parser should implement the accepted precedence rather than relying on this sketch.

---

## 26. Deferred / Open

### Open implementation/prototype details

- closure escape analysis algorithm；
- effect/resource classification；
- foreign `.d.ts` binding generation；
- async callback/browser adapter ABI；
- application bootstrap special surface；
- HMR compatibility fingerprints；
- html-base metadata generation/versioning。

### Separate CSS design phase

CSS syntax/scoping remains intentionally open under the compatibility requirements in §18.

### Deferred beyond v0.1

- scoped slot / slot parameter；
- first-class UI value；
- struct update sugar；
- match guard；
- macro system；
- user operator overloading；
- Rust-style borrow/lifetime syntax；
- SSR-only syntax；
- ecosystem/package policy finalization。

---

## 27. Implementation gate

The language P0 baseline is now sufficiently specified for a parser/compiler prototype, but implementation has not started.

Before implementation：

1. create a new implementation branch；
2. consult current official docs for each external package before selecting/using it；
3. use current project-compatible package versions；
4. add/update `AI-build/compiler.md`, `type-system.md`, `routing.md`, `runtime.md`, `security.md`, `known-issues.md` as implementation contracts；
5. do not merge to `main` without explicit authorization；
6. merge to `main` only with squash commit。