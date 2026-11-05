# Live decoration root verification

This change installs the existing Rust BufferDecorationRenderer in BrowserServices and binds DecorationLineCache microtasks when Terminal is constructed, before open. It does not add the missing browser public marker/decoration API bindings. Overall equivalence remains NOT PROVEN.

Executed on Windows in the private O: Ricardo16 worktree, based on main 3ee576a. Node and browser versions, exact compiler/bindings/CSS hashes, command outcomes and per-browser output are in the two `*-root-results.json` reports. Paths in those reports identify the original temporary execution location. Binary build artifacts are disposable; source hashes, outputs and counts are retained here.

Commands ran sequentially with `CARGO_BUILD_JOBS=2`, the worktree's private target, and TEMP/TMP set to its own tmp directory. The browser driver used the existing wasm-bindgen 0.2.100 CLI and Playwright installation from the read-only reference checkout.

```sh
WASM_TEST=wasm_buffer_decoration node scripts/browser-host-check.mjs
WASM_TEST=wasm_terminal_decoration node scripts/browser-host-check.mjs
node scripts/buffer-decoration-oracle.mjs
cargo test -p xterm-core --test decoration_raw --test decoration_precision --test decoration_upstream --test decoration_edges --test decoration_review_edges --test decoration_differential --test marker_raw -j2
cargo clippy -p xterm-web --lib --test wasm_buffer_decoration --test wasm_terminal_decoration --target wasm32-unknown-unknown -j2 -- -D warnings
```

- Component suite: eight discovered/executed/passed tests per Chromium, Firefox and WebKit; zero failures, ignored or filtered tests. One test replays eight preserved actual-source DOM observations; one opens the exported Rust Terminal.
- Additional actual-Terminal regression: one discovered/executed/passed test per engine; zero failures, ignored or filtered tests. Checks pre-open cache synchronization, public render before overlay callbacks, one live overlay container, alternate/normal buffer transitions, resize, reset and cancellation of queued work on whole-terminal disposal.
- Source oracle: eight actual-source Chromium DOM observations, unchanged fixture SHA256 e791a05a1f374a30ec392a1301187b206dcb1920b6a7692129dd0f90348affdb, no drift. There is no original BufferDecorationRenderer unit test file, so original test-function count is zero. Full imported source hashes are in source.json.
- Native focused suites: 37 passed, zero failures or ignored tests. This includes ten faithful original DecorationService tests, the 1024-operation fixture replay and 81-case raw-marker fixture replay. Full test names and results are in root-native.log.
- Focused WASM library and both test targets pass warning-denied Clippy. Output is in root-clippy.log.

An exploratory full xterm-web WASM --tests check found the unrelated locked-js-sys Symbol::new fixture error and forwarded it to the workspace quality owner. That command was not a passing full WASM gate. No full workspace/platform/release or public-decoration API equivalence is claimed by this receipt.

Temporary ownership was limited to the O: Ricardo16 worktree and descendants. A temporary source_repo/node_modules junction reused the original installed tools for the source oracle and was unlinked immediately after the command, before any whole-worktree cleanup. No background service remains.
