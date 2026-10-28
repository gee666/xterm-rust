# Shared WASM support extraction

Athena proposed an internal `xterm-wasm-support` crate to separate generic JavaScript conversion and error handling from the unfinished DOM service graph. Xavier approved the extraction with these constraints.

- Move the implementation once. Keep old `xterm-web` Rust module paths as reexports where callers need compatibility. Do not copy types such as JsCallbackError into both crates, since downcasts must retain identity.
- The support crate may depend on xterm-core and the already locked wasm-bindgen/js-sys packages. Keep DOM-specific setters, event geometry and terminal assembly in xterm-web. Target-gate WASM dependencies; the portable core must not import browser APIs.
- Coordinate the root workspace member and Cargo.lock with Chloe. Keep edition, MSRV, license and lint policy consistent. Do not create another target directory or change build profiles.
- Preserve actual Terminal integration assertions in xterm-web. Do not replace a test that constructs Terminal with a fake options-only host and count it as the same integration proof. Direct support tests are additional component evidence.
- The full workspace, all relevant features and the real browser demo must still pass before claiming equivalence. A green support-crate test does not waive viewport, renderer, addon, lifecycle or GC work.
- Update source-to-Rust paths, evidence commands, package names and hashes when files move. Preserve source notices and the test assertions.

One useful shared binding belongs here: source disposal tokens are ordinary JavaScript objects with a callable dispose property, not necessarily this-bound exported WASM class methods. Extracted cleanup functions must retain the source behavior. Add tests for calling `const dispose = token.dispose; dispose()` and for return-object/function identity, listener additions/removals, reentrant disposal and thrown values. Do not introduce Closure::forget or a last-payload cache.

Rust owns terminal algorithms and state. Small JavaScript helpers are restricted to ABI operations such as object-literal construction, raw property/primitive behavior, host scheduling and ownership that JavaScript GC must trace. This extraction is not delegation of the terminal engine back to xterm.js.
