# Voiles Security Implementation Contract

Status: `Implementation contract; security lowering not started`.

## 1. Trusted HTML boundary

Plain `String` is never trusted HTML. Ordinary text sinks escape HTML-sensitive content.

`TrustedHtml` is nominal. Runtime user content must pass an approved sanitizer API before it can become `TrustedHtml`. Any raw/unsafe conversion must be explicit and visually unsafe.

## 2. URL boundary

Compile-time URL literals used in sensitive sinks should be scheme-validated by the compiler. Dynamic sensitive URL values use nominal safe URL wrappers produced by a safe parser.

Dangerous schemes such as `javascript:` must be rejected by the safe parser/binding surface.

## 3. Foreign JavaScript boundary

Native JS/npm modules require explicit `extern import`. Untyped foreign values default to opaque `JsValue`; v0.1 has no unconstrained `any`.

Typed adapters must normalize:

```text
JS undefined            -> Option / none
JS throw                -> explicit error result
Promise rejection       -> explicit error result
mutable foreign object  -> explicit wrapper semantics
```

`.d.ts` may be used to generate bindings but never bypasses runtime validation/normalization rules.

## 4. Resource ownership

Cleanup-requiring external resources may not be hidden in ordinary `shared` initializers. They must have an explicit scoped module/component owner with `init:/cleanup:` or `mount:/cleanup:`.

## 5. Compiler enforcement points

Security checks eventually belong in typed HIR/lowering:

- HTML text vs TrustedHtml sink typing;
- URL nominal type/scheme checks;
- foreign adapter conversion requirements;
- owner-bound closure escape diagnostics;
- cleanup-requiring resource/effect classification;
- unsafe boundary diagnostics/source spans.
