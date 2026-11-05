# Live image storage inspection

Task390 adds a read-only `_storage._images` inspection binding to the browser ImageAddon. The binding reads the real Rust store. It does not mirror images in JavaScript or delegate decoding, tile placement, eviction or rendering to the reference engine.

The storage view and its image collection keep stable identity for one activation. A later activation creates another view; a retained earlier view still reads its original store. The collection supports `size`, `has`, `get`, `keys`, `values`, `entries`, `forEach` and iteration. Keys are not coerced. Traversal follows native insertion ordinals and observes resets and new insertions until exhaustion. No storage borrow crosses a consumer callback.

Image values expose the actual original and resized canvas/bitmap, cell sizes, tile count, buffer type, layer and z-index. A weak JavaScript handle cache preserves image-value identity while a caller retains a value, without retaining removed image resources. This is an inspection API, not a complete writable source-private Map or IImageSpec replacement. Private Map mutation, marker fields, arbitrary private spec mutation, `_opts`, `_handlers` and the remaining `_core` bindings are still unfinished under311/324. No missing mutator returns successful no-op behavior.

## Observed checks

Windows, Rust/Cargo1.85.1, Node24.15.0, wasm-bindgen0.2.100:

- The complete directed coordinator matrix now passes16 tests per engine in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. Four new tests compare real source/Rust storage keys, identity, original pixels, layers, reset/disposal, retained activations, live iteration, callback error identity and reset from a forEach callback.
- Focused native storage tests pass21 tests, including two new live-traversal tests. Full native all-target/all-feature workspace execution passes2064 tests across412 summaries, with zero failures or ignored tests. This does not execute WASM assertions.
- Exact `cargo fmt --all -- --check`, native workspace warning-denied Clippy and scoped WASM warning-denied Clippy pass.
- An initial new native test placed successive images on the same tile, so ordinary overwrite eviction correctly removed earlier entries. The test now places images on different cells to exercise traversal. No production overwrite behavior or source assertion changed.

## Unchanged original suite outcomes

The first bounded600-second full addon-image run discovered639 case-projects. It executed428, passed217, failed211, skipped4 and left207 unrun. All observed failures stopped at missing `_core._renderService`, `_opts`, `_storage` or `_handlers` inspection. A timeout is not a source skip or a passing original matrix.

With the storage binding, the complete unchanged `ImageAddon.test.ts` family accounts for108 case-projects in all three engines. It executes105, passes92, fails13, skips3 and leaves0 unrun. Nine failures are the still-missing core/options inspection bindings. Four newly reached image-size assertions report165 pixels where the test expects160. The retained original JavaScript baseline has the exact same four failing titles, engines and values. The original handler uses fractional cell width while the assertion helper rounds it. Do not round Rust's geometry or weaken the source assertions to hide these failures. The historical JavaScript baseline also has four WebKit/SIXEL initialization failures, recorded separately from today's Rust results.

Raw initial and scoped Playwright reports, discovery, source-baseline failure identities and quality logs are under `results/image-storage-inspection`. Those reports contain old temporary absolute paths; source and outcome hashes remain durable evidence after generated packages are removed.

## Reproduce

Use the reference/tool prerequisites from docs/image-addon.md. Keep the supplied reference source and assertions unchanged.

```sh
cargo fmt --all -- --check
cargo test --locked -j2 -p xterm-addons --test image_storage
cargo test --locked -j2 --workspace --all-targets --all-features
cargo clippy --locked -j2 --workspace --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test image_addon_browser -- -D warnings
WASM_TEST=image_addon_browser CARGO_BUILD_JOBS=2 node scripts/image-canvas-check.mjs
node scripts/e2e-upstream.mjs --suite addon-image --grep ImageAddon.test.ts --timeout-ms 180000
```

The final command is expected to report the unresolved failures, not a green family pass. Complete Kitty suite execution, the remaining original assertions, img01..48 acceptance adapter and final integrated/platform gates remain311/324 obligations. Overall equivalence is NOT PROVEN.
