---
name: webgl-context-check
description: Use when changing the actual WebGL canvas/context construction, physical observer sizing, global restoration timer, window observer replacement or context event lifecycle.
---

Read docs/webgl-context.md for the component boundary and remaining306 renderer/addon integration. Reuse the existing browser-host-check.mjs runner, not a new browser server. Use owned O: target/TMP/TEMP/TMPDIR, Cargo -j2 and the coordinator's compiler/browser grants.

Generate the unchanged WebglRenderer source prelude with SOURCE_ROOT=/unchanged/reference node scripts/webgl-context-oracle.mjs. The separate source test wrapper bypasses GPU/model initialization only; it does not replace the original event closures, observer, timer, resize or disposal methods. All bundled originals and the wrapper have separate hashes. No source assertions are edited.

Run WASM_TEST=wasm_webgl_context WASM_PRELUDE=tmp/webgl-context/source.js node scripts/browser-host-check.mjs with the locked wasm-bindgen0.2.100 and existing Playwright installation. Verify all29 exact source/Rust trace pairs, thrown-value identity and partial canvas mutation. Controlled phase cases reuse one real WebGL2 return object to avoid native context quotas; dedicated native tests do not substitute getContext and assert per-canvas identity. The actual GPU tests assert RGBA pixels, preserveDrawingBuffer and native WEBGL_lose_context loss/restoration events. Unsupported native loss is a failure with an explicit unrun message, not a passing skip.

Run focused native webgl46, all-feature WASM test compilation and warning-denied native/WASM lib/test Clippy. Run scripts/webgl-context-negative.mjs for the actual production sync-flag3P1F control, followed by the restored full matrix. Retain failed inputs, raw reports and exact restoration hashes. The first-runtime burst/GC inputs have unresolved initial Chromium page errors; do not call that issue fixed from the controlled suite. Run scripts/webgl-context-audit.mjs --runtime before cleanup and its default check afterward. --record creates a producer snapshot, not independent review. After all processes stop, clean only owned target/tmp/generated browser outputs, preserving source, tracked evidence and reference inputs. This is not a working renderer/Addon/Root or full original-suite equivalence claim.
