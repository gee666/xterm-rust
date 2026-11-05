# Search selection host after reactivation

Task469 follows accepted458. Full search347, family322 and full-port equivalence remain NOT PROVEN.

## Contract

The original SearchAddon checks activation before firing `onBeforeSearch`, but does not retain that terminal for the later selection phase. Before-search listeners, decoration getters, highlight getters and term validation can call `activate(otherTerminal)`. `_findNextAndSelect` and `_findPreviousAndSelect` resolve the current terminal and engine after those callbacks. An invalid term clears the current terminal's selection. A valid term captures and clears its selection before matching. The entry terminal's selection stays untouched.

A later selection-decoration getter runs after the engine has already captured and cleared selection. Reactivation there changes where the result is selected, not the engine position already used. The regression fixture includes this control so an implementation that redirects all phases indiscriminately cannot pass.

## Rust change

`crates/xterm-addons/src/search/find.rs` still checks activation before events. It now obtains the host again for invalid-term clearing and at engine selection capture after validation, instead of retaining the entry host across earlier callbacks. No new dependency, unsafe code, public API or JS search delegation. Matching-option getter reentry and raw mutable term matching are not closed by this patch.

## Verification

The shared `crates/xterm-web/tests/fixtures/search-live-host.js` runs against unchanged source and real Rust/WASM bindings. Its40 directed scenarios cover both directions, matches and misses, empty terms, before-search reactivation, initial decoration eligibility, both late highlight getters, two invalid raw-length validation phases, six thrown identities after reactivation, and late selection-getter controls. The two terminals have different selected columns, so the assertion checks position and clear destination rather than just final text.

`crates/xterm-web/tests/search_cases/live_host.rs` adds three native functions expanding28 scenarios through real terminal/selection/decorations services. Existing53 native search functions remain intact.

Observed on native Windows with Node24.15.0 and Rust/Cargo1.85.1:

- All48 inherited458 pins independently verified before reuse. Both private and original776-file reference manifests match.
- Unchanged source40 cases passed in each of Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4,120 total.
- Native negative control replaced only `find.rs` with accepted458 bytes. All three new functions failed because the entry terminal's selection was cleared. Zero passed,53 filtered. A `finally` restored the exact fixed bytes and recorded hashes.
- Restored native suite passed56 functions, none failed, ignored or filtered.
- Scoped native and WASM Clippy with `-D warnings` passed. Fixed WASM17-function executable built. Changed Rust formatting passed. Exact workspace formatting failed with Windows OS206, not a successful workspace gate.
- Actual Rust/WASM negative control replaced only `find.rs` with accepted458 bytes. Chromium passed16 functions and failed the new reactivation function. A `finally` restored the exact fixed bytes. The final suite passed17 functions in each of Chromium, Firefox and WebKit,51 functions total with no failures, ignored or filtered tests. The40 shared cases are inside one function per engine, not40 extra functions. Compiler artifacts contain the exact shared fixture bytes.
- Raw reports, command records, restore hashes and artifact provenance are in `results/search-live-host/`. Isolated independent review accepted the source/evidence checkpoint. It reran the hash-verified existing native executable,56PASS, and verified all37 then-current pins, all48 inherited pins, both776-file reference trees, both WASM variants, embedded fixture bytes, restore hashes and changed formatting. It inspected but did not rerun Cargo/browser matrices. The later review receipt adds the38th pin. Cleanup was not part of the review verdict.

Reproduction from a prepared checkout:

```sh
cargo test --locked -j2 -p xterm-web --test search_addon
cargo clippy --locked -j2 -p xterm-addons -p xterm-web --lib --test search_addon -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_search_addon -- -D warnings
SEARCH_CASES=search-live-host node scripts/search-call-args-source.mjs
WASM_TEST=wasm_search_addon node scripts/browser-host-check.mjs
```

Use the existing runners' external `SOURCE_DEPENDENCIES`, `PLAYWRIGHT_MODULE` and matching `WASM_BINDGEN`0.2.100 settings when tools live outside the owned checkout. During this swarm all targets and scratch must remain under the owned O: root and jobs must be coordinated with309.

No unchanged original Search E2E rerun, random corpus, complete workspace test/Clippy gate, demo, native Linux/macOS run or GC proof is claimed. Only commits after `archive/Leonard458-complete` belong to469. The private prerequisite history is not a canonical integration recipe.
