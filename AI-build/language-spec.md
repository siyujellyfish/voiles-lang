# Voiles Language Specification

Status: `Draft`

Target: initial syntax planning before parser/compiler implementation.

Voiles source files use the `.voil` extension. The language targets front-end applications that compile to native HTML, CSS and JavaScript modules while keeping routing, reactivity, style validation and safety semantics available to the compiler.

This document intentionally separates accepted language principles from syntax that still requires prototype validation.

---

## 1. Language goals

The syntax must optimize for the following properties:

1. Minimal markup noise: no closing tags and no JSX/XML boilerplate.
2. Flat source structure: layout should not require artificial wrapper nodes.
3. Compiler-readable UI: View and Style must be semantic structures, not opaque strings.
4. Strict safety by default: no implicit `any`, `null` or `undefined` semantics.
5. Explicit reactivity: reactive mutation is visible in source.
6. Native Web output: ordinary HTML/CSS/ESM remain the primary browser target.
7. File-based routing: route path is derived from the project filesystem.
8. Low configuration burden: ordinary syntax must not depend on project configuration files.

---

## 2. Source file model

A `.voil` file is a module.

A file may contain:

- imports;
- type declarations;
- constants and functions;
- component properties;
- reactive state;
- route parameter declarations;
- one primary `view` block;
- zero or more `style` blocks.

The source filename and filesystem location may add semantic meaning. In particular, a file under `app/` may become a route automatically.

### 2.1 Initial project convention

```text
app/
	index.voil
	about.voil
	users/
		[id].voil
components/
	UserCard.voil
```

Expected routes:

```text
app/index.voil       -> /
app/about.voil       -> /about
app/users/[id].voil  -> /users/:id
```

The route path is not repeated inside each page source file.

---

## 3. Block structure

Status: `Draft`

The preferred syntax uses significant indentation instead of `{}` for ordinary blocks.

```voil
fn greet(name: String) -> String
	return "Hello {name}"

view
	main.page
		h1 "Hello"
		p "Welcome to Voiles"
```

The formatter will eventually define one canonical indentation style. The parser prototype must verify that multiline expressions, comments and nested View nodes remain unambiguous.

Whether a block opener requires `:` remains open.

---

## 4. Comments

Proposed initial syntax:

```voil
// line comment
```

Multiline comments are not required for the first parser prototype. If added later, they must preserve lossless CST behavior.

---

## 5. Identifiers and naming

Proposed conventions:

- values/functions: `camelCase`
- types/components: `PascalCase`
- constants may remain `camelCase`; uppercase constants are not required by grammar
- CSS class names retain normal CSS-compatible spelling in View/Style contexts

Naming convention should primarily be formatter/linter guidance rather than parser restrictions.

---

## 6. Primitive values and types

Initial core types:

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

Potential optional shorthand:

```voil
String?
```

is equivalent to:

```voil
Option<String>
```

`T?` does not mean a nullable reference. Voiles does not expose JavaScript-style `undefined` as a normal language value.

### 6.1 Literals

```voil
true
false
42
3.14
"hello"
```

String interpolation:

```voil
"Hello {user.name}"
```

Interpolation converts supported values through explicit language-defined formatting rules. Arbitrary object-to-string coercion is not allowed implicitly.

---

## 7. Variable declarations

The initial model separates immutable local values, mutable local values and reactive state.

### 7.1 Immutable value

```voil
let title: String = "Voiles"
```

Type inference should be permitted when unambiguous:

```voil
let title = "Voiles"
```

### 7.2 Mutable local value

Candidate syntax:

```voil
var index: Int = 0
```

`var` is non-reactive and should not by itself cause View updates.

### 7.3 Reactive state

```voil
state count: Int = 0
```

Mutation:

```voil
count += 1
```

The compiler tracks View dependencies on `state` values and lowers them into direct reactive updates.

A normal `let`/`var` does not silently become reactive because it is referenced by View code.

### 7.4 Compile-time constant

Candidate syntax:

```voil
const maxItems: Int = 20
```

Exact compile-time evaluability rules remain deferred until semantic analysis design.

---

## 8. Functions

Initial syntax:

```voil
fn add(a: Int, b: Int) -> Int
	return a + b
```

Functions must have known parameter types. Return type inference may be allowed for local/private functions, but exported API rules should prefer explicit types.

Potential expression-body sugar is deferred:

```voil
fn add(a: Int, b: Int) -> Int = a + b
```

### 8.1 Async

Candidate:

```voil
async fn loadUser(id: Int) -> Result<User, LoadError>
	...
```

Async/error propagation syntax is still `Open` and must be designed together with JavaScript exception interop.

---

## 9. Structs and enums

### 9.1 Struct

Proposed syntax:

```voil
struct User
	id: Int
	name: String
	email: String?
```

Construction syntax is still open. Candidate:

```voil
let user = User(
	id: 1,
	name: "Ada",
	email: none
)
```

### 9.2 Enum

Proposed syntax:

```voil
enum LoadState<T>
	idle
	loading
	ready(T)
	failed(Error)
```

Pattern matching must be exhaustiveness-checked where the enum value is closed and known.

---

## 10. Control flow

The same language control flow is used in ordinary logic and in View blocks.

### 10.1 if

```voil
if user.isAdmin
	showAdminPanel()
else
	showUserPanel()
```

Inside View:

```voil
view
	main
		if user.isAdmin
			AdminPanel
		else
			UserPanel
```

### 10.2 for

```voil
for item in items
	process(item)
```

Inside View:

```voil
view
	ul
		for item in items
			li item.name
```

Keyed iteration is required for stable reactive DOM lists. Candidate syntax:

```voil
for item in items key item.id
	UserRow(item: item)
```

The exact key grammar remains `Draft`.

### 10.3 match

```voil
match state
	idle
		text "Idle"
	loading
		Spinner
	ready(data)
		Results(data: data)
	failed(error)
		ErrorView(error: error)
```

The compiler should report non-exhaustive matches when it can prove the input enum is closed.

---

## 11. View

A file may define one primary View block:

```voil
view
	main.page
		h1 "Hello"
		p "Welcome"
```

View syntax is not HTML text. The parser builds a typed View AST/HIR.

### 11.1 Native element node

Preferred node shape:

```text
element[.class...][#id][(attributes)] [value]
```

Examples:

```voil
h1 "Account"
p.description user.bio
button.primary(type: "button") "Save"
img.avatar(src: user.avatar, alt: user.name)
```

Attribute separator is still open between `:` and `=`.

### 11.2 Text

Literal text:

```voil
p "Hello"
```

Expression text:

```voil
p user.name
```

Interpolated text:

```voil
p "Welcome, {user.name}"
```

All normal text insertion is escaped.

### 11.3 Classes and IDs

Candidate shorthand:

```voil
main.page
section.content.wide
button#submit.primary
```

Dynamic class syntax remains open and should not become string concatenation by default.

Possible future model:

```voil
button(class: { primary: true, loading: busy }) "Save"
```

No final syntax is selected yet.

### 11.4 Attributes

Candidate:

```voil
a(href: user.profileUrl, target: "_blank") user.name
```

Attribute values are typed expressions. The compiler should understand special Web types such as URL-like or boolean attributes where practical.

### 11.5 Boolean attributes

Candidate:

```voil
button(disabled: busy) "Save"
```

The compiler emits or removes the native boolean attribute according to the expression value rather than serializing `"false"`.

---

## 12. Layout primitives

Voiles should provide layout-semantic nodes that do not necessarily create DOM wrappers.

Initial candidates:

```text
stack
row
grid
layer
```

Example:

```voil
view
	stack.page
		h1 "Dashboard"
		row.actions
			button "Refresh"
			button "Settings"
```

`stack` and `row` describe layout intent. During lowering, the compiler may:

1. reuse an existing semantic element;
2. create a container if CSS/layout semantics require one;
3. eliminate a redundant container where output semantics are unchanged.

The exact wrapper-elision algorithm is compiler work and not part of the surface grammar.

---

## 13. Components

Status: `Draft`

The preferred model treats one `.voil` component file as an implicit default component.

Example `components/UserCard.voil`:

```voil
prop user: User
prop compact: Bool = false

view
	article.card
		h2 user.name
		if !compact
			p user.email
```

A consumer imports the file and invokes it as a component.

Candidate invocation:

```voil
UserCard(user: currentUser, compact: true)
```

This intentionally resembles typed function invocation rather than XML tags.

### 13.1 Props

Required prop:

```voil
prop user: User
```

Optional/default prop:

```voil
prop compact: Bool = false
```

Props are immutable inside the component unless a future explicit mechanism says otherwise.

### 13.2 Children

Typed child content / slots are still open. v0 parser may initially support components without named slots and add children semantics after the basic component HIR stabilizes.

---

## 14. Events

Status: `Open`

Current preferred direction:

```voil
button "Save"
	on click save
```

Handler:

```voil
fn save(event: ClickEvent)
	...
```

Potential modifiers:

```voil
button "Save"
	on click prevent save
```

Modifier vocabulary and event type model are not yet defined.

Alternative syntax remains under evaluation because event blocks can add nesting for otherwise leaf nodes.

---

## 15. Binding

Status: `Open`

Explicit one-way form is always representable:

```voil
input(value: name)
	on input updateName
```

A dedicated two-way binding feature may be added only if its mutation and type semantics remain obvious.

Candidate:

```voil
input
	bind value: name
```

No two-way binding syntax is accepted yet.

---

## 16. Styles

Status: `Draft`

Style declarations are parsed by Voiles and lowered into typed Style IR.

Preferred baseline keeps CSS property names recognizable while removing braces and semicolons:

```voil
style .card
	display: grid
	gap: 1rem
	padding: 1rem
	border-radius: 0.75rem
```

The compiler must validate at least:

- known property names;
- value grammar where supported;
- unit compatibility where statically knowable;
- invalid combinations where the language intentionally adds stronger constraints.

### 16.1 Style scope

Default scoping is still open. The preferred direction is component-local styles with an explicit global escape hatch.

### 16.2 Pseudo selectors

Candidate:

```voil
style .button:hover
	transform: translateY(-1px)
```

### 16.3 Responsive/container conditions

Not yet specified. This must preserve compatibility with modern CSS capabilities rather than replace them with a permanently smaller custom feature set.

### 16.4 Native CSS escape hatch

Required before v1, syntax open. Escape-hatch regions must be clearly marked because the compiler may not be able to provide the same typed guarantees or optimization.

---

## 17. Routes

Filesystem routing is part of project semantics rather than explicit route path syntax.

### 17.1 Static route

```text
app/about.voil -> /about
```

No source declaration is necessary.

### 17.2 Dynamic route

```text
app/users/[id].voil -> /users/:id
```

By default, `id` may enter the file as `String`.

A page can request a stronger route type:

```voil
param id: Int
```

This produces a typed `id: Int` binding after route parsing.

Invalid conversion must not produce an unchecked value. The router/compiler contract will define whether conversion failure means non-match, 404 or an explicit route error.

### 17.3 Route-special files

Planned project-level conventions include:

```text
_layout.voil
_404.voil
_error.voil
```

Their exact exported bindings and lifecycle are not part of the first syntax prototype.

---

## 18. Page metadata/head

Status: `Open`

A future route file may support a compiler-aware head block rather than direct DOM mutation.

Candidate:

```voil
head
	title "Profile - {user.name}"
	meta(name: "description", content: user.bio)
```

This is not required for the first parser milestone.

---

## 19. HTML safety

Normal View expressions produce text, not HTML.

```voil
p userInput
```

must escape HTML-sensitive content.

Raw HTML requires a distinct trusted type or an explicit unsafe boundary.

Preferred safe API concept:

```voil
prop content: TrustedHtml

view
	html content
```

An unsafe conversion from arbitrary String must be visibly exceptional and may be unavailable without an explicit unsafe/interop API.

A plain `String` must never implicitly satisfy `TrustedHtml`.

---

## 20. URL/style safety types

Long-term safety model should distinguish data domains that JavaScript commonly represents as raw strings.

Candidates include:

```text
Url
TrustedHtml
CssValue / typed style values
```

These are semantic safety types, not necessarily all primitive syntax-level types.

The first type checker prototype may begin with `String` while reserving the right to introduce stronger standard-library nominal types before stable release.

---

## 21. Modules and imports

Status: `Open`

Voiles needs to distinguish or safely unify three sources:

1. `.voil` modules;
2. built-in/platform modules;
3. JavaScript/npm modules.

Candidate surface syntax:

```voil
use UserCard from "../components/UserCard.voil"
```

A more language-native module path syntax may replace this. JS interop cannot simply inherit TypeScript trust; external values should cross an explicit validation/type boundary.

No import syntax is accepted yet.

---

## 22. JavaScript interop

Status: `Open`

Interop is required for ecosystem compatibility, but JS must be treated as a safety boundary.

Rules to preserve:

- imported JS values do not implicitly gain trustworthy Voiles types;
- `undefined`, thrown exceptions and mutable object behavior must be normalized explicitly;
- DOM/Web API wrappers may be provided by the standard library with known types;
- unsafe direct interop, if supported, must be visually explicit.

Exact syntax is deferred until the type system and module system prototypes exist.

---

## 23. Error handling

Status: `Open`

The type system should include `Result<T, E>`.

Candidate pattern:

```voil
match loadUser(id)
	ok(user)
		show(user)
	err(error)
		report(error)
```

Propagation syntax such as postfix `?` is not accepted yet. It must be evaluated together with async functions and JS exception boundaries.

---

## 24. Complete component example

This example demonstrates the preferred current direction, not final syntax.

```voil
struct User
	id: Int
	name: String
	email: String

prop user: User
prop compact: Bool = false
state expanded: Bool = false

fn toggle()
	expanded = !expanded

view
	article.card
		row.header
			h2 user.name
			button(type: "button") "Details"
				on click toggle

		if !compact && expanded
			p user.email

style .card
	display: grid
	gap: 1rem
	padding: 1rem
	border-radius: 0.75rem

style .header
	display: flex
	align-items: center
	justify-content: space-between
```

---

## 25. Complete route example

Conceptual file: `app/users/[id].voil`

```voil
param id: Int
state user: User? = none
state loading: Bool = true

async fn load()
	...

view
	main.page
		if loading
			p "Loading..."
		else
			match user
				none
					p "User not found"
				some(value)
					UserCard(user: value)

style .page
	max-width: 72rem
	margin-inline: auto
	padding: 1rem
```

The lifecycle that invokes `load()` is intentionally unspecified until component/page lifecycle semantics are designed.

---

## 26. Initial grammar sketch

This grammar is descriptive only and is not ready to become the parser source of truth.

```text
module          := declaration* EOF

declaration     := importDecl
                 | structDecl
                 | enumDecl
                 | valueDecl
                 | functionDecl
                 | propDecl
                 | paramDecl
                 | stateDecl
                 | viewDecl
                 | styleDecl

viewDecl         := "view" block<viewStatement>

viewStatement    := viewNode
                 | ifStatement
                 | forStatement
                 | matchStatement

viewNode         := nodeHead block<viewStatement>?
nodeHead         := identifier classRef* idRef? attributes? expression?
classRef         := "." identifier
idRef            := "#" identifier
attributes       := "(" namedArgument ("," namedArgument)* ")"

styleDecl        := "style" selector block<styleProperty>
styleProperty    := propertyName ":" styleValue

stateDecl        := "state" identifier typeAnnotation? "=" expression
propDecl         := "prop" identifier ":" type ("=" expression)?
paramDecl        := "param" identifier ":" type
functionDecl     := asyncModifier? "fn" identifier parameters returnType? block<statement>
```

A real grammar must be produced only after indentation/tokenization and expression precedence prototypes are tested.

---

## 27. Parser/CST requirements derived from syntax

The syntax design implies the parser must eventually support:

- significant indentation tokens if D-001 is accepted;
- lossless comments/trivia preservation;
- source spans on every syntax node relevant to diagnostics;
- recovery after malformed View nodes and declarations;
- context-sensitive distinction between View nodes, component invocation and ordinary expressions;
- CSS-like selector/property tokenization inside Style without turning Style into a raw string;
- future formatter stability.

These requirements should influence parser architecture before implementation begins.

---

## 28. MVP syntax subset for first parser prototype

The first parser prototype should intentionally support less than this entire document.

Target subset:

```text
comments
primitive literals
identifiers
let / var / state
basic expressions
fn
if / else
for
struct
enum
prop
param
view
native View nodes
component invocation
style blocks with simple property:value pairs
```

Explicitly excluded from parser milestone 1:

```text
async lowering
JS interop
raw HTML
slots
advanced CSS at-rules
route lifecycle
SSR syntax
macros
operator overloading
```

---

## 29. Next decisions required

Before implementing the lexer/parser, the following must be decided or prototyped in order:

1. whether indentation is syntax-significant;
2. whether block openers require `:`;
3. attribute/named-argument separator (`:` vs `=`);
4. View node vs component invocation disambiguation;
5. event syntax;
6. optional `T?` syntax;
7. Style scoping baseline;
8. import/module syntax.

Items 1-4 are parser-blocking. Items 5-8 can remain draft during the earliest lexer experiments.
