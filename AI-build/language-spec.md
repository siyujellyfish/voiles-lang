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

Core properties:

1. `:` opens a block.
2. indentation defines hierarchy.
3. `#` is comment syntax only.
4. import module paths are always quoted strings.
5. function declarations use `fn`.
6. semantic page structure may appear directly at module level.
7. native HTML leaf APIs are accessed through the standard HTML namespace.
8. user components use function-like PascalCase invocation.
9. no HTML/XML closing tags are used.

---

## 2. Source file model

A `.voil` file is a module.

A module may contain:

- imports;
- type declarations;
- immutable or reactive bindings;
- functions;
- route parameter declarations;
- semantic HTML structural blocks;
- container blocks;
- native HTML API calls;
- user component invocations.

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

main:
	container():
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

## 5. Imports

Status: `Accepted` for module path quoting; import forms remain partially open.

Module specifiers are always strings:

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

The complete default/named/namespace import grammar remains open.

`std` is the current canonical alias used by Voiles examples for `@voiles/html-base` so native HTML APIs remain visually distinct from user components.

---

## 6. Identifiers and naming

Naming style is not currently enforced by the parser.

Examples may use either:

```voil
click_btn
```

or another formatter-defined convention later.

PascalCase remains useful for user component symbols because it visually distinguishes them from native HTML APIs and ordinary functions.

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

## 8. Bindings

Status: `Draft`

The preferred reduced binding model is:

```voil
const title = "Voiles"
state count = 0
```

### 8.1 `const`

Proposed semantics:

- immutable binding;
- may be initialized at runtime;
- does not imply compile-time evaluation;
- does not trigger reactive dependency updates.

Example:

```voil
const title: String = "Voiles"
```

### 8.2 `state`

Proposed semantics:

- mutable binding;
- reactive;
- mutation notifies compiler-generated dependency updates;
- intended primarily for page/component UI state.

Example:

```voil
state count: Int = 0

fn click_btn():
	count += 1
```

### 8.3 Non-reactive mutation

Still open.

Removing `let`/`var` makes the language smaller, but general algorithms can require mutable locals that should not be reactive.

A possible future solution is function-local `mut`:

```voil
fn sum(values: List<Int>) -> Int:
	mut total = 0
	for value in values:
		total += value
	return total
```

This syntax is not accepted yet.

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

A page does not require a `view:` wrapper.

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

Status: `Accepted` invocation form.

```voil
UserBtn(
	text="just click",
	onclick=click_btn
)
```

User components use function-like invocation rather than JSX/XML.

Component children, when supported, use the same block syntax:

```voil
Card(title="Profile"):
	std.p(user.name)
```

The child/slot type model is still open.

### 14.1 Named arguments

Current preferred syntax uses `=`:

```voil
UserBtn(
	text="just click",
	onclick=click_btn
)
```

This remains Draft until assignment, default arguments and struct construction are checked for grammar ambiguity.

---

## 15. Container

Status: `Draft` semantics.

`container` exists to create an explicit block-level layout region.

```voil
main:
	container(...):
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

## 20. Current canonical example

```voil
# Native HTML and user component imports are visually distinct.
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

This is the current syntax reference for parser planning. `container(...)` layout arguments and the exact `const/state` model are intentionally not finalized yet.

---

## 21. Initial grammar sketch

Descriptive only:

```text
module              := moduleItem* EOF

moduleItem          := importDecl
                     | typeDecl
                     | bindingDecl
                     | functionDecl
                     | paramDecl
                     | structuralBlock
                     | expressionStatement

block               := ":" NEWLINE INDENT statement* DEDENT
comment             := "#" commentText NEWLINE

importDecl          := "import" identifier "from" stringLiteral
functionDecl        := "fn" identifier parameters returnType? block

bindingDecl         := constDecl | stateDecl
constDecl           := "const" identifier typeAnnotation? "=" expression
stateDecl           := "state" identifier typeAnnotation? "=" expression

structuralBlock     := structuralName callArguments? block
componentCall       := PascalIdentifier callArguments childBlock?
standardHtmlCall    := identifier "." identifier callArguments
callArguments       := "(" namedArgumentList? ")"
namedArgument       := identifier "=" expression
```

The grammar must still resolve:

- multiline call indentation;
- function-local mutation syntax;
- structural block name table;
- component child blocks;
- expression precedence;
- CSS/style contexts.

---

## 22. Parser/CST requirements

The accepted syntax requires:

- explicit `NEWLINE`, `INDENT`, `DEDENT` behavior or an equivalent parser model;
- `:` block opener recognition;
- line comments beginning with `#`;
- lossless comment/trivia preservation;
- source spans suitable for diagnostics;
- multiline parenthesized expression continuation;
- recovery after malformed indentation;
- distinction between structural block, standard HTML call, ordinary call and user component call;
- formatter stability.

---

## 23. Next decisions required

Parser-blocking or near-blocking priorities:

1. finalize multiline indentation/tokenization rules;
2. finalize `const/state` semantics and whether a local `mut` form is needed;
3. finalize named argument separator `=`;
4. define top-level structural block grammar and valid structural names;
5. define component child block grammar;
6. define expression precedence;
7. define `@voiles/html-base` minimum API surface;
8. separately design CSS/layout integration before locking `container(...)` parameters.
