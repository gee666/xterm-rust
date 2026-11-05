# Raw image storage limits

Task439 repairs the browser ImageAddon storageLimit option and public setter. It does not complete image addon parity.

The source constructor copies options twice but does not convert storageLimit. Before activation, assigning the public property stores the argument without conversion. The public getter still returns -1 because no storage exists. Reset does not reset this option. Once storage exists, including after disposal, setLimit evaluates `value < 0.5`, then `value > 1000` unless the first comparison rejected it, then `value / 4 * 1000000 >>> 0`. An object can therefore run user code three times and return a different value each time. BigInts can pass the comparisons and fail division. A Symbol or an arbitrary thrown value must escape unchanged. Only successful conversion and eviction commit the raw public option. Pixel-budget changes precede eviction callbacks and survive an eviction failure.

The WASM setter now accepts JsValue. The first constructor copy and the public setter retain raw values in one shared Rust allocation. The stable read-only `_opts` diagnostic reads that allocation. No RefCell borrow survives conversion or callbacks. Native ImageStorage remains the image, budget and eviction authority. Its new pixel-budget constructor and setter accept an already converted u32 without applying megabyte validation again. That matters when an object's third conversion returns a value that would have failed the earlier comparisons.

A small JavaScript binding adapter performs only the source's relational, division and unsigned conversion operations. Replacing them with one eager Number conversion changes callback count, exception identity and reentry. The constructor adapter also preserves source's Error-only console.error, unconditional console.warn on conversion failure, 10 MB fallback and exceptions thrown by either console callback. No terminal engine logic runs in this adapter. Other browser options still use their existing typed projection. The typed ImageOptions.storage_limit is not authoritative for the browser addon; native typed consumers keep their existing contract. No dependency or unsafe code was added.

Rust browser callers now pass `addon.set_storage_limit(64.0.into())`. Public JavaScript spelling is unchanged. The numeric storageLimit getter continues to report the actual pixel budget, with the source's falsy-to-minus-one behavior.

## Verification

Windows with Rust/Cargo1.85.1, Node24.15.0, wasm-bindgen0.2.100:

- Native image_storage passes23 tests, including two new tests for the complete u32 pixel-budget range, constructor diagnostics and committed budget/error-chain ordering during failed eviction. The first new test incorrectly expected an unwrapped callback error. Its retained initial failure led to checking the actual nested error source instead. No production change was needed for that test correction.
- The directed image_addon_browser target passes33/33 functions in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. It contains28 retained tests and five new raw-limit tests. This branch does not replay the separate accepted431 placeholder changes.
- The new tests cover21 raw values, nested inactive assignments, first-versus-second constructor copies, deferred activation, conversion failure diagnostics, throwing/reentrant console callbacks, primitive active values,120 changing-value/coercion/reentry variants, retained post-disposal storage and actual QOI image eviction. Object.is checks preserve raw identity and signed zero. A source comparator is bundled unchanged by the existing image harness.
- Exact old-code control restored seven production files to main dcde3e9c while retaining all five new browser tests. Chromium passed28 and failed exactly the five new tests. All seven files were restored byte-for-byte in finally, followed by another33/33 pass in all three engines. Baseline, fixed and restored hashes are retained in negative-audit.json. The control script is retained as negative-control.mjs.
- Focused native and WASM Clippy pass with warnings denied. Each of the five workspace packages passes formatting separately. Exact `cargo fmt --all -- --check` fails Windows OS206 and is retained as a failure, not a pass.

Commands use the environment in docs/image-canvas.md and an owned O: checkout. Temporary compiler, browser and server work is bounded and sequential under the coordinator's grant.

```sh
cargo test --locked -j2 -p xterm-addons --test image_storage
cargo clippy --locked -j2 -p xterm-addons --lib --test image_storage -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test image_addon_browser -- -D warnings
WASM_TEST=image_addon_browser CARGO_BUILD_JOBS=2 node scripts/image-canvas-check.mjs
node results/image-storage-limit-values/negative-control.mjs
cargo fmt --all -- --check
```

Retained reports and logs are under results/image-storage-limit-values. Historical temporary paths identify the actual build, not permanent installations. The identity audit checks the final Rust input pins, unchanged comparator source files against the supplied P: reference and all seven control files against their Git baseline and saved fixed hashes.

## Independent review

Task439 was independently accepted through522deaf2. The reviewer reran23 native tests,33 browser test functions in each engine, focused native and WASM warning-denied Clippy and formatting for both changed packages. The reviewer also checked101 Rust input pins,121 unchanged comparator files and seven control baselines/restored files. No code changes were requested. The retained negative control was inspected rather than repeated. Fresh browser reports are review-browser.json and review-verification.json under results/image-storage-limit-values.

## Limits

This scoped branch starts at dcde3e9c. The integration coordinator owns accepted431 and its shared-file union. Other raw options, writable private records, input-handler inspection, full original image suites, img01..48, final workspace gates and native Windows/Linux/macOS matrices remain unfinished311/324 and project work. Full equivalence is NOT PROVEN.
