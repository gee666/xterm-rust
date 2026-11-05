# Numeric NaN HTML scrollback

Task481 repairs only numeric primitive `NaN` passed as HTML `scrollback`. Full raw serialization346, family322 and overall equivalence remain NOT PROVEN.

## Behavior

The original `_serializeBufferAsHTML` reads buffer length, `options.scrollback`, terminal rows, and clamps `scrollback + rows`. Numeric `NaN` survives `Math.min` and `Math.max`. The resulting start row is NaN, so `row <= endRow` is false. HTML header construction still reads global-background and font options, preserves UTF-16 strings, runs their effects, and propagates their exact thrown values. No row fragments or missing-cell warnings occur.

Rust previously reused the ANSI scrollback helper and raised `RangeError` before those HTML callbacks. `crates/xterm-web/src/serialize_addon/html.rs` now recognizes this primitive input after the existing length/scrollback/rows reads. It uses the same empty integral row range as the accepted explicit-NaN/reversed-row path. The shared Rust HTML implementation still constructs the header and footer normally. This does not change ANSI's array-length failure, native APIs, clamping of other inputs, source fixtures, or arbitrary replaced terminal methods.

Objects or strings whose arithmetic produces NaN, fractional scrollback, raw method replacement, other coordinate gaps, palette-array mutation, and broader GC coverage remain346 work. No source-hang behavior is approved here.

## Source and integration

The producer starts at main `3a4f5e8f`. `e0e941b7` adds two source/Rust differential probes and scoped intent. `ac5db73c` is the repair. Later commits retain proof and review. No accepted467,456 or446 history was replayed. Main already contains456 and446. Accepted467 is not in this branch. Its two oversized-column probes and these two NaN-scrollback probes must both survive later integration, giving74 cases rather than either branch's72.

The probes compare complete output, warning arrays, getter logs, thrown-value identity and terminal snapshots after scrollback/header resize and reset. The throw probe separately interrupts includeGlobalBackground, fontFamily and fontSize.

## Observed results

All evidence is under `results/serialize-nan-scrollback/`. `summary.json` pins46 pre-review evidence files, six source/runner files and16 native executables. Build reports retain exact source/package identities. The fixed package input hash is `bbe0503468ef723897da58c447bc316f70ffbf3bf020adb7e4c8f09db26f09cc`.

- Baseline e0e941b7 matches70/72 cases in Chromium148, Firefox150 and WebKit26.4. Only the two new probes fail. The runner exits1.
- Fixed ac5db73c matches72/72 in each engine, including32 deterministic seeded cases. The runner exits0.
- Both runs execute the unchanged original SerializeAddon suite against source and Rust. Each engine implementation passes120/120 expanded cases, zero failed, skipped or unrun, on both runs. These are source120 plus Rust120 per run, not120 per browser.
- All776 reference hashes match before and after both runs and during the final producer audit.
- All16 native serialization test executables pass69 tests, zero failed/ignored/filtered. This is scoped native coverage, not a workspace pass.
- Warning-denied Clippy passes for core library plus all16 serialization tests and for the actual WASM web library and dependencies. `cargo fmt -p xterm-web -- --check` passes. Exact `cargo fmt --all -- --check` fails with Windows OS206 before formatting; the retained log is not a passing workspace gate.
- Windows Rust1.85.1, Cargo1.85.1, wasm-bindgen0.2.100. Native Linux/macOS, full workspace all-feature/release checks and complete browser/demo acceptance were not run for this task.

## Reproduction

From an owned O: worktree, use the existing team resource allocation and original reference with its installed Playwright dependencies. Set TEMP/TMP/TMPDIR to the owned worktree's `tmp/runtime` and CARGO_BUILD_JOBS=2. Set WASM_BINDGEN to the installed0.2.100 CLI and SOURCE_ROOT to the supplied unchanged reference.

```sh
node scripts/build-demo.mjs --debug
node scripts/serialize-browser-check.mjs --suite
cargo test --locked -j2 -p xterm-core --test serialize_html --test serialize_reentry
cargo clippy --locked -j2 -p xterm-web --lib --target wasm32-unknown-unknown -- -D warnings
cargo fmt -p xterm-web -- --check
cargo fmt --all -- --check
```

`commands.json` records the full16-executable native command, including all `crates/xterm-core/tests/serialize*.rs` targets. The two named native targets above are a small reproduction subset, not the69-test total. Check out e0e941b7 for the negative control or ac5db73c for the fixed implementation. Do not replace either reference assertions or accepted467's probes during integration.

Only known owned temporary root is `O:/xterm-rust-swarm-16/tmp/worktrees/SuryavarmanHtmlScrollback`, including target, copied reference, generated browser package and runner scratch. The initial baseline-build shell command assigned TEMP there but expanded inherited TEMP for TMP/TMPDIR in the same export invocation. Those inherited values were not recorded, so this task cannot prove every compiler scratch file used O:. Later commands assign TEMP first, then the other variables. No unknown external scratch paths were deleted. Historical files copied with the default browser results directory were removed from these new evidence copies before review; only fresh reports, traces, screenshots and suite output are retained. No original reference junction was created. Cleanup and independent-review receipts follow separately so the pre-review evidence pins remain stable.
