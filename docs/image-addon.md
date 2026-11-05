# Public browser ImageAddon

Task386 exports `ImageAddon` from the Rust WASM module and `browser/xterm.mjs`. Use the latter as the supported JavaScript entry point. It retains activation hosts in the JavaScript heap; native parser, renderer and storage adapters use weak terminal handles. A JavaScript WeakMap keeps directly activated addons alive while their terminal remains reachable, without a Rust-held terminal/addon ownership cycle. There is no JavaScript terminal-engine delegation on the Rust side and no new dependency or unsafe code.

The coordinator connects the reviewed BrowserIipProtocol, BrowserSixelProtocol and BrowserKittyProtocol producers to one ImageRendererLifecycle and ImageStorage per activation. It owns graphics CSI handlers, DECSTR/RIS/reset-request listeners, public render, resize and buffer-change subscriptions. Protocol registrations capture their original activation. Parent handlers reread the current terminal/storage fields, including after repeated activation and reentry. Cleanup follows source insertion order and stops on the first error. A failed cleanup leaves the list, handler map and image emitter available for the next dispose call. Reset restores constructor scrolling/palette defaults, clears current storage and visits current protocol handlers without holding a state borrow across callbacks. Storage limit and placeholder choices survive reset.

Public bindings include `activate`, `dispose`, `reset`, stable `onImageAdded`, `storageLimit`, `storageUsage`, `showPlaceholder`, `getImageAtBufferCell` and `extractTileAtBufferCell`. Storage-limit setter failures preserve source RangeError name/message and browser callback exceptions retain their thrown value. Pending bitmap completion is not cancelled by addon disposal. It can insert into the original storage later, but the disposed emitter does not deliver another image-added notification. Enabled size-report flags remain enabled after disposal, as in source.

Constructor configuration covers the twelve declared option fields. Runtime and reset defaults are cloned separately, so primitive-valued getters run twice. SIXEL decoder initialization crosses a promise boundary, and data received while it is pending is discarded rather than replayed. IIP's portable size_limit now uses f64, matching the source's `ceil(limit * 4 / 3)` before checked encoded-capacity conversion. This avoids silently truncating fractional byte limits at the browser boundary. The native trace example accepts the same numeric representation. JavaScript and Rust default option records intentionally repeat the twelve ABI field values, because Object.assign and native typed configuration have separate representations.

## Verification

Observed locally on Windows with Rust/Cargo1.85.1, Node24.15.0 and wasm-bindgen0.2.100, initially based on main1729d89:

- Twelve directed Rust browser tests pass in each of Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. Each engine has zero failed, ignored or filtered tests and zero unexpected page errors. Expected source/Rust limit warnings remain in the reports.
- The tests bundle unchanged source Terminal and ImageAddon. They compare complete cells, widths, attributes, image/tile IDs, wrapped rows, cursor/scroll positions, replies, image-added counts, storage usage/limit, placeholder state, retrieved image pixels and extracted tile pixels. Snapshot width follows current columns after resize. Browser-frame boundaries are awaited before observing rendering-dependent usage; arbitrary sleeps are not used.
- Cases exercise all three producers sharing storage, enabled and disabled protocols, pre-open graphics queries, configured pixel/size limits including fractional IIP capacity, palette and geometry reports, constructor getter/default copies, scrolling controls, DECSTR/RIS, alternate-buffer clearing, resize, public terminal reset, repeated activation with old-protocol/current-parent contexts, image-added reset reentry, successful and throwing disposal, late bitmap insertion and suppressed post-disposal events.
- A separate directed test executes the shipped bootstrap's class bodies against the current WASM instance, changing only ES module directives for the test loader. It constructs Terminal and ImageAddon, calls Terminal.loadAddon, writes actual QOI input, verifies image pixels/storage/events and checks managed addon disposal. It also compares the invalid storageLimit error against unchanged source.
- Native workspace all-target/all-feature tests pass2044 tests across402 summaries, with zero failed or ignored tests. Native execution does not run WASM assertions. The focused native IIP target passes11 tests, including a new fractional-limit regression.
- Exact workspace formatting, native workspace all-target/all-feature warning-denied Clippy and scoped WASM warning-denied Clippy are recorded separately. A long owned worktree path first hit Windows OS206 for workspace rustfmt. Moving the same worktree to the shorter O:tmp/M path lets the exact command run; no formatter gate is waived.

Raw reports, source/input hashes and concise gate logs are retained under `results/image-addon`. Generated browser/WASM binaries and intermediate run directories are temporary; their hashes and observed results, not continued availability at the absolute temporary paths, are the retained evidence. New code is under the 350-line file limit.

## Reproduce

Use the reference dependency/tool environment from docs/image-canvas.md. The supplied source tree is read-only.

```sh
cargo fmt --all -- --check
cargo test --locked -j2 --workspace --all-targets --all-features
cargo clippy --locked -j2 --workspace --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test image_addon_browser --test image_iip_terminal_browser --test image_sixel_terminal_browser --test image_kitty_terminal_browser -- -D warnings
WASM_TEST=image_addon_browser CARGO_BUILD_JOBS=2 node scripts/image-canvas-check.mjs
```

## Remaining scope

This is executable coordinator coverage, not complete image-addon equivalence. Tasks311/324 still require the unchanged full original image suites, img01..48, broader seeded protocol/coordinator traces and exact suite mapping. Arbitrary raw JS option values are not equivalent yet: the typed coordinator eagerly converts numeric option values, whereas source can retain objects and defer coercion; raw placeholder values, patched-host partial registration behavior, internal private-object facades and full garbage-collection/lifetime adversarial tests remain open. The native encoded-limit boundary rejects nonfinite/out-of-range capacities safely; raw source constructor failure semantics need separate checks. The supported bootstrap owns host retention; direct use of generated WASM addon classes alone does not provide that JavaScript ownership contract. Full native OS and final integrated matrices remain separate requirements. Overall equivalence remains NOT PROVEN.

## Refreshed integrated base

The coordinator commit rebased onto current mainaa3f792, which already contains the reviewed font, serializer, mouse and marker-resource changes. On that union the final twelve-addon-test matrix and all five regression matrices pass in all three engines: IIP12, SIXEL15, Kitty19, canvas23 and lifecycle13 tests per engine. Full native workspace all-target/all-feature execution passes2061 tests across410 summaries with zero failed or ignored tests. Exact workspace formatting, native workspace and scoped WASM warning-denied Clippy, and both changed JavaScript syntax checks pass.

A negative control removed the coordinator's custom DA1 response. Chromium rejected it with two failed and ten passed tests, with no unexpected page errors. The mutation was restored before every final browser matrix and final quality gate. The retained negative report is not a production failure or an equivalence result.

Earlier directed-fixture attempts found two mistakes in new tests, not weakened source assertions: a one-pixel QOI fixture was below the source metric sniffer's32-byte minimum, and one query string was accidentally double-escaped. The corrected QOI fixture has real2x2 pixels, and reply tests assert explicit nonzero counts and exact query replies. Rendering-dependent storage usage is sampled after the actual requested browser frame. The initial unframed observation was a timing mismatch in the new fixture, not justification to change renderer behavior.
