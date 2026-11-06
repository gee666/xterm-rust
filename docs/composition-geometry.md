# IME overlay bounds

Task 570 repairs one input contract. Full input/accessibility task 319, the complete source suite matrix and full-port equivalence remain OPEN and NOT PROVEN.

## Defect and repair

`CompositionHelper.updateCompositionElements` applies `Math.max(bounds.width, 1)` and `Math.max(bounds.height, 1)` before assigning textarea CSS. JavaScript propagates NaN. CSS then rejects `NaNpx` and keeps the existing dimension. Rust's `f64::max` instead selects 1 when the other operand is NaN, replacing that dimension with `1px`.

The live browser adapter already calls the shared Rust `composition_geometry` function. That function now uses the crate's existing JavaScript-compatible `links::debounce::js_max`. No new dependency, unsafe code, browser authority or JS terminal algorithm was added. Browser/host code can replace `getBoundingClientRect`, so this contract is observable even though normal layout returns finite bounds. This is not a claim that normal browser layout produces NaN.

The source's one-pixel minimum remains intact for zero and negative bounds. Finite measurements, infinity and raw line-height handling are unchanged. The upstream source and assertions were not modified.

## Source and callable paths

- Oracle: `source_repo/src/browser/input/CompositionHelper.ts`, exact `updateCompositionElements` body.
- Native policy: `crates/xterm-web/src/input/composition_geometry.rs`.
- Live DOM route: `crates/xterm-web/src/input/dom/composition.rs`, compositionupdate and deferred geometry refresh.
- Public route: exported Rust `Terminal::open` and its real textarea composition events.
- Source body: `crates/xterm-web/tests/fixtures/composition-geometry-body.js`. The existing source-body drift test now checks this sixth method too.
- Shared browser driver: `crates/xterm-web/tests/fixtures/composition-geometry-cases.js`. Original JS executes only on its oracle side. Rust state and geometry execute on the actual side.
- Native regression: `nan_overlay_bounds_remain_nan_instead_of_becoming_minimum_size`.
- Browser tests: `wasm_composition_geometry.rs` plus `input_cases/composition_geometry_terminal.rs`.

The two browser tests compare real InputBindings and the exported Terminal with source behavior for seven bound pairs. They check immediate and deferred CSS results. The cases are finite, zero, negative, NaN width, NaN height, both NaN and infinity. InputBindings also verifies that geometry processing still permits the composed text to be sent. Source method bodies are not port-coverage counters.

## Observed evidence

Base is actual main `14139449bae2792afb019f42e8438b074938cfe7`. Source freeze is `8b9a01216ff526485079925090850a8b2eb6a68b`, following test commit `05c4d2e539ab3315cae8007f99dac6708fb3cd10`. There are no private producer prerequisites. Evidence lives in `results/composition-geometry/`.

On native Windows x64, Rust/Cargo 1.85.1, Node 24.15.0 and wasm-bindgen 0.2.100:

- Source tests: 13 passed, comprising six unchanged-body drift tests and seven geometry vectors.
- Old native policy: the new NaN regression failed, while the existing finite/minimum test passed.
- Final old-policy browser control: both tests failed in each of Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4. Each failure reports source width `7px` versus old Rust `1px` for NaN width. No page errors occurred in this final control.
- Fixed browser tests: two passed per engine, no failures, ignored tests, filtered tests or page errors. Each test exercises seven cases and two CSS phases, not just initialization.
- Existing `wasm_input_callbacks`: seven passed per engine with no failures, ignored tests, filtered tests or page errors.
- Native `cargo test -p xterm-web --all-targets --all-features`: 435 passed across 190 target summaries, zero failed or ignored.
- Warning-denied Clippy passed for native xterm-web all-targets/all-features and WASM library plus the new test target.
- `cargo fmt --all -- --check` returned Windows OS error 206. Separate exact package rustfmt checks passed for all five workspace packages. This is not a successful invocation of the full command.
- Both owned and original reference manifests contain 776 matching file hashes before and after the checks.

The first negative browser probe used test `unwrap` on the expected rejected comparison. All engines logged the intended NaN-width mismatch, but Firefox additionally trapped in dlmalloc during failed-test teardown and had no completed count summary. Its raw receipt remains `old-browser.json`. Tests now return the comparison error to the runner, without changing any comparison assertion. The repeated final old-policy control produced the same intended failures in all three engines without that teardown trap. Both receipts are retained. No claim is made that the earlier teardown failure is a repaired production allocator defect.

The first strict WASM check found four unused declarations caused by the new test importing shared input support without exercising its event/send helpers. The final test checks actual composition delivery after geometry, so those helpers and host observations have a behavioral use. No warning suppression was added. `initial-check-findings.md` records the observed diagnostic excerpt; the final complete strict log is retained.

## Reproduction

Use an owned O: worktree and private target, literal O: TEMP/TMP/TMPDIR, Cargo `-j2`, current measured resource coordination and the locked tool versions. Source dependencies are read-only oracle tooling, not a Rust runtime dependency.

```sh
node --test scripts/composition-source-fixture.test.mjs scripts/composition-geometry-source.test.mjs
cargo test -p xterm-web --lib input::composition_geometry
cargo test -p xterm-web --all-targets --all-features
cargo clippy -p xterm-web --all-targets --all-features -- -D warnings
cargo clippy -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_composition_geometry --all-features -- -D warnings
WASM_TEST=wasm_composition_geometry node scripts/browser-host-check.mjs
WASM_TEST=wasm_input_callbacks node scripts/browser-host-check.mjs
cargo fmt --all -- --check
```

Set WASM_BINDGEN to the matching installed CLI and PLAYWRIGHT_MODULE to the existing source dependency's `playwright/index.mjs`. All shell invocations need explicit timeouts. The browser runner starts bounded jobs and closes its browsers/server. The negative control restores only the two old max expressions and removes the now-unused import; its final test/fixture bytes match the positive run. Restore the fixed bytes in a finally path. Never mutate source_repo.

This task does not run or certify complete original suites, full workspace quality, release/MSRV/native Linux/macOS, all IME platforms, all decorated geometry getter/reentry contracts or final performance/distribution comparisons. Those requirements remain with the existing broader tasks.
