# Voiles Type System Implementation Contract

Status: `Implementation contract; semantic HIR exists, checker not started`.

## 1. v0.1 core types

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
fn(...) -> T
user struct
user enum
nominal safety types such as TrustedHtml / Url
foreign opaque JsValue
```

`T?` is syntax sugar for `Option<T>` and `none` is the empty option literal.

## 2. Binding rules

```text
const   immutable lexical binding
state   ordinary mutable lexical/instance binding
shared  module-top-level application-global mutable cell
```

There is no `let`, `var`, or `mut` in v0.1. Reactivity is not a separate type: the compiler attaches reactive tracking only when an observer exists.

Captured `state` is a cell capture. Owner-bound captures may not escape beyond component/module lifetime.

Milestone 1A now provides resolved lexical symbols, references, scope identities, mutability classes and closure-capture discovery. Type checking should consume those identities rather than re-resolve names from CST.

## 3. Struct baseline

Struct values are immutable in v0.1. Construction is named-only; fields may have defaults. Missing required, duplicate, and unknown fields are compile errors.

## 4. Enum / pattern baseline

Enums may carry payloads. `match` over known enum types is exhaustiveness checked and unreachable patterns are diagnosed. `_` is wildcard. Pattern guards are deferred.

## 5. Component/callback baseline

Component parameters are immutable and component calls are named-only. Callback parameters use function types, including concrete DOM event types supplied by html-base metadata.

Repeated component key expressions must type-check as `String` or `Int`.

## 6. Error/async baseline

`async fn`, `await`, `Result<T,E>`, and `?` propagation are v0.1 surface. Native `throw` is not core control flow. Foreign exceptions/rejections must be adapted into explicit error values.

## 7. Milestone 1A handoff

Available from `voiles-hir` before the checker starts:

- `ScopeId` / `SymbolId`;
- declaration `SymbolKind` including binding mutability category;
- resolved lexical references;
- use-before/duplicate/unresolved diagnostics;
- component/function parameter declaration order;
- closure capture relationships;
- lexical lifecycle scopes.

Not yet available and required before broad type checking:

- cross-module exported/imported symbol identity;
- resolved type-reference identity;
- generic type parameter environments;
- struct field/enum case semantic tables;
- component/html metadata symbol identity.

## 8. Milestone 1B implementation order

Type checker work begins after module/type identity is attached to resolved HIR. Recommended order:

1. cross-module export/import tables and runtime/type dependency graph;
2. primitive types + literals;
3. explicit type-reference resolution;
4. lexical binding types and assignment checks;
5. function signatures/calls;
6. Option/Result generics;
7. struct/enum declarations;
8. pattern exhaustiveness;
9. component parameter/callback validation;
10. nominal safety types and foreign binding signatures.

The checker must not duplicate lexical name resolution already owned by `voiles-hir`.
