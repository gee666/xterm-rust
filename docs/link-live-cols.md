# Live columns during link hit-testing

Task 489 repairs column sampling when a link range getter resizes the terminal. Full selection/link parity and global equivalence remain NOT PROVEN.

## Source contract

`source_repo/src/browser/Linkifier.ts::_linkAtPosition` evaluates `range.start.y`, then reads the live buffer columns before multiplying. It reads `start.x` and completes the lower bound before evaluating the end bound in the same order. Only after both bounds does it read columns again for the current position. A y getter runs before the corresponding columns read. A y value's numeric coercion runs after that read. An x getter can change columns before the next bound or current-position calculation.

Previously Rust read a single columns value before any link getter, and calculated current before both bounds. This can select a link that the original rejects. For example, with start and end on row one, start x one and end x twenty, an end-x getter resizing from twenty to forty columns leaves the source upper bound at forty but moves current beyond it. The old Rust controller still compares against current calculated with twenty columns.

The payload now accepts a private live-column callback for controller hit-tests. All controller call sites use it, including current-link mouse movement, mouseup, cached and newly resolved provider probes, and selection. The existing public fixed-column method remains available and delegates to the same evaluation implementation. Native link payloads have no JavaScript getters and retain the typed range calculation. The callback only reads a `Cell`; no state borrow spans a JavaScript getter or coercion. No dependency, unsafe operation or JavaScript terminal-engine delegation is added.

## Tests

The method oracle executes the unchanged source TypeScript in memory. Ten cases cover unchanged values, start/end y getters, start/end x getters, start/end numeric coercions, two resizes that restore the original width, and exact thrown-value identity at each endpoint. It records the three columns reads relative to endpoint access and coercion.

The shared opened-Terminal program registers a public provider and resolves a pending real DOM mouse request with live range getters. It calls the public `resize` API from those getters and coercions. Ten cases assert hover count, event object and receiver/text identity, exact thrown marker identity, final columns and getter order. The same assertions run against the original bundle and Rust's exported Terminal. Original bundle source maps must match the original TypeScript.

```sh
SOURCE_ROOT=/absolute/reference/source_repo node scripts/link-live-cols-methods.mjs
SOURCE_ROOT=/absolute/reference/source_repo PLAYWRIGHT_MODULE=/absolute/reference/source_repo/node_modules/playwright/index.mjs node scripts/public-link-provider-source.mjs --live-cols --generate
WASM_TEST=wasm_link_live_cols CARGO_BUILD_JOBS=2 PLAYWRIGHT_MODULE=/absolute/reference/source_repo/node_modules/playwright/index.mjs WASM_BINDGEN=/absolute/wasm-bindgen node scripts/browser-host-check.mjs
```

The generated fixture is an original-source result. Do not change it to hide a Rust failure. Reports belong under `results/link-live-cols/` and `results/browser-host/`.

## Observed results

On integrated base `735e114a51444dd2f0542225a1bb1fadee0393ea`, the ten source-method cases passed. The original opened-Terminal ten-case trace passed Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4. Rust passed one test function replaying the same ten cases per engine. The two exact production files from that base compiled against the new assertions and failed Chromium with `end x: expected 0 hover, got 1`. The driver restored both final files in `finally` and verified their hashes before the final three-engine pass.

Six retained Rust targets passed per engine: public provider one function, provider transactions one function replaying six cases, hover-event sampling one function replaying seven cases, linkifier seven functions, disposal three functions and terminal links four functions. Those are 17 retained functions per engine, separate from the new ten-case function. These checks exercise actual exported Terminal and native-provider component callbacks, not the demo distribution or all original suites.

`cargo test --locked -j2 -p xterm-web --all-targets --all-features` passed 397 tests across 164 summaries, with zero failed or ignored. Native and WASM-target `cargo clippy --locked -j2 -p xterm-web --all-targets --all-features -- -D warnings` passed. All five package-format checks passed. Exact `cargo fmt --all -- --check` failed with Windows OS error 206; its raw log remains retained. No full workspace-format-command pass is claimed.

The initial shared trace tried to request another link on the same cached row. Its fresh-request assertion failed before the behavioral case. That diagnostic is retained. The final program alternates rows to request each case from the provider, retaining the fresh-request assertion rather than weakening it. The generated fixture comes only from the successful original three-engine run.

The initial compiler command set `TEMP` to the owned O: directory but a combined shell export left `TMP` inherited. Node resolved its temporary directory to the O: `TEMP`; outside compiler scratch was not inventoried. Later browser/compiler commands explicitly set both variables in separate statements, and final native tests and Clippy passed in that corrected environment. `environment-notes.json` records this limit. Private worktree and target paths stayed on O:. Unknown outside paths were not deleted.

All 776 original hashes passed preflight. Tools were Node 24.15.0, Rust/Cargo 1.85.1 and wasm-bindgen 0.2.100 on native Windows. Linux/macOS runtime proof is not claimed. `results/link-live-cols/index.json` pins source, receipts and temporary compiler/browser artifacts for independent review. Binaries are retained until that review, then removed with the owned worktree after preserving source and raw receipts.

## Remaining scope

This change does not close full selection/link task 321. Arbitrary mapped or sparse provider containers, prototype behavior, raw drawing and selection number domains, underline/selection geometry sampling, deeper current-link replacement during getters, and lifecycle/GC still require full source/public proof. Earlier accepted fixes, including provider transaction timing in 464 and hover event sampling in 478, must remain intact. Full original suites, demo matrices, workspace quality and native operating-system evidence remain separate requirements.
