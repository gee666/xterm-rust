# Browser addon ownership review

This is a static ownership finding, not an executed GC test. It matters to image renderer task 127 because source-compatible method wrappers need JavaScript receiver identity without adding Rust/JavaScript cycles.

## Current path to inspect

`crates/xterm-web/src/terminal/mod.rs` stores `BrowserAddonManager` in `Shared`. `addon_manager.rs` stores a strong `js_sys::Array` of entries in that Rust manager. Each entry retains its addon instance. `activate_addon(instance, terminal)` passes the actual public JavaScript terminal object to user code.

A normal addon can retain that argument:

```js
activate(terminal) { this._terminal = terminal; }
```

The resulting ownership path appears to be:

1. The exported terminal's Rust `Shared` owns its manager.
2. The manager's Rust-held `JsValue` strongly roots the entries array.
3. An entry roots the JavaScript addon instance.
4. The addon roots the exported JavaScript terminal object.
5. That object retains the Rust terminal allocation and `Shared`.

Keeping closure captures entirely in JavaScript, as `addon_bridge.js` does, solves one class of cycles. It does not by itself remove the Rust-owned entries-array root in this path. An unopened terminal with no DOM listeners is the smallest useful test case. Explicit successful disposal removes entries and can break the path, but forgotten disposal or an exception during disposal still needs review against the original JS manager's GC behavior.

## Proposed ownership boundary

Keep the manager entry record in JavaScript-owned state attached to the public instance or a JavaScript weak-keyed table. A Rust manager can hold a weak reference to that record. Its Rust route closure remains stateless, receiving the record as an argument. Preserve existing activation/disposal order, original method receivers, entry identity, partial failures and constructor-time setup. Do not use a dummy terminal receiver or silently clear state after a thrown disposer.

Image renderer records can follow the same pattern. The native `StorageRenderer` must not strongly root a JavaScript record containing the public terminal. Async work may temporarily retain the record while a source promise is pending; that is intentional lifetime retention, not a permanent ownership cycle. The image source explicitly permits pending work to finish after addon disposal.

## Required verification

- Independently inspect whether wasm-bindgen's current `JsValue` ownership makes the path a strong hidden root.
- With actual Rust/WASM bindings, construct an unopened terminal, load an addon retaining the terminal, remove all external strong references, and use a bounded explicit GC facility where supported. Compare with the original JS terminal. Record unsupported collection controls rather than inventing a pass.
- Include ordinary disposal, a thrown disposer, an externally retained addon and no retained addon. Do not weaken lifecycle assertions to obtain collection.
- Confirm the image open/setRenderer wrappers preserve saved-function alias behavior and resource cleanup with no Rust self-cycle.

Root task 31 and addon adapter task 66 own any manager implementation change. Task 127 should not add another hidden root while that review is pending.
