# Live Kitty handler inspection

Task393 adds read-only `_handlers` diagnostics to the real browser ImageAddon. Protocol execution, image bytes, pending transmissions, placement IDs and map ordering stay in Rust. This binding does not delegate to the source terminal or maintain a JavaScript protocol-state copy.

The handler view exists before activation and keeps its identity for the addon's lifetime. It exposes `size`, `has`, `get`, `keys`, `values`, `entries`, `forEach` and iteration. Keys are not coerced. Entries appear in registration order. Repeated activation replaces a handler value without changing its position; disposal clears the same view. Monotonic native insertion ordinals let retained iterators observe appends after clear until exhaustion. Handler replacement and clearing release old owners outside the collection borrow.

Retained handler values own their original protocol. Each exposes `reset`, which invokes that native protocol's actual reset path. Kitty values also expose:

- `images`, a live read-only map of transmitted image records with `id`, `data`, `width`, `height`, `format` and `compression`. `data` is the actual immutable Blob created during transmission, not reconstructed bytes or a decoded canvas.
- `_kittyIdToStorageId`, a live read-only map of the native placement IDs.
- `pendingTransmissions`, membership-only diagnostics with `size`, `has` and `keys`. This deliberately does not invent detached command or decoder values for the source's mutable pending records.

JavaScript caches only weak binding handles. Native allocation identity distinguishes an image or protocol replacement from an older retained record under the same key. A live handle owns its Rust record, so the allocation cannot be reused while that handle remains reachable. Evicted record wrappers and their Blob objects are not retained by the cache. Membership and iteration release native borrows before consumer callbacks. Callback exceptions preserve their thrown value.

These objects are diagnostic adapters, not drop-in writable JavaScript Maps. Arbitrary Map mutation, pending command/decoder object access, patched private methods, complete SIXEL/IIP private fields, raw `_opts` and the terminal's remaining private `_core` services are not implemented by this task. No missing mutator returns a successful no-op. Those contracts and img01..48 remain311/324 work. No dependency or unsafe code was added.

## Observed checks

Windows, Rust/Cargo1.85.1, Node24.15.0, wasm-bindgen0.2.100:

- The directed coordinator matrix passes21 tests per engine in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. Five new tests compare unchanged source and Rust for activation identity/order, replacement records and actual Blob bytes, placement IDs, chunked pending membership, delete/reset, retained iterations after clear/reactivation and reset/error callbacks.
- The unchanged original Kitty family discovers531 case-projects, executes530, passes524, fails6, skips1 and leaves0 unrun. All six failures stop at missing `term._core._renderService.dimensions`, before their image-size assertions.
- The complete unchanged original image family discovers639 case-projects, executes635, passes616, fails19, skips4 and leaves0 unrun. No executed failure stops at `_handlers`. The19 failures comprise9 missing private core/render bindings,6 missing `_opts` records and4 image-size assertions with the same165-versus160 results as the retained JavaScript baseline. Do not round image geometry or change those assertions to hide that baseline discrepancy.
- Native workspace all-target/all-feature tests pass2064 tests across413 summaries, with zero failed or ignored tests. This does not execute WASM assertions. Exact workspace formatting, native workspace warning-denied Clippy and scoped WASM warning-denied Clippy pass.

Raw original reports, directed browser/source/input identities, unchanged reference hashes and gate logs are under `results/image-handler-inspection`. They retain historical temporary paths for provenance. Generated packages are disposable after review; their hashes remain in the reports.

After rebasing onto integrated maincbe72b5, all gates and the complete639 original matrix were rerun. All19 original failure titles, projects and messages exactly match the earlier run. The source under test includes the integrated Base64 and IME changes, not a stale generated package.

The first scoped check found an inference error in a new test's `Into` array. The first exact formatting invocation hit Windows OS206 at the longer owned path. Moving the same worktree to `O:tmp/Q` allowed the exact command to run and found one module-order formatting difference, which was fixed. No original assertion or production behavior changed to clear these checks.

## Reproduce

Use the reference and tool prerequisites from docs/image-canvas.md. Source tests and assertions must remain unchanged.

```sh
cargo fmt --all -- --check
cargo clippy --locked -j2 --workspace --all-targets --all-features -- -D warnings
cargo test --locked -j2 --workspace --all-targets --all-features
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test image_addon_browser -- -D warnings
WASM_TEST=image_addon_browser CARGO_BUILD_JOBS=2 node scripts/image-canvas-check.mjs
node scripts/e2e-upstream.mjs --suite addon-image --timeout-ms 600000
```

The last command currently reports unresolved failures, not a full image-family pass. Browser/native OS equivalence and the full image-addon contract remain NOT PROVEN.
