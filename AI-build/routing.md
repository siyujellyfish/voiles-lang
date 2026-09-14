# Voiles Routing Implementation Contract

Status: `Implementation contract; router not started`.

## 1. File route grammar

```text
index.voil          -> segment root
about.voil          -> static segment
[id].voil           -> one dynamic segment
[...slug].voil      -> one-or-more catch-all
[[...slug]].voil    -> optional catch-all
```

Conflicting route patterns are compile errors.

Reserved route files:

```text
_layout.voil
_404.voil
_error.voil
```

## 2. Typed path parameters

```voil
param id: Int
```

Path conversion happens before page code receives the value. A conversion failure means the typed route does not match and enters nearest `_404` fallback resolution.

Catch-all default type is `List<String>`.

## 3. Layout/error lifetime

Nested `_layout.voil` identity is retained while navigation remains inside the same route subtree and is destroyed when leaving that subtree using normal component/module cleanup semantics.

`_error.voil` resolves to the nearest ancestor error boundary. `_404.voil` resolves to the nearest segment fallback and then root fallback.

## 4. Generated route schema

The compiler should generate a route schema before runtime codegen. A typed link API must validate required parameters against that schema so required path parameters cannot be omitted silently.

## 5. Implementation order

Routing implementation begins after parser/HIR are available:

1. filesystem route scanner;
2. route conflict detector;
3. typed `param` HIR binding;
4. generated route schema;
5. layout/error/404 ownership tree;
6. browser navigation runtime;
7. typed link API.
