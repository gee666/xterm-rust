# Raw keyboard lookup recovery

Task343 recovers retained implementation from `swarm/Aisha`, notably commits `16b12b2` and `6fd9fa9`, onto integrated main `c0dd42b`. The historical branch was inspected before applying selected files. Its unrelated implementation, stale test-host changes and old results were not replayed. Full input/accessibility parity remains task319 and full terminal equivalence is NOT PROVEN.

## Behavior and ownership

Upstream `common/input/Win32InputMode.ts` and `KittyKeyboard.ts` use ordinary objects, including mutable own entries and inherited getters. The portable policy now retains their lookup results until the actual conversion expression:

- `keyboard_event_source::lookup::{Value, HostValue, Hint}` holds portable values or opaque host handles. `KeyboardEventSource::win32_lookup` and `kitty_lookup` run at source field-read boundaries. Number inspection never coerces a host object.
- Win32 preserves Vk and Uc unless undefined, and applies truthiness only to Sc. It collects all fields before interpolating Vk, Sc and Uc in order using the string hint.
- Kitty applies truthiness to letter/SS3 values, and undefined checks to tilde/functional values. It retains the first functional result across the second lookup, uses strict primitive Number equality for Enter/Tab/Backspace, and converts with string addition's default hint.
- `input::keyboard_lookup::KeyboardTables` owns persistent browser objects alongside InputBindings. Lookups include own entries, retain the actual receiver and do not hold Rust borrows across getters or conversion callbacks. Its historical `Win32Tables` alias is retained.
- Attributed `kitty_tables.js` and `win32_tables.js` intentionally duplicate only source table literals and two host conversion operations. Their drift checkers require exact source literal content. Rust owns encoding decisions. No engine delegation, dependency or unsafe code was added.
- Native events have no mutable JavaScript realm. Their default intrinsic names use documented Node24/V8 text from `keyboard_map_inherited.rs`; browser events bypass that table and use actual runtime values. All output remains UTF-16, including lone surrogates.

The only change to the current DOM keyboard binder is passing its persistent table owner to `LiveKeyEvent`. Task292's lazy InputEvent reader and source-derived IME/clipboard test fixes remain intact. Existing source assertions were not changed. The generated test-only source fixtures exceed350 lines to retain complete auditable upstream bodies, not production JavaScript encoding.

## Commands and current evidence

Local environment is native Windows x86-64, Node24.15.0, Rust/Cargo1.85.1 and wasm-bindgen0.2.100. Builds use private O: targets and Cargo-j2. Results are under `results/keyboard-recovery343/`; ordinary encoder summaries retain their existing `results/kitty-keyboard`, `results/keyboard` and `results/win32-input` locations.

```sh
node scripts/generate-kitty-dom-tables.mjs
node scripts/generate-win32-dom-tables.mjs
node scripts/kitty-lookup-fixture.mjs
node scripts/win32-prototype-fixture.mjs
node scripts/kitty-lookup-controls.mjs
node scripts/win32-independent-controls.mjs
node scripts/verify-kitty-keyboard.mjs
node scripts/verify-keyboard.mjs
node scripts/verify-win32-input.mjs
node --test scripts/input-event-fixture.test.mjs
cargo test -j2 -p xterm-core --lib --test kitty_upstream --test kitty_edges --test keyboard_live --test win32_lookup --test kitty_lookup
cargo test -j2 -p xterm-web --test keyboard_dispatch --test keyboard_dispatch_callbacks --test keyboard_dispatch_getters
cargo clippy -j2 -p xterm-core --lib --test win32_lookup --test kitty_lookup --test kitty_edges --test kitty_upstream --test keyboard_live -- -D warnings
cargo clippy -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_keyboard_live -- -D warnings
cargo fmt --all -- --check
node scripts/keyboard-realm-node.mjs
WASM_TEST=lib node scripts/browser-host-check.mjs
WASM_TEST=wasm_keyboard_live node scripts/browser-host-check.mjs
WASM_TEST=wasm_input_event node scripts/browser-host-check.mjs
node scripts/keyboard-recovery-report.mjs
```

Set WASM_BINDGEN and PLAYWRIGHT_MODULE to existing installed tools if they are not on PATH. The reference copy is physical and unchanged. Node realm temporary bindings are removed in finally. Browser runners close browsers and their HTTP server in finally; temporary generated artifacts are removed after retaining reports.

Fresh checks so far:

| Check | Result |
| --- | --- |
| All seven generated binding table literals and two test-source fixture drift checks | Pass |
| InputEvent and CompositionHelper source-body integrity | 2 tests pass |
| Kitty original and differential | 165 tests,173 original assertions,508249 comparisons pass, seed1263096916 |
| Legacy original and differential | 61 tests,156 original assertions,251452 comparisons pass, seed1262836051 |
| Win32 original and differential | 64 original tests,13904 comparisons pass, seed5714738 |
| Core library plus retained native keyboard targets | 80 tests pass,67 library and13 integration |
| Dispatch native targets | 14 tests pass |
| Added portable raw Kitty contracts | 4 tests pass,84 native core tests across the two commands |
| WASM library and live keyboard test compilation | Pass |
| Scoped core and WASM warning-denied Clippy | Final reruns pass including added native Kitty tests |
| Exact full workspace rustfmt | Passed before rebase;8ab2dd3 inherited marker-module ordering fails, separate fix559daa9 pending integration |
| Node/WASM raw realm | 2 tests pass,10224 Kitty and592 Win32 comparisons |
| Chromium/Firefox/WebKit raw realm | Both raw tests pass in every engine with the same10224+592 comparisons |
| Complete WASM library in every browser | 24 pass,1 viewport test fails,0 ignored/filtered |
| Real DOM keyboard in every browser | 15 tests pass,0 failed/ignored/filtered |
| InputEvent regression in every browser | 4 tests pass, including288 guard combinations per engine |

Source-only negative controls detect missing prototypes in4440 cases, wrong string hints in1080, premature numeric conversion in2166, lossy UTF-16 in2048, and replaced exceptions in924. A separate Win32 probe detects all five deliberate mutations. These controls validate the comparator, not Rust behavior. Raw control results remain in `results/kitty-realm/negative-controls.json` and `results/win32-independent/negative-controls.json`.

The raw realm matrix compares10224 Kitty cases and592 Win32 cases per host, including own/inherited entries, receiver and persistent mutation, falsy values, native intrinsics, BigInt/Symbol/boxed/proxy values, throwing lookup/conversion, strict special-key equality and reentry. It preserves the retained560 Win32 cases and adds32 changing-key-to-empty getter traces. The hypothesis that these should emit undefined was disproved by the unchanged source's explicit `codePointAt(0) || 0`; no production codepoint change was made. Real textarea tests cover the production listener's callback mutation, conversion-error identity, persistent tables, mode switching, nested dispatch and disposal.

The first exact workspace rustfmt attempt failed with Windows OS206 at the long private path. A safe Git worktree move to `O:/xterm-rust-swarm-16/tmp/H` allowed the unchanged exact command to pass, without a junction or partial-format substitute. Both logs are retained. A subsequent clean rebase onto main8ab2dd3 left all23 recorded implementation/fixture hashes unchanged, but introduced that main revision's existing terminal/mod.rs marker_api declaration-order formatting failure. Its separate reviewed fix559daa9 is owned by Milica and queued for canonical integration. `fmt-after-rebase.log` retains the current failure; the earlier pass is not claimed as a current full quality gate.

## Retained failure and remaining scope

The unfiltered WASM library suite runs25 tests in each browser. Both raw keyboard tests pass, but `viewport::browser::tests::constructor_uses_actual_scroll_element_source_getter_order_and_theme_css` fails at `viewport_browser_tests.rs:116`. Its expected getter trace has4 entries and the actual trace has10. No viewport source/assertion was changed or failure normalized. The complete failed reports and three logs are retained alongside the passing DOM/InputEvent receipts. Post432 routes this failure to task317. `verification.json` records the library failures separately from scoped raw keyboard success.

No native Linux/macOS runtime, final full workspace feature/release/platform matrix, complete unchanged input/accessibility suite or whole-terminal equivalence is claimed. In particular, admitted InputEvent second reads returning non-string values remain task319, as do raw event fields outside declared primitive types. This recovery does not coerce those values to empty text or waive their lifecycle/error-order contracts.
