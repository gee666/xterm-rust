# Full Terminal scrollbar construction

Task #354 fixes an observable scrollbar option getter mismatch in the real browser Terminal. `CoreBrowserTerminal.open` reads `showScrollbar` and then `width` unconditionally before deciding whether to create its overview renderer. A later scrollbar option change deliberately short-circuits `width` when hidden. Rust previously used that later-change rule for both paths. Hidden initial width getters never ran, including their side effects and thrown values.

`terminal/overview_services.rs` now separates the initial reads from update evaluation and shares renderer creation. The initial width read fetches the live scrollbar option again after the visibility getter. It does not hold an options or renderer borrow across JavaScript getters. The update path retains its short-circuit behavior and same-canvas lifecycle.

## The retained Viewport failure

The existing `viewport_browser_tests` assertion expected four getter reads after opening a complete Terminal. That was correct for the isolated Viewport option literal, but stopped describing the full Terminal after overview integration. Full unchanged source construction with a visible width of 18 reads ten properties: four for Viewport, two for the overview creation decision, and four for its two width calculations. The new source oracle runs the actual supplied public Terminal, with all imported source hashes retained. All three original engines agree on that exact ten-read trace.

The full-composition assertion now checks all ten reads. A separate assertion retains the original exact four-read Viewport option-literal contract. Existing DOM, theme CSS, styles, scrolling and disposal assertions remain intact. No original source file or source assertion was changed, and the additional six reads were not removed from the implementation to satisfy an outdated scoped expectation.

## Directed coverage

`tests/scrollbar_construction_cases.js` is shared by source and Rust. Its eight independent exact expectations cover a visible ruler, hidden initial width reads, the original thrown Symbol identity, zero width, a different second width value, hidden option-update short-circuiting, a visibility getter throwing at the overview phase, and a width getter changing visibility before overview creation. Tests run synchronously before animation frames can add unrelated renderer reads. The Rust path uses the actual Rust public Terminal and DOM services, not the original engine.

The pre-fix Chromium comparison fails scenario 1 because Rust omits the final width read. `results/scrollbar-construction/before-fix.json` retains its exact trace, command and artifact hashes. The initial full-lib attempt with the new isolated assertion had a test-only `PublicOptions` versus `OptionsService` type error. It was fixed by obtaining the existing service through native state, not by removing the assertion.

## Reproduce

With the exact supplied reference and dependencies available:

```text
REFERENCE_ROOT=/path/to/source_repo node scripts/scrollbar-construction-oracle.mjs
WASM_TEST=scrollbar_construction_browser node scripts/browser-host-check.mjs
WASM_TEST=lib node scripts/browser-host-check.mjs
WASM_TEST=overview_ruler_browser node scripts/browser-host-check.mjs
WASM_TEST=overview_terminal_browser node scripts/browser-host-check.mjs
cargo test -p xterm-web --lib -j2
cargo clippy -p xterm-web --target wasm32-unknown-unknown --lib --test scrollbar_construction_browser --test overview_ruler_browser --test overview_terminal_browser -j2 -- -D warnings
```

The browser runner also accepts `WASM_BINDGEN` and `PLAYWRIGHT_MODULE` to reuse installed tools. Use private O: work and explicit TEMP, TMP and TMPDIR paths. Both runners close their browsers and HTTP server. The source oracle removes its unique scratch. Generated bundles and binaries are disposable; retained JSON records their hashes, not permanent artifact availability.

## Observed scope

On native Windows with Rust 1.85.1 and Node 24.15.0, the original eight cases passed separately in Chromium, Firefox and WebKit. The new Rust test passed all eight cases per engine. On base `3baa698` plus this repair, WASM library tests passed 24/24 per engine, existing overview fixture replay passed one test per engine, and existing live overview integration passed two tests per engine. The fixture replay still covers all four original canvas cases. Native web library tests passed 54/54 and scoped warning-denied WASM Clippy passed. These counts are test functions unless explicitly called cases.

After rebasing onto main `4a38b52267251b4fb7b6476fae262299ae939145`, all four browser suites passed again in all three engines. The library now executes 26 tests per engine because that integrated base includes two additional keyboard tests. The eight directed cases, one overview fixture test and two live overview tests still pass separately. Native 54/54 and scoped warning-denied WASM Clippy passed again. `refreshed/` retains these results. Exact full `cargo fmt --all -- --check` passed after moving the owned worktree to the shorter O: path. The first long-path attempt's OS206 diagnostic remains retained, not reported as a pass.

`results/scrollbar-construction/` retains the commands, source hashes, failure and passing receipts. `results/coordination/amrapali-node-3baa698.log` separately records 54 passing benchmark-contract/full-state parser validation tests. Those are verifier checks, not measured terminal benchmarks. This directed repair does not close full browser root/layout task #317, raw option reentry generally, the complete upstream suite, native Linux/macOS or overall equivalence. Full equivalence remains NOT PROVEN.
