# Raw image boolean options

Task485 follows accepted474 and repairs raw flag identity at the browser addon boundary. Full image-addon parity remains open in311/324. Full Rust xterm.js equivalence is NOT PROVEN.

The source ImageAddon copies constructor options twice. It never converts enableSizeReports, sixelSupport, sixelScrolling, iipSupport or kittySupport to booleans in the stored option bag. It evaluates their truthiness only when deciding behavior. Symbols, bigints, functions, revoked proxies and objects therefore retain identity and do not invoke valueOf, toString or Symbol.toPrimitive. The current options use the first copy. Reset restores only scrolling from the second copy, alongside the existing palette reset. DECSET80 and DECRST80 replace scrolling with false and true. Unrelated modes, a value80 only inside a subparameter array, and graphics palette controls leave scrolling unchanged. Source Params.toArray flattens CSI80:1 to [80,[1]], so the scalar80 still changes scrolling. Both this handled case and the ignored CSI81:80 case are retained in the new tests.

Rust now retains those five raw flags beside ImageOptions. Native protocols continue using their existing boolean truthiness policy. Both raw and typed scrolling are updated before storage reset, handler reset or event callbacks can reenter. The diagnostic _opts view remains stable and read-only. It does not become a writable private JavaScript option bag. Other unchanged gaps include raw pixel/palette coercion, full private ABI, invalid IIP decoder payload settings, GC, unchanged full image suites and img01..48.

Five new browser test functions cover 22 raw forms on all five flags, exact identity and retained-view reads after owner release, two-copy getters and second-copy scrolling reset, ignored controls, real SIXEL buffer/cursor/image/pixel comparisons through controls and reset, 14 activation flag forms with actual protocol membership/window reports/DA1, and reset reentry from onImageAdded. All54 inherited browser functions and assertions are retained unchanged.

Corrected initial and final matrices pass59/59 in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. The five-file production control at accepted474 fdd3d621 completes56PASS3FAIL in Chromium. Exactly the new raw identity, two-copy getter and reset-reentry functions fail. Both real-protocol functions and all54 inherited functions pass. Finally restores all five production hashes before the final three-engine pass.

The first run completed58PASS1FAIL in each engine. Its new test incorrectly expected CSI80:1h to be ignored. The source Params.toArray contract disproved that expectation. The corrected test retains80:1 as explicitly handled and81:80 as an ignored subarray case. No source assertion or inherited test changed. After all three engines completed, the test server hung on open HTTP clients until the outer300s deadline. The initial raw logs and explicitly log-derived summary are retained, not relabeled a completed runner report. Accepted task495 separately fixes owned HTTP connection cleanup and adds four bounded regressions. Baseline Node results are3PASS4timeout cancellations, fixed and independent-review runs are7PASS. It changes no terminal behavior.

Warning-denied native web all-target/all-feature Clippy and final WASM library plus addon-test all-feature Clippy pass. All five package formatting checks, JavaScript syntax and git diff whitespace checks pass. Exact aggregate workspace formatting still hits Windows OS206. The audit verifies112 source/script pins,128 comparator inputs including121 original source files, both776-file manifests, all five restored production hashes, three actual Cargo compilations with fresh=false and seven retained WASM/binding/compiler artifact pins. Full workspace quality, native OS matrices, original639-family and img01..48 were not run by485.

Commands use Node24.15.0, rustc/cargo1.85.1, locked wasm-bindgen0.2.100 and the existing matching source reference dependencies. Set REFERENCE_NODE_MODULES, PLAYWRIGHT_MODULE and WASM_BINDGEN to their existing installations. New target, TEMP/TMP and scratch stay in the owned O:/xterm-rust-swarm-16/tmp/worktrees/RobertImageFlags tree.

```sh
cargo clippy --locked -j2 -p xterm-web --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown \
  --lib --test image_addon_browser --all-features -- -D warnings
WASM_TEST=image_addon_browser CARGO_BUILD_JOBS=2 node scripts/image-canvas-check.mjs
node results/image-raw-flags/negative-control.mjs
# Rerun the fixed matrix after the control restores all five product files.
WASM_TEST=image_addon_browser CARGO_BUILD_JOBS=2 node scripts/image-canvas-check.mjs
ORIGINAL_REFERENCE=/path/to/original/source_repo node results/image-raw-flags/audit.mjs
```

Retain the initial and final browser/verification JSON files under results/image-raw-flags before running another suite. The negative control restores only the five485 production files to accepted474 fdd3d621. It retains new tests and all original assertions, expects exactly three new failures and56 other passes, and verifies each fixed file hash in finally. The audit checks final source inputs, comparator inputs against the original reference, both776-file manifests, per-engine counters, restoration hashes and retained WASM/binding artifacts before owned cleanup. Temporary artifact paths record historical build identity, not permanent installations.
