# Raw IIP size limit

Task474 repairs the `iipSizeLimit` constructor and activation boundary after accepted465. Full image-addon parity remains open in311/324.

The source `ImageAddon` copies constructor options twice without converting this value. It retains the first copy for activation. Disabled IIP support never converts it. Enabled activation evaluates `Math.ceil(this._opts.iipSizeLimit * 4 / 3)` once when constructing the IIP handler. Renderer, storage, controls and enabled SIXEL registration already exist at this point. Kitty registration follows IIP. A thrown conversion must retain the earlier work and skip the later registrations.

The Rust browser coordinator now retains the first raw value and exposes its identity through the existing read-only `_opts` view. It evaluates the exact source expression outside state borrows, then passes the cap to `BrowserIipProtocol::new_with_encoded_limit` and `IipHandler::new_with_encoded_limit`. Exact expression spelling matters because some browser engines include it in generated TypeErrors. The small JavaScript helper only performs boundary arithmetic. Rust still owns headers, base64 payload limits, protocol state, geometry, codecs, storage, events and pixels. No dependency or unsafe code was added.

The existing typed constructors compute their cap and delegate. A private IIP decoder adapter distinguishes source-accepted construction from memory initialization and shares all actual byte decoding with `PayloadDecoder`. A cap above the source maximum still throws `invalid byte settings` at construction. NaN, negative and negative-infinite caps can construct and answer geometry reports without allocating a decoder. They fail safely with `InvalidSettings` if payload initialization is attempted. **That payload error behavior is not claimed equivalent.** The source evaluates those settings when initializing its WebAssembly decoder. These payload outcomes remain a compatibility gap, not an approved divergence. Tests never feed browser payloads through those settings, and Rust does not attempt invalid memory sizing. Zero is different: nonempty payload exceeds a zero cap and is safely rejected by both implementations.

## Verification

Six new browser functions cover19 raw constructor forms, first-versus-second copy selection, disabled support, repeated activation, number-hint callback counts, exact thrown values and engine messages, prior SIXEL registration, skipped Kitty registration, reset/dispose reentry, six invalid-numeric activation-only forms and14 finite numeric forms through real IIP payloads. The finite cases assert both sides of the41 versus41.5 decoded-byte threshold and compare resulting storage, terminal state, events and pixels. They reuse the accepted Kitty activation host without changing its assertions.

Initial and final addon matrices pass54/54 in each of Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. The control restores eight modified production files to accepted465 `7b20151c`, leaving the new decoder module unreferenced and retaining all tests. Chromium completes49PASS5FAIL. Precisely the five new boundary tests fail; the finite numeric payload test and all48 inherited tests pass. Finally restores all eight fixed files byte-for-byte before the final passing matrix.

The existing typed IIP terminal suite also passes12/12 in all three engines. Native focused tests pass19/19 across `image_iip`, `image_iip_raw_limit`, `iip_payload` and `iip_warning`. The existing source policy oracle passes9925 comparisons with seed `0x13372026`, including1000 randomized protocol cases,1000 resize cases and3000 number-format cases. It records image requests, not native pixels or original upstream assertions. Its initial invocation exposed a stale numeric `sizeLimit` wire field while the current Rust replay expects IEEE754 hex bits. The script now encodes and decodes that field consistently on both sides. No source assertion or expected state changed. The initial failure is retained.

Scoped native and WASM Clippy pass with warnings denied. Each of the five workspace packages passes its own formatting check. The exact aggregate `cargo fmt --all -- --check` still fails Windows OS206. An initial new-test compile failure from an invalid `IipGeometry::default` assumption is retained alongside the corrected passing result. Full workspace quality and native OS matrices were not run by474.

`results/image-iip-size-limit/` contains the reports, controls, logs and audit. The audit checks117 Rust/test/script pins,129 comparator inputs including121 original source files, all776 original manifest entries, eight restored production files and13 native/WASM/binding artifact pins. The runner overwrites its generated comparator entry between suites. The audit reconstructs each entry from the pinned generator, checks its retained hash and labels the two reconstructed entries. It does not relabel them original source. Temporary artifact paths are historical build identities, not permanent installations.

## Reproduction and integration

Use matching reference dependencies and wasm-bindgen0.2.100 settings from `docs/image-canvas.md`.

```sh
cargo test --locked -j2 -p xterm-addons --test image_iip \
  --test image_iip_raw_limit --test iip_payload --test iip_warning
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown \
  --lib --test image_addon_browser --test image_iip_terminal_browser -- -D warnings
WASM_TEST=image_addon_browser CARGO_BUILD_JOBS=2 node scripts/image-canvas-check.mjs
node results/image-iip-size-limit/negative-control.mjs
WASM_TEST=image_iip_terminal_browser CARGO_BUILD_JOBS=2 node scripts/image-canvas-check.mjs
node scripts/image-iip-oracle.mjs
```

This branch starts directly at accepted465 `7b20151c`. Integrate only its new commits after the coordinator's451/465 union. Do not replay private455 or the451/465 ancestors. Owned temporary work is under `O:/xterm-rust-swarm-16/tmp/worktrees/DerrickIip` until cleanup. Writable private options, other raw options, full `_core._inputHandler` access, GC, unchanged639-family and img01..48 remain open. Full Rust xterm.js equivalence is NOT PROVEN.
