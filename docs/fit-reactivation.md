# Fit reactivation during measurement

Overall equivalence is NOT PROVEN. The complete user intent remains in `docs/intent.md` and `PORT_REQUIREMENTS.md`. Task424 addresses only the terminal chosen by FitAddon.fit after measurement callbacks. Full utility/status closure323, original suites and final quality/platform evidence remain open.

## Behavior and repair

The unchanged source measures first, then reads the current terminal for resize. The Rust browser binding cloned the addon before measuring and kept that old host through resize. A synchronous getComputedStyle callback that activates another real terminal therefore resized the wrong terminal.

`crates/xterm-web/src/utility_addons/fit_bindings.rs` now obtains the proposal without holding a binding borrow, then clones the current terminal and resizes it. Missing measurements and NaN proposals still do nothing. Measurement and resize errors still propagate. Disposal still retains the terminal, as upstream requires. Geometry and CSS parsing remain in the existing Rust implementation. No JS terminal delegation, dependency, unsafe code or native behavior change was added.

The binding deliberately repeats the small optional-proposal and NaN guard from the native fit method. Native activation needs exclusive access, whereas the shared browser binding permits reactivation inside host callbacks. Calling the native fit method on a snapshot cannot observe that reactivation. Both paths reuse the same proposal calculation and terminal resize implementation.

## Verification

`crates/xterm-web/tests/fixtures/fit-reactivation.js` contains four assertion cases shared between unchanged source and Rust:

- An ordinary fit resizes the measured terminal.
- Reactivation during element getComputedStyle resizes the newly active terminal using the already measured proposal and leaves the original unchanged.
- Reactivation followed by dispose still resizes the current terminal.
- A measurement exception preserves the exact thrown object, resizes neither terminal and retains reactivation for the next fit.

The source oracle uses explicit DOM/terminal host fixtures and unchanged `addon-fit/src/FitAddon.ts`. This is not the original browser suite. `results/fit-reactivation/source.json` records four discovered/executed/passed cases, zero failures/skips/unrun, hashes and all776 reference entries unchanged before/after.

`wasm_fit_reactivation` runs the shared assertions against two actual Rust Terminals per case, each opened in its own browser DOM parent. It checks actual cols/rows after fit, not mocked resize calls. Its single test function passes all four cases in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. `before.json` retains the same final test-source hash failing in all three engines with `reactivate: latest terminal not resized`. `after.json` records the repaired pass. No assertion changed between those runs.

Further Windows checks at base main3d755b9d passed:

- `utility-browser.json` records the three existing `wasm_utility_addons` functions in all engines, including CSS padding/scrollbar/repeated-fit behavior and a real WebSocket test. Together with the new suite,12 browser test executions pass, zero fail/ignore. Four cases inside one function are not four test functions.
- `native.log` records eight passing utility_oracle and utility_live_terminal tests, including the existing source-derived Fit geometry corpus.
- `clippy-wasm.log` records warning-denied Clippy for both browser suites.
- `fmt-packages.log` records all five packages passing separate formatting checks. Exact `cargo fmt --all -- --check` still fails with Windows OS206 in `fmt.log`. The separate checks are not an exact full-command pass.

Node24.15.0, Rust/cargo1.85.1 and wasm-bindgen0.2.100 were used. Source assertions, AGENTS files, credentials and unrelated work were not changed. Generated binary/browser artifacts are disposable; retained reports contain their historical hashes/paths, not committed distributions.

## Reproduce

Use the existing original source dependencies and wasm-bindgen0.2.100. Set SOURCE_ROOT to the original reference, PLAYWRIGHT_MODULE to its `node_modules/playwright/index.mjs`, NODE_PATH to its `node_modules`, and WASM_BINDGEN to the matching CLI. Use CARGO_BUILD_JOBS=2 and keep the worktree, TMP/TEMP and private target on O: within the team heavy-job limit.

```sh
node scripts/fit-reactivation-oracle.mjs
WASM_TEST=wasm_fit_reactivation node scripts/browser-host-check.mjs
node scripts/utility-addon-browser-check.mjs
cargo test -j2 -p xterm-addons --test utility_oracle --test utility_live_terminal
cargo clippy -j2 -p xterm-web --target wasm32-unknown-unknown --test wasm_fit_reactivation --test wasm_utility_addons -- -D warnings
cargo fmt --all -- --check
```

The source runner removes scratch in finally; browser and WebSocket runners close their servers/clients in finally. Task424 owns only `O:/xterm-rust-swarm-16/tmp/C424`, including its private target, scratch and generated browser artifacts. Main14f25027 was inspected initially; the worktree was created from newer main3d755b9d. Accepted Base64418 at af8a07eb and pending404 were not replayed. Canonical integration remains309's responsibility. The isolated task424 reviewer accepted source15f5f157 with no findings and independently reran all scoped checks. `results/fit-reactivation/review.md` retains the scope. Safe owned cleanup follows review; final union verification remains task309's responsibility.
