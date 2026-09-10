# Voiles Language Specification

Status: `Draft`

Target: initial syntax planning before parser/compiler implementation.

Voiles source files use the `.voil` extension. The language targets front-end applications that compile to native HTML, CSS and JavaScript while keeping routing, reactivity, component identity and safety semantics available to the compiler.

This document separates accepted syntax from areas that still require prototype validation.

---

## 1. Core syntax direction

Accepted baseline:

```voil
# comment
import std from "@voiles/html-base"
import UserBtn from "../ui/components/UserBtn.voil"

state count = 0

fn click_btn():
	count += 1

header:
	...

main:
	container:
		std.h1("Hello world")
		UserBtn(
			text="just click",
			onclick=click_btn
		)

	std.p(count)
```

Explicit component declaration:

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
```

Core properties:

1. `:` opens a block.
2. indentation defines hierarchy.
3. `#` is comment syntax only.
4. import module paths are quoted strings.
5. function declarations use `fn`.
6. reusable UI declarations use `component`.
7. semantic page structure may appear directly at module level.
8. native HTML leaf APIs use the standard HTML namespace.
9. user components use function-like PascalCase invocation.
10. no HTML/XML closing tags are used.
11. ordinary mutable state is scoped; application-global mutation is explicit with `shared`.
12. every component invocation identity owns an independent component instance scope.
13. every top-level component declaration is automatically exportable.
14. unreachable component declarations are compiler dead-code-elimination candidates.
15. multiple components from one module use comma-separated imports.
16. component children use default/named `slot` projection and retain caller lexical scope.
17. v0.1 slots are optional and single-outlet/single-provision by name.
18. component parameters are immutable named inputs; defaults are evaluated per new instance.
19. reactive parameter updates preserve the existing component instance/local state.
20. non-repeated component identity comes from stable structural invocation position.
21. repeated component UI requires explicit `key`; implicit index identity is not used.
22. component-owned resources use one-shot `mount:` with nested `cleanup:`.

---

## 2. Source file model

A `.voil` file is a module.

A module may contain:

- imports;
- type declarations;
- `const`, `state` and top-level `shared` bindings;
- functions;
- component declarations;
- route parameter declarations;
- semantic HTML structural blocks;
- container blocks;
- native HTML API calls;
- user component invocations.

Voiles does not require file-role markers such as `page`, `module` or `component` on the first line. A component is identified explicitly by its `component` declaration.

Files under `app/` may become routes automatically.

```text
app/
	index.voil
	about.voil
	users/
		[id].voil
ui/
	components/
		UserBtn.voil
```

Expected routes:

```text
app/index.voil       -> /
app/about.voil       -> /about
app/users/[id].voil  -> /users/:id
```

Route paths are not repeated inside source files.

---

## 3. Blocks

Status: `Accepted`

Voiles uses significant indentation. Every block opener ends with `:`.

```voil
fn greet(name: String) -> String:
	return "Hello {name}"

component Greeting(name: String):
	container:
		std.p(name)

main:
	container:
		std.h1("Hello")
```

Ordinary blocks do not use `{}`.

The lexer/parser must eventually model indentation transitions deterministically and support multiline expression continuation without confusing expression indentation with block indentation.

---

## 4. Comments

Status: `Accepted`

Line comments begin with `#`:

```voil
# comment
```

`#` has no HTML id shorthand or other surface-language meaning.

Multiline comment syntax is not required for the first parser milestone.

---

## 5. Imports and scoped module identity

Module specifiers are quoted strings:

```voil
import std from "@voiles/html-base"
import UserBtn from "../ui/components/UserBtn.voil"
```

The same module-specifier string grammar is intended to cover:

```text
@voiles package
npm package
relative .voil module
project module path
```

`std` is the canonical documentation/formatter alias for `@voiles/html-base`. Whether it becomes a language-reserved namespace remains open.

Ordinary `.voil` module identity:

```text
same importer scope + same resolved module path
	-> same module instance

different importer scope + same resolved module path
	-> different module instance
```

Different aliases inside the same importer scope do not clone state:

```voil
import a from "./store.voil"
import b from "./store.voil"
```

`a` and `b` resolve to the same module instance in that importer scope.

### 5.1 Component imports

Every top-level `component Name(...):` declaration is automatically part of the module component export surface. There is no required `export component` syntax.

```voil
# buttons.voil
component UserBtn(text: String):
	...

component IconBtn(icon: String):
	...
```

Single component import:

```voil
import UserBtn from "./buttons.voil"
```

Multiple component import:

```voil
import UserBtn, IconBtn from "./buttons.voil"
```

Each item resolves the automatically exported component declaration with the same name. Component declaration names must be unique within one module.

Component alias syntax is still Draft. Current recommendation:

```voil
import UserBtn as PrimaryBtn, IconBtn as SmallIconBtn from "./buttons.voil"
```

Mixed aliased/non-aliased items would be valid if this form is accepted:

```voil
import UserBtn, IconBtn as SmallIconBtn from "./buttons.voil"
```

Non-component default/named/namespace import and JS/npm interop import semantics remain open.

---

## 6. Identifiers and naming

Naming style is not currently enforced by the parser.

Examples may use ordinary identifiers such as:

```voil
click_btn
```

PascalCase is the preferred component naming convention because it visually distinguishes user components from native HTML APIs and ordinary functions.

---

## 7. Primitive types

Initial type candidates:

```text
Bool
Int
Float
String
List<T>
Map<K, V>
Option<T>
Result<T, E>
```

Optional shorthand remains Draft:

```voil
String?
```

would mean:

```voil
Option<String>
```

Voiles does not expose JavaScript-style `undefined` as a normal language value.

---

## 8. Bindings and scope

Voiles uses lexical scope with nearest-binding resolution. Name lookup begins in the innermost scope and proceeds outward. Inner declarations may shadow outer declarations.

Redeclaration in the same lexical scope is a compile error. Declarations are not visible before their declaration point.

Current reduced binding model:

```voil
const title = "Voiles"
state count = 0
shared session = none
```

### 8.1 `const`

`const` is an immutable runtime binding.

```voil
const title: String = "Voiles"
```

Semantics:

- immutable;
- may be initialized from a runtime expression;
- does not imply compile-time evaluation;
- follows lexical scope;
- cannot be reassigned.

### 8.2 `state`

`state` is mutable scoped storage.

```voil
state count: Int = 0

fn click_btn():
	count += 1
```

Semantics:

- mutable;
- follows lexical scope;
- storage belongs to the owning function/module/component instance;
- retains its declared or inferred type after initialization;
- compiler-generated reactivity is used when UI/runtime dependencies observe it.

Function-local state is recreated per independent function invocation:

```voil
fn count_once() -> Int:
	state i = 5
	i += 1
	return i
```

Each call returns `6`.

### 8.3 `shared`

`shared` is explicit application-global mutable storage.

```voil
shared session = none
```

Accepted restrictions:

- mutable by definition;
- does not require an additional `state` keyword;
- one shared storage cell per declaration identity across module/component instances;
- only legal at module top level;
- compiler-generated reactivity is used when observed.

Invalid:

```voil
component Counter():
	shared value = 0
```

```voil
fn test():
	shared value = 0
```

`shared` is the explicit escape from normal scoped-state isolation.

### 8.4 Non-reactive local mutation

Still open.

Current direction may allow function-local `state` to lower to ordinary mutable local storage when no observer exists. A separate `mut` keyword is not accepted yet.

---

## 9. Functions

Status: `Accepted` for declaration marker and block form.

```voil
fn add(a: Int, b: Int) -> Int:
	return a + b
```

`fn` is required so declaration grammar does not collide with invocation grammar.

Async/error handling remains open:

```voil
async fn load_user(id: Int) -> Result<User, LoadError>:
	...
```

---

## 10. Structs and enums

Status: `Draft`

```voil
struct User:
	id: Int
	name: String
	email: String?
```

Candidate construction syntax:

```voil
const user = User(
	id=1,
	name="Ada",
	email=none
)
```

Enum candidate:

```voil
enum LoadState<T>:
	idle
	loading
	ready(T)
	failed(Error)
```

Pattern matching should be exhaustiveness-checked where possible.

---

## 11. Control flow

Status: `Draft` syntax using accepted block rules.

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

Repeated UI that creates component instances uses explicit key identity:

```voil
for user in users key user.id:
	UserCard(
		user=user
	)
```

Control flow should be usable directly inside page/component UI hierarchy rather than requiring template-only directives.

---

## 12. Page structure

Status: `Accepted` baseline.

A page does not require a `view:` wrapper or first-line `page` declaration.

```voil
header:
	std.h1("Account")

main:
	std.p("Content")

footer:
	std.p("Footer")
```

These structural blocks correspond to semantic HTML elements.

Initial candidates:

```text
header
main
footer
nav
section
article
aside
```

The exact syntax-level structural-name set remains open so the language core does not duplicate the entire HTML specification.

Sibling structural roots such as `header:` + `main:` + `footer:` are expected, but exact root validation remains open.

---

## 13. Standard HTML module

Status: `Accepted` concept, surface API Open.

Native HTML leaf elements are visually distinct from user components:

```voil
import std from "@voiles/html-base"

std.h1("Hello world")
std.p(count)
std.img(
	src=user.avatar,
	alt=user.name
)
```

The compiler/standard module can give these functions privileged HTML semantics for:

- correct element lowering;
- text escaping;
- typed attributes;
- boolean attributes;
- event types;
- accessibility diagnostics where appropriate.

The boundary between syntax-level structural HTML and `std.*` APIs should remain intentionally small.

---

## 14. User components

Status: `Accepted` declaration, export, parameter, instance, identity, lifecycle and slot baseline.

### 14.1 Declaration

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

Component header parameters define the public input surface. A separate `input` / `prop` keyword is not required.

```text
fn
	-> callable logic

component
	-> instantiable UI
```

### 14.2 Automatic component export

Every top-level component declaration is automatically exportable from its `.voil` module.

```voil
component UserBtn(...):
	...

component IconBtn(...):
	...
```

No `export component` syntax is required.

A module may contain multiple component declarations, each with a unique declaration name.

The compiler builds a component dependency graph from imports and invocations. Unreachable component declarations are eligible for dead-code elimination.

Component-local initialization/lifecycle occurs only when the component is instantiated. Unused component declarations must not create component state, run `mount`, or emit render output simply because the containing module exists.

Unrelated module-top-level side effects must still be preserved unless effect analysis proves the whole module removable.

### 14.3 Parameters and invocation

Status: `Accepted`.

Component parameters are immutable input bindings.

```voil
component UserCard(
	name: String,
	title: String = name,
	disabled: Bool = false
):
	...
```

Rules:

- no default -> required;
- default present -> optional;
- component call arguments are named-only;
- parameter bindings cannot be reassigned inside the component;
- defaults are evaluated per new component instance;
- initialization is left-to-right;
- defaults may reference only earlier initialized parameters.

Valid:

```voil
UserCard(
	name="Alice"
)
```

Invalid positional call:

```voil
UserCard("Alice") # compile error
```

Mutable component-owned copies require explicit state:

```voil
component Input(value: String):
	state current = value
```

`current` is initialized from the current input once for that instance; future updates to parameter `value` do not implicitly reset it.

Per-instance default example:

```voil
component Panel(id: String = create_id()):
	...

Panel()
Panel()
```

`create_id()` is evaluated independently for each new instance.

### 14.4 Instance state and reactive parameter updates

Every component invocation identity creates an independent component instance scope when first active.

```voil
UserBtn(text="A")
UserBtn(text="B")
```

If the component owns `state count = 0`, the two instances have independent count storage.

A component instance is also an importer scope. Ordinary stateful `.voil` modules imported by the component are independently instantiated per component instance unless the dependency uses `shared`.

Within one component instance, the same resolved dependency path still maps to one dependency module instance.

Reactive parameter updates preserve the existing component instance:

```voil
state name = "Alice"

UserCard(
	name=name
)

name = "Bob"
```

The same `UserCard` receives `"Bob"` while preserving local `state` and ordinary dependency instances.

Component-local state initializers run only when a new component identity is created.

### 14.5 Component identity

Status: `Accepted`.

Outside repeated UI, stable structural invocation position defines component identity.

Changing reactive values/parameters at the same position preserves the instance.

Conditional branch removal is an unmount boundary:

```voil
if show_profile:
	UserCard(
		name=user.name
	)
```

```text
false -> true
	-> create instance

true -> false
	-> cleanup component-owned mount resources
	-> unmount instance
	-> release ordinary local state/dependency instances

false -> true
	-> create a new instance
	-> mount new instance
```

Branch-local component state is not implicitly retained while the branch is inactive.

### 14.6 Keyed repeated UI

Status: `Accepted`.

Repeated UI that creates components must define explicit iteration identity:

```voil
for user in users key user.id:
	UserCard(
		user=user
	)
```

This is invalid:

```voil
for user in users:
	UserCard(
		user=user
	) # compile error
```

Voiles does not silently use list index/position as component identity.

Stable keys preserve instances across reorder. Removed keys unmount old instances. New keys create new instances. Changed keys replace identity.

For v0.1, key values are `String` or `Int`.

Duplicate key values in one repeated UI evaluation are runtime errors; the runtime must not guess which instance to reuse.

Algorithmic loops that do not create repeated component UI do not require keys.

### 14.7 Component lifecycle and cleanup

Status: `Accepted` for v0.1.

Component-owned resources use `mount:` with optional nested `cleanup:`.

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

Lifecycle rules:

- `mount:` executes exactly once when a new component instance becomes mounted/active;
- reactive prop/state updates do not rerun `mount:`;
- keyed reorder with unchanged key does not cleanup/remount;
- `cleanup:` is legal only inside `mount:`;
- `cleanup:` may capture lexical bindings declared in the enclosing mount block;
- `cleanup:` executes exactly once when that mounted instance is unmounted;
- conditional branch removal triggers cleanup + unmount;
- conditional re-entry creates a new instance and runs mount again;
- key removal/replacement triggers cleanup of the old instance;
- a new key creates/mounts a new instance;
- v0.1 does not define reactive `effect`, dependency arrays or automatic effect reruns.

Subscription example:

```voil
mount:
	const subscription = store.subscribe(update)

	cleanup:
		subscription.close()
```

The resource handle can remain lexical to `mount` instead of being stored in component `state`.

Cleanup of ordinary scoped module instances remains a separate module-lifecycle decision.

### 14.8 Children and slots

Status: `Accepted` baseline.

Component child blocks use the ordinary `:` + indentation model.

```voil
Card(title="Profile"):
	std.p(user.name)
	std.p(user.email)
```

Unwrapped children form default slot content.

Default outlet:

```voil
component Card(title: String):
	container:
		std.h2(title)
		slot
```

Named outlets:

```voil
component Modal():
	container:
		slot header
		slot
		slot footer
```

Caller:

```voil
Modal():
	slot header:
		std.h2("Confirm")

	std.p("Delete this item?")

	slot footer:
		Button(text="Cancel")
		Button(text="Delete")
```

`slot` is a compiler-level insertion point, not an ordinary function call.

Slot content preserves caller lexical scope:

```voil
const username = "Alice"

Card():
	std.p(username)
```

`username` resolves where `Card()` is invoked, not where the component is declared.

Caller slot content does not implicitly access component-local bindings, and the component does not implicitly gain caller-local bindings.

#### 14.8.1 Slot cardinality

Status: `Accepted` for v0.1.

All slot outlets are optional by default.

A component declaration may contain at most one default outlet and at most one outlet per named slot.

A component invocation may provide each named slot at most once.

Compile errors:

- duplicate slot outlets;
- duplicate named-slot provisions;
- unknown named slot supplied by caller;
- ordinary/default children supplied when callee has no default outlet.

A slot provision may contain any number of child nodes.

Required slots, repeated projection/cloning and scoped-slot parameters are not part of v0.1.

---

## 15. Container

Status: `Draft` semantics.

`container` creates an explicit block-level layout region.

```voil
main:
	container:
		std.h1("Hello")
		UserBtn(text="Click")
```

The intended model is closer to an HTML `<div>` than to a purely virtual layout construct.

Default lowering may therefore use a `<div>` or equivalent concrete block container.

Compiler wrapper elimination is only valid when removal preserves:

- layout;
- style scope;
- event behavior;
- DOM semantics;
- accessibility;
- component lifecycle behavior.

`container(...)` style/layout parameters remain unspecified until CSS integration is designed.

---

## 16. CSS and layout integration

Status: `Open`.

The design must answer at least:

1. Does `container(...)` accept a typed layout API, CSS-like properties, or both?
2. Are styles inline, in separate `.voil` blocks, imported CSS, or a combination?
3. How does component-local styling coexist with native cascade?
4. How are custom properties handled?
5. How are pseudo classes/elements handled?
6. How are media/container queries handled?
7. How are animations/keyframes handled?
8. How can new browser CSS features be used without waiting for a Voiles release?
9. Which parts can the compiler safely type-check?
10. What is the raw/native CSS escape hatch?

Until resolved, examples should not treat custom layout names such as `display=horizontal` as final syntax.

---

## 17. Routes

Filesystem routing remains part of project semantics.

```text
app/about.voil -> /about
app/users/[id].voil -> /users/:id
```

Candidate typed route parameter:

```voil
param id: Int
```

A route conversion failure must not inject an unchecked invalid value into the page. Exact failure behavior remains open.

Planned special files:

```text
_layout.voil
_404.voil
_error.voil
```

---

## 18. HTML safety

Normal text/data passed to standard HTML APIs is not trusted HTML.

```voil
std.p(user_input)
```

must escape HTML-sensitive content.

Raw HTML requires a separate trusted type or explicit unsafe boundary.

Candidate:

```text
TrustedHtml
```

A plain `String` must not implicitly satisfy it.

---

## 19. JavaScript interop

Status: `Open`.

Interop is required, but JS/npm modules are a type/safety boundary.

Required principles:

- JS values do not automatically gain trustworthy Voiles types;
- `undefined` must be normalized;
- thrown exceptions must be represented explicitly;
- mutable external objects require known interop semantics;
- unsafe direct interop, if provided, must be visibly explicit.

---

## 20. Current canonical examples

Components:

```voil
# buttons.voil
import std from "@voiles/html-base"

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

component Clock():
	state now = get_time()

	mount:
		const timer = start_timer():
			now = get_time()

		cleanup:
			timer.stop()

	std.p(now)

component Card(title: String):
	container:
		std.h2(title)
		slot
```

Page:

```voil
import std from "@voiles/html-base"
import UserBtn, Clock, Card from "../ui/components/buttons.voil"

state page_count = 0
shared session = none
const username = "Alice"

header:
	std.h1("Voiles")

main:
	UserBtn(text="First")
	UserBtn(text="Second")

	if show_clock:
		Clock()

	for user in users key user.id:
		UserBtn(text=user.name)

	Card(title="Profile"):
		std.p(username)

	std.p(page_count)
```

Independent `UserBtn()` identities create independent component instance state. Reactive parameter updates preserve an existing identity. `Clock` runs its mount logic once per mounted instance and cleans up on unmount. The keyed loop preserves component instances by `user.id`. `shared` remains application-global. `Card` projects caller-authored children without rebinding caller lexical scope.

---

## 21. Initial grammar sketch

Descriptive only:

```text
module                 := moduleItem* EOF

moduleItem             := importDecl
                        | typeDecl
                        | bindingDecl
                        | functionDecl
                        | componentDecl
                        | paramDecl
                        | structuralBlock
                        | expressionStatement

block                  := ":" NEWLINE INDENT statement* DEDENT
comment                := "#" commentText NEWLINE

importDecl             := "import" importItem ("," importItem)* "from" stringLiteral
importItem             := identifier importAlias?
importAlias            := "as" identifier

functionDecl           := "fn" identifier parameters returnType? block
componentDecl          := "component" PascalIdentifier componentParameters componentBlock
componentParameters    := "(" componentParameterList? ")"
componentParameter     := identifier typeAnnotation ("=" expression)?
componentBlock         := ":" NEWLINE INDENT componentItem* DEDENT

bindingDecl            := constDecl | stateDecl | sharedDecl
constDecl              := "const" identifier typeAnnotation? "=" expression
stateDecl              := "state" identifier typeAnnotation? "=" expression
sharedDecl             := "shared" identifier typeAnnotation? "=" expression

componentItem          := statement | slotOutlet | mountBlock
componentCall          := PascalIdentifier componentCallArguments childBlock?
componentCallArguments := "(" namedArgumentList? ")"

childBlock             := ":" NEWLINE INDENT childItem* DEDENT
childItem              := namedSlotBlock | uiStatement
namedSlotBlock         := "slot" identifier block
slotOutlet             := "slot" identifier?

mountBlock             := "mount" block
cleanupBlock           := "cleanup" block

keyedForUi             := "for" identifier "in" expression "key" expression block

structuralBlock        := structuralName callArguments? block
standardHtmlCall       := identifier "." identifier callArguments
callArguments          := "(" argumentList? ")"
namedArgument          := identifier "=" expression
```

Component imports use comma-separated item lists. `importAlias` with `as` remains Draft.

Component-call semantic validation must reject positional arguments, duplicate arguments, unknown argument names and missing required parameters.

Component parameter bindings are immutable. Defaults are evaluated for each new instance in declaration order and may reference only earlier parameters.

Semantic validation/runtime lowering must also:

- reject `sharedDecl` outside module top level;
- reject duplicate component declaration names;
- create independent scopes for new component invocation identities;
- preserve existing instances when only reactive parameter values change;
- use structural invocation identity outside repeated UI;
- unmount branch-local identities when their condition becomes inactive;
- reject repeated component UI without explicit `key`;
- type-check v0.1 keys as `String` or `Int`;
- detect duplicate runtime keys in one repeated UI evaluation;
- preserve component instance/state across keyed reorder;
- run `mount` once per new mounted instance identity;
- allow `cleanup` only nested under `mount`;
- preserve mount lexical bindings for cleanup capture;
- run cleanup exactly once at component unmount;
- lower slot content without rebinding caller lexical scope;
- validate slot cardinality and unknown slots;
- expose top-level component declarations automatically to component import resolution.

The grammar still needs final decisions for:

- multiline call indentation;
- exact structural block name table;
- expression precedence;
- CSS/style contexts;
- component alias finalization;
- slot parameter/type details;
- non-component import/export semantics.

---

## 22. Compiler/CST/runtime requirements

Accepted syntax requires:

- deterministic `NEWLINE`, `INDENT`, `DEDENT` behavior or equivalent parser model;
- `:` block opener recognition;
- line comments beginning with `#`;
- lossless comment/trivia preservation;
- source spans suitable for diagnostics;
- multiline parenthesized-expression continuation;
- recovery after malformed indentation;
- comma-separated import item parsing;
- distinction among structural block, component declaration, standard HTML call, ordinary call and component call;
- component named-only argument validation;
- component default dependency/order validation;
- parsing/lowering of child blocks and slots;
- caller lexical-scope preservation through slot lowering;
- slot cardinality validation;
- structural component identity generation;
- keyed repeated-component identity and reorder preservation;
- duplicate-key runtime validation;
- component lifecycle lowering for one-shot mount/cleanup;
- cleanup lexical capture preservation;
- component-local state preservation across reactive parameter updates/reorder;
- component symbol indexing per module;
- component dependency graph construction;
- dead-code elimination eligibility for unreachable component declarations.

---

## 23. Next decisions required

Parser-blocking or near-blocking priorities:

1. finalize `NEWLINE` / `INDENT` / `DEDENT` and multiline continuation rules;
2. finalize remaining `const/state/shared` lowering details and decide whether local `mut` is needed;
3. define top-level structural block grammar and valid structural names;
4. finalize component import alias syntax (`as` currently recommended);
5. finalize general named-argument `=` grammar for ordinary functions/struct construction;
6. define initial expression precedence;
7. define `@voiles/html-base` minimum API surface and event typing;
8. define scoped `.voil` module cleanup/lifetime separate from component `mount` resources;
9. define cyclic import and `shared` initialization behavior;
10. separately design CSS/layout integration before locking `container(...)` parameters;
11. define module top-level side-effect policy so tree-shaking guarantees are precise;
12. define non-component default/named/namespace import semantics;
13. define slot parameter/type semantics only if a concrete use case requires them.