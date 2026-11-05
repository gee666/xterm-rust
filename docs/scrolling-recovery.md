# Public scrolling and Viewport recovery

This repairs an integration omission, not the raw scroll-value contract. Full port equivalence remains NOT PROVEN.

## Recovered implementation

The launch handoff identified reviewed production changes missing from main even though their tests were integrated. Comparing `main..swarm/Joyce-integration` found one commit, `ae23b056c62dec67841c47aa9b054dc785cd7a11`, touching three production files. It was recovered without conflicts as `3dac46b` on `swarm/Margaret16`, based on main `3ee576a9b01c774d2305936ec9e44993df7f0b5d`.

- `terminal/operations.rs` routes opened public scroll methods through the existing Viewport. Clamping to buffer rows before this delegation lost the fractional pixel remainder after touch scrolling. Before open, the native core still clamps the line displacement.
- `terminal/browser_lifecycle.rs` disposes the owned Viewport. The previous omission left its DOM element, stylesheet and listeners alive.
- `viewport_styles.rs` uses the existing exception-preserving property adapter. A DOM style getter may be overridden. Its thrown value must return unchanged rather than cross an unchecked web-sys getter.

The routing follows unchanged `CoreBrowserTerminal.ts` lines702-734 and the public integer validation in `browser/public/Terminal.ts` lines203-219. All scroll methods converge on the same Viewport. There is no second position cache.

## Verification

Run on native Windows with rustc1.85.1, Cargo1.85.1, Node24.15.0, the locked wasm-bindgen0.2.100 CLI and the installed Playwright engines. New builds and scratch were in `O:/xterm-rust-swarm-16/tmp/worktrees/Margaret16`. Cargo used two jobs and a private regular `target` directory. The source dependency junction was created only inside that owned worktree and must be unlinked before worktree removal.

The clean-main baseline built successfully, but all three browsers reproduced the fractional position assertion, 54 versus46.8, and missing Viewport disposal. An overridden style getter threw a Symbol. wasm-bindgen-test's own failure formatter could not stringify it, so the runner reached its bounded60-second deadline. These are failed runs, not counted as successful tests. Raw console/exception evidence is retained in `results/scrolling-recovery/baseline-browser.json`.

After recovery, all21 existing lib tests passed in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4, with no failures, ignored tests or filters. These include actual Terminal composition, touch scrolling, style getter error identity and live Viewport state. The unchanged test sources were not edited. `recovered-browser.json` and the engine logs retain this run.

Two additional tests in `viewport_public_browser_tests.rs` cover the unopened native fallback, clamping large integral values, rejection of fractional/nonfinite input without mutation, direct opened `scrollLines`, zero displacement and same-row `scrollToLine` with a retained pixel remainder. The final run passed all23 tests in each of the three engines with no failures, ignored tests or filters. `final-browser.json` and `final-lib-{chromium,firefox,webkit}.log` retain that result.

Commands, from the owned worktree:

```sh
CARGO_BUILD_JOBS=2 WASM_TEST=lib node scripts/browser-host-check.mjs
cargo test -p xterm-web --test viewport_controller --test scrollable -j2
cargo clippy -p xterm-web --target wasm32-unknown-unknown --lib -j2 -- -D warnings
node scripts/viewport-source-check.mjs
```

Set `WASM_BINDGEN` to the matching installed CLI and `PLAYWRIGHT_MODULE` to the installed Playwright module when they are outside this worktree. Set `TEMP`, `TMP` and `CARGO_TARGET_DIR` inside the owned O: worktree. The source method checker ran10 unchanged Viewport method scenarios successfully. It is narrower than the actual browser constructor tests. Native tests passed5 Scrollable and11 ViewportController cases. Warning-denied WASM library Clippy passed. Focused rustfmt checks passed for all five changed Rust files. Full workspace formatting/lint/test, all-feature/release and native Linux/macOS gates remain separate team work, not claimed by this recovery.

Raw generated WASM/bindings are disposable build outputs. Their hashes remain in the browser reports; the binaries are not committed and will be removed during owned cleanup. Paths inside those reports describe the execution location, not permanent downloads.

## Independent review

Task289 review accepted the recovery at `6eb6766`. The reviewer confirmed the three recovered production files matched `ae23b05`, compared routing with unchanged upstream source, and independently reran all16 native cases,10 source cases,23 WASM tests in each engine, warning-denied WASM library Clippy and focused formatting. `independent-browser.json` and its engine logs retain that independent browser run. No tracked source changed during review. Canonical integration is a separate final step.

## Remaining scope

The numeric scroll model still does not preserve arbitrary raw JS values when `forceIntegerValues=false`. That affects ScrollState, smooth animation from/to values, Scrollable and Element consumers. Task301 and criterion458 track this separately. See retained `tmp/margaret-raw-scroll-findings.md` in the original launch checkout. Do not treat passing public numeric scroll tests as proof of this raw contract, complete scrolling equivalence, full browser-suite equivalence or another native platform passing.
