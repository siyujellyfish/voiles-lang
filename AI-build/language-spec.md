# Voiles Language Specification

Status: `Draft`

Target: initial syntax planning before parser/compiler implementation.

Voiles source files use the `.voil` extension. The language targets front-end applications that compile to native HTML, CSS and JavaScript modules while keeping routing, reactivity and safety semantics available to the compiler.

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
	container():
		std.h1("Hello world")
		UserBtn(
			text="just click",
			onclick=click_btn
		)

	std.p(count)
```

Component declarations are explicit:

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
4. import module paths are always quoted strings.
5. function declarations use `fn`.
6. component declarations use `component`.
7. semantic page structure may appear directly at module level.
8. native HTML leaf APIs are accessed through the standard HTML namespace.
9. user components use function-like PascalCase invocation.
10. no HTML/XML closing tags are used.
11. ordinary mutable state is scoped; application-global mutation must be explicit with `shared`.
12. every component invocation creates an independent component instance scope.
13. every declared component is automatically exportable; source code does not require an `export` keyword.
14. unused component declarations are eligible for compiler dead-code elimination.
15. multiple components from one module use a comma-separated import list.
16. component children use default/named `slot` projection while retaining caller lexical scope.
17. v0.1 slots are optional and single-outlet/single-provision by name.
18. component parameters are immutable named inputs; defaults are evaluated per invocation.
19. reactive parameter updates preserve the existing component instance and its local state.

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

Example:

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

The lexer/parser must eventually emit or model indentation transitions in a deterministic way and support multiline expression continuation without confusing expression indentation with block indentation.

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

Module specifiers are always quoted strings:

```voil
import std from "@voiles/html-base"
import UserBtn from "../ui/components/UserBtn.voil"
```

The same string grammar is intended to cover:

```text
@voiles package
npm package
relative .voil module
project module path
```

Component single/multi-symbol import syntax is defined below. Component alias syntax and non-component default/named/namespace import grammar remain partially open.

`std` is the current canonical alias used by Voiles examples for `@voiles/html-base` so native HTML APIs remain visually distinct from user components.

Ordinary `.voil` modules use scoped module identity:

```text
same importer scope + same resolved module path
	-> same module instance

different importer scope + same resolved module path
	-> different module instance
```

Different aliases inside the same importer scope do not clone state.

```voil
import a from "./store.voil"
import b from "./store.voil"
```

`a` and `b` resolve to the same module instance inside that importer scope.

### 5.1 Component imports

Every `component Name(...):` declaration is automatically part of the module's component export surface. There is no required `export component` syntax.

Example module:

```voil
# buttons.voil
component UserBtn(text: String):
	...

component IconBtn(icon: String):
	...
```

A single component may be imported by declaration name:

```voil
import UserBtn from "./buttons.voil"
```

Multiple components from the same module use a comma-separated import list:

```voil
import UserBtn, IconBtn from "./buttons.voil"
```

Each item resolves the automatically exported component declaration with the same name. Component names must be unique within one module.

Component alias syntax is not yet Accepted. The current recommendation is item-local `as`:

```voil
import UserBtn as PrimaryBtn, IconBtn as SmallIconBtn from "./buttons.voil"
```

This also permits mixed aliased and non-aliased items:

```voil
import UserBtn, IconBtn as SmallIconBtn from "./buttons.voil"
```

The `as` form is preferred over `=` because `=` is already used by assignment/named arguments, and preferred over `{ ... }` because the import list does not otherwise require wrapper punctuation.

This is called automatic/implicit component export in the language specification. It is not JavaScript's single-default-export rule.

---

## 6. Identifiers and naming

Naming style is not currently enforced by the parser.

Examples may use:

```voil
click_btn
```

PascalCase remains the preferred component naming convention because it visually distinguishes user components from native HTML APIs and ordinary functions.

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

Voiles uses lexical scope with nearest-binding resolution. Name lookup begins in the innermost scope and proceeds outward. Inner declarations may shadow outer declarations. Redeclaration in the same lexical scope is a compile error, and declarations are not visible before their declaration point.

The current reduced binding model is:

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

Function-local state is recreated for each function invocation:

```voil
fn count_once() -> Int:
	state i = 5
	i += 1
	return i
```

Each independent call returns `6`.

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

`shared` is the explicit escape from normal scoped state isolation.

### 8.4 Non-reactive local mutation

Still open.

The current direction may allow function-local `state` to lower to ordinary mutable local storage when no observer exists. A separate `mut` keyword is not accepted yet.

---

## 9. Functions

Status: `Accepted` for declaration marker and block form.

```voil
fn add(a: Int, b: Int) -> Int:
	return a + b
```

`fn` is required so declaration grammar does not collide with invocation grammar.

Example without explicit return value:

```voil
fn click_btn():
	count += 1
```

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

Status: `Draft`, using the accepted block syntax.

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

```voil
match state:
	idle:
		std.p("Idle")
	loading:
		Spinner()
	ready(data):
		Results(data=data)
```

Control flow should be usable directly inside page/component UI hierarchy rather than introducing template-only directives.

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

Initial candidates include:

```text
header
main
footer
nav
section
article
aside
```

The exact list and whether every semantic HTML container is syntax-level must be defined with `@voiles/html-base` so the language core does not unnecessarily duplicate the entire HTML specification.

A page may plausibly contain sibling structural roots such as `header:` + `main:` + `footer:`; exact root validation remains open.

---

## 13. Standard HTML module

Status: `Accepted` concept, surface API Open.

Native HTML leaf elements are intentionally visually different from user components:

```voil
import std from "@voiles/html-base"

std.h1("Hello world")
std.p(count)
std.img(
	src=user.avatar,
	alt=user.name
)
```

The namespace approach avoids making every HTML tag a language keyword.

The compiler/standard module can still give these functions privileged HTML semantics for:

- correct element lowering;
- text escaping;
- typed attributes;
- boolean attribute behavior;
- event types;
- accessibility diagnostics where appropriate.

The boundary between syntax-level structural HTML (`main:`) and `std.*` HTML APIs must be kept intentionally small.

---

## 14. User components

Status: `Accepted` declaration, automatic export, parameter, invocation, instance-isolation and slot baseline.

### 14.1 Declaration

A component is declared explicitly:

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

The component declaration header defines its input parameters. A separate `input` or `prop` declaration keyword is not required.

Conceptually:

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

No additional syntax is required:

```voil
export component UserBtn(...): # not required
```

A module may contain multiple component declarations. Component declaration names must be unique within that module.

Multiple exported components may be imported together:

```voil
import UserBtn, IconBtn from "./buttons.voil"
```

The compiler builds the component dependency graph from imports and invocations. A component declaration that is unreachable from application entry points is eligible for tree-shaking/dead-code elimination.

Component-local initialization occurs only when the component is instantiated, so an unused component should not itself force runtime component state or render output into the bundle.

If the containing module has independent top-level side effects, removing an unused component does not by itself authorize removing those unrelated side effects. Top-level side-effect policy remains a separate module-system concern.

### 14.3 Parameters and invocation

Status: `Accepted`.

Component parameters are immutable input bindings declared in the component header.

```voil
component UserCard(
	name: String,
	title: String = name,
	disabled: Bool = false
):
	...
```

A parameter without a default value is required. A parameter with a default value is optional.

Component invocation is named-argument-only:

```voil
UserCard(
	name="Alice"
)
```

Positional component invocation is invalid:

```voil
UserCard("Alice") # compile error
```

Component parameters cannot be reassigned inside the component:

```voil
component UserBtn(text: String):
	text = "changed" # compile error
```

If mutable component-owned data is needed, the component must create explicit `state`:

```voil
component Input(value: String):
	state current = value

	fn clear():
		current = ""
```

Parameter default expressions are evaluated independently for each component invocation. Initialization proceeds left-to-right, so a default may reference only parameters already initialized earlier in the parameter list.

Valid:

```voil
component Avatar(
	name: String,
	alt: String = name
):
	...
```

Invalid:

```voil
component Avatar(
	alt: String = name, # compile error
	name: String
):
	...
```

Per-invocation default evaluation means:

```voil
component Panel(id: String = create_id()):
	...

Panel()
Panel()
```

evaluates `create_id()` separately for the two component instances.

### 14.4 Component instance state and reactive parameter updates

Every component invocation identity creates a component instance scope when it first becomes active.

If `UserBtn` contains:

```voil
state count = 0
```

then:

```voil
UserBtn(text="A")
UserBtn(text="B")
```

creates two independent `count` storage cells.

Conceptually:

```text
UserBtn A
	├─ text = "A"
	└─ count = 0

UserBtn B
	├─ text = "B"
	└─ count = 0
```

Updating A must not update B unless the component intentionally reaches a `shared` declaration.

A component instance is also an importer scope. Therefore ordinary stateful `.voil` modules imported by the component are independently instantiated per component invocation.

Within one component instance, the same resolved module path still maps to one dependency module instance.

`shared` remains application-global across all component instances.

If a caller expression supplying a component parameter changes reactively, the existing component instance receives the new immutable parameter value. A parameter value change alone does not destroy and recreate the component.

```voil
state name = "Alice"

UserCard(
	name=name
)

name = "Bob"
```

The existing `UserCard` instance observes `name` changing to `"Bob"` while preserving component-local `state` and ordinary scoped dependency instances.

Component-local state initialization happens only when a new component instance is created. Therefore:

```voil
component Input(value: String):
	state current = value
```

uses `value` to initialize `current` once for that instance. Later parameter updates do not implicitly reset `current`.

The exact definition of invocation identity across conditional branches and repeated/list UI remains open.

### 14.5 Children and slots

Status: `Accepted` baseline.

Component invocation child blocks use the same `:` + indentation model as other Voiles blocks.

```voil
Card(title="Profile"):
	std.p(user.name)
	std.p(user.email)
```

Ordinary child statements that are not inside a named-slot block form the default slot content.

A component renders its default children with the compiler-level `slot` outlet:

```voil
component Card(title: String):
	container:
		std.h2(title)
		slot
```

`slot` is not an ordinary function call. It marks the insertion point for caller-provided UI content.

Named slots use the same keyword in both declaration and invocation contexts.

Component outlets:

```voil
component Modal():
	container:
		slot header
		slot
		slot footer
```

Caller-provided slot content:

```voil
Modal():
	slot header:
		std.h2("Confirm")

	std.p("Delete this item?")

	slot footer:
		Button(text="Cancel")
		Button(text="Delete")
```

The unwrapped `std.p(...)` targets the default slot. `slot header:` and `slot footer:` target their corresponding named outlets.

Slot content preserves caller lexical scope:

```voil
const username = "Alice"

Card():
	std.p(username)
```

`username` resolves in the scope where `Card()` is invoked, not in the lexical scope where `component Card` is declared. The component controls placement only.

Caller-authored slot content cannot implicitly access component-local `const`, `state` or helper functions. If values need to cross from a component instance into slot content, that requires a later explicit slot-parameter/scoped-slot design.

#### 14.5.1 Slot cardinality

Status: `Accepted` for v0.1.

All slot outlets are optional by default. If no matching content is provided, the outlet renders nothing.

```voil
component Card():
	container:
		slot

Card() # valid
```

A component declaration may contain at most one default slot outlet and at most one outlet for each named slot. Duplicate outlets are compile errors.

A component invocation may provide each named slot at most once. Duplicate named-slot provisions are compile errors.

Caller-provided named slots must match declared named outlets. Unknown named slots are compile errors.

Ordinary child content requires a default `slot` outlet on the callee. Passing default children to a component without one is a compile error.

A single slot provision may contain any number of child UI nodes. The one-per-name rule applies to the slot outlet/provision, not to the contents of that slot.

Required slots, repeated projection/cloning and slot parameters/scoped slots are not part of the v0.1 baseline.

---

## 15. Container

Status: `Draft` semantics.

`container` exists to create an explicit block-level layout region.

```voil
main:
	container:
		std.h1("Hello")
		UserBtn(text="Click")
```

The intended model is closer to an HTML `<div>` than to a purely virtual layout construct.

Baseline expectation:

```text
container -> concrete block/layout container
```

The default lowering may therefore be a `<div>` or equivalent element.

Compiler wrapper elimination should only happen when it can prove removal preserves:

- layout;
- style scope;
- event behavior;
- DOM semantics;
- accessibility;
- component lifecycle behavior.

The parameters accepted by `container(...)` are deliberately unspecified until CSS integration is designed.

---

## 16. CSS and layout integration

Status: `Open`.

CSS is too broad to lock into a simplified custom DSL before evaluating compatibility requirements.

The design must answer at least:

1. Does `container(...)` accept a Voiles-specific typed layout API, CSS-like properties, or both?
2. Are styles inline with the node, separate in the `.voil` file, or imported from CSS?
3. How are component-local styles scoped without breaking native cascade?
4. How are custom properties handled?
5. How are pseudo classes/elements handled?
6. How are media queries and container queries handled?
7. How are animations/keyframes handled?
8. How quickly can new browser CSS features be used without waiting for a Voiles release?
9. Which parts can the compiler type-check safely?
10. What is the raw/native CSS escape hatch?

Until this is resolved, examples should avoid treating names such as `display=horizontal` as final language syntax.

---

## 17. Routes

Filesystem routing remains part of project semantics.

```text
app/about.voil -> /about
app/users/[id].voil -> /users/:id
```

Candidate typed param declaration:

```voil
param id: Int
```

A route conversion failure must never inject an unchecked invalid value into the page. Exact failure behavior remains open.

Planned special files include:

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

Interop is required, but JS/npm modules are a type and safety boundary.

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

component IconBtn(icon: String):
	container:
		std.button(icon)

component Card(title: String):
	container:
		std.h2(title)
		slot
```

Page:

```voil
import std from "@voiles/html-base"
import UserBtn, IconBtn, Card from "../ui/components/buttons.voil"

state page_count = 0
shared session = none
const username = "Alice"

header:
	std.h1("Voiles")

main:
	UserBtn(text="First")
	UserBtn(text="Second")
	IconBtn(icon="plus")

	Card(title="Profile"):
		std.p(username)

	std.p(page_count)
```

Each independent `UserBtn()` invocation identity creates its own component instance scope. Reactive parameter changes update an existing instance without resetting its local `state`. `shared` is the explicit application-global form. `Card` places its caller-authored child content at `slot`, while `username` still resolves in the page/caller lexical scope.

---

## 21. Initial grammar sketch

Descriptive only:

```text
module              := moduleItem* EOF

moduleItem          := importDecl
                     | typeDecl
                     | bindingDecl
                     | functionDecl
                     | componentDecl
                     | paramDecl
                     | structuralBlock
                     | expressionStatement

block               := ":" NEWLINE INDENT statement* DEDENT
comment             := "#" commentText NEWLINE

importDecl          := "import" importItem ("," importItem)* "from" stringLiteral
importItem          := identifier importAlias?
importAlias         := "as" identifier

functionDecl        := "fn" identifier parameters returnType? block
componentDecl       := "component" PascalIdentifier componentParameters componentBlock
componentParameters := "(" componentParameterList? ")"
componentParameter  := identifier typeAnnotation ("=" expression)?
componentBlock      := ":" NEWLINE INDENT componentItem* DEDENT

bindingDecl         := constDecl | stateDecl | sharedDecl
constDecl           := "const" identifier typeAnnotation? "=" expression
stateDecl           := "state" identifier typeAnnotation? "=" expression
sharedDecl          := "shared" identifier typeAnnotation? "=" expression

componentItem       := statement | slotOutlet
structuralBlock     := structuralName callArguments? block
componentCall       := PascalIdentifier componentCallArguments childBlock?
componentCallArguments := "(" namedArgumentList? ")"
childBlock          := ":" NEWLINE INDENT childItem* DEDENT
childItem           := namedSlotBlock | uiStatement
namedSlotBlock      := "slot" identifier block
slotOutlet          := "slot" identifier?

standardHtmlCall    := identifier "." identifier callArguments
callArguments       := "(" argumentList? ")"
namedArgument       := identifier "=" expression
```

For component imports, comma-separated `importItem` lists are Accepted. `importAlias` using `as` remains Draft pending explicit acceptance. Non-component symbol-resolution forms remain open even though the surface parser can share the same list structure.

Component calls accept only named arguments. Semantic validation must reject positional arguments, duplicate component arguments, unknown argument names and missing required parameters.

Component parameter bindings are immutable. Defaults are evaluated once for each new component instance, in declaration order, and may reference only parameters initialized earlier in the same component parameter list.

Semantic validation must also:

- reject `sharedDecl` outside module top level;
- require component declaration names to be unique within a module;
- create an independent scope for each new component invocation identity;
- preserve an existing component instance when only its reactive parameter values change;
- expose top-level component declarations automatically to component import resolution;
- resolve each component import item by component declaration name before applying any accepted alias;
- lower default and named slot content without rebinding its lexical environment from caller to callee;
- reject more than one default slot outlet or duplicate named slot outlets in one component;
- reject duplicate named-slot provisions in one invocation;
- reject named-slot provisions that have no corresponding declared outlet;
- reject ordinary/default child content when the callee has no default slot outlet.

The grammar must still resolve:

- multiline call indentation;
- structural block name table;
- expression precedence;
- CSS/style contexts;
- component alias finalization;
- component identity/key semantics for conditional/repeated UI;
- slot parameter/type details;
- non-component import/export semantics.

---

## 22. Compiler/CST requirements

The accepted syntax requires:

- explicit `NEWLINE`, `INDENT`, `DEDENT` behavior or an equivalent parser model;
- `:` block opener recognition;
- line comments beginning with `#`;
- lossless comment/trivia preservation;
- source spans suitable for diagnostics;
- multiline parenthesized expression continuation;
- recovery after malformed indentation;
- comma-separated import item parsing;
- distinction between structural block, component declaration, standard HTML call, ordinary call and user component call;
- component named-only argument validation;
- component parameter default dependency/order validation;
- parsing of component child blocks, named `slot Name:` blocks and component `slot` outlets;
- preservation of caller lexical scope through slot lowering;
- slot cardinality and unknown-slot validation;
- preservation of component-local state across reactive parameter updates;
- formatter stability;
- component symbol indexing per module;
- component dependency graph construction;
- dead-code elimination eligibility for unreachable component declarations.

---

## 23. Next decisions required

Parser-blocking or near-blocking priorities:

1. define component identity/key semantics for conditional and repeated/list UI;
2. finalize multiline indentation/tokenization rules;
3. finalize the remaining `const/state/shared` lowering details and whether a local `mut` form is needed;
4. define top-level structural block grammar and valid structural names;
5. finalize component import alias syntax (`as` currently recommended);
6. finalize general named-argument `=` grammar for ordinary functions/struct construction;
7. define expression precedence;
8. define `@voiles/html-base` minimum API surface;
9. separately design CSS/layout integration before locking `container(...)` parameters;
10. define module top-level side-effect policy so tree-shaking guarantees are precise;
11. define non-component default/named/namespace import semantics;
12. define slot parameter/type semantics only if a concrete use case requires them.
