# Private DSR raw option callbacks

The source InputHandler.deviceStatusPrivate handles CSI ? 996 n by reading rawOptions.vtExtensions, optionally reading colorSchemeQuery, defaulting null or undefined to true, then testing truthiness. The event fires synchronously before parsing continues. Additional parameters do not change this first-parameter dispatch. Primitive bags must not undergo numeric or string coercion.

Before task 534, replies.rs called TerminalState::mode_option during a mutable TerminalState borrow. A raw bag or nested getter could not inspect, reset, resize or replace terminal state without a Borrowed failure. The shared command now resolves both getter phases outside terminal-state borrows, using the existing option_flag policy. Only then does it enqueue ColorSchemeQuery for the ordinary synchronous dispatcher. Other DSR commands retain the original state-only implementation. The now-unused mode_option forwarding helper is deleted, not suppressed. No unsafe code or dependencies were added.

## Source and native corpus

`scripts/dsr-option-oracle.mjs` executes the unchanged headless public Events/onData test and compares a generated 468-case fixture against the actual supplied source. The two access phases, two buffers, nine values and thirteen actions cover optional/nullish defaults, primitive boxing, truthiness without coercion, reset, buffer activation, resize, cursor/mode changes, disableStdin, replacement of the raw bag, original throws, mutation before throws, and color-query observer reset/throws. Each trace continues with private CPR and printed text to expose early aborts and live continuation. Getter and event observations plus final snapshots compare both buffers, every cell and attribute, cursor, dimensions, margins and relevant modes. Event order and thrown identity are separate checks.

The generated fixture exceeds 350 lines because each line retains one complete source case. Both snapshot implementations are existing shared helpers. The native store and callback closures retain weak state/store references to avoid cycles.

## Verification status

Source generation passed 468 cases, four non-996 short-circuit commands and one unchanged headless assertion, with all 776 reference hashes verified before and after. The native regression passed all three test functions. Replacing exactly four production files with main11ad rejected all 468 cases with the intended Borrowed failures, exit 101. Finally restoration verified all four fixed hashes, and the regression passed again.

Native Windows core all-target/all-feature tests passed 1,577 tests across 252 summaries with no failures, ignored, measured or filtered tests. Strict native core Clippy and actual WASM xterm-web dependent library check and Clippy passed. Full cargo fmt failed with Windows OS206, retained verbatim. Each of the five package-wide fmt checks passed. Three historical generated report changes are retained reversibly with before/after bytes and hashes, and their original bytes were restored. Receipts are under results/dsr-option.

The unchanged six-case browser DSR subset passed in Chromium, FirefoxStable and WebKit against both engines, 18 original cases plus 18 Rust cases. The original transport bundles the actual source Terminal and UnicodeGraphemesAddon required by unchanged TestUtils. The Rust phase uses the actual debug WASM distribution and existing Rust-only fixture, not that source bundle. Its 1,838-file on-disk build identity is 6b03e51502ab9f03b1e5a08be135b8a4daa58672c2bb94828151b1b546d202e6, and the runtime WASM hash is bfe3d61601f8aa0aabb91161d038fccbc596b1ba372faab2bfbd662ed923e5a0. Node was 24.15.0, Playwright 1.60.0 and Rust/Cargo 1.85.1 on native Windows.

Three failed original setup attempts remain explicit. The initial transport lacked UnicodeGraphemesAddon, producing three beforeAll failures and 15 unrun cases. Adding that real addon initially exposed a missing source alias during bundling, before browser execution. After resolving the alias, the two-worker run passed 12 cases but hit one Firefox beforeAll timeout, leaving five unrun. The final original run used one worker and passed all 18 without changing the original 10-second timeout, assertions or retries=0. The timeout's exact cause is not proven. The Rust run passed all 18 on its first launch with retries=0. Raw logs and JSON for the failed original matrices are retained. Their temporary error-context files were not retained by the first transport cleanup. No failed or unrun attempt is relabeled as passing.

Final native Windows workspace all-target/all-feature tests passed 2,200 tests across 505 summaries, plus two doctests across five summaries. Strict workspace native Clippy, WASM workspace all-feature library check and strict Clippy passed. All test summaries report zero failed, ignored, measured or filtered tests. This last compiler allocation measured 310 seconds, exceeding the initial under-120-second estimate. Seven reversible generated-report deltas cover the two native gate runs and the temporary build pointer. The original report bytes were restored each time.

Redundant log EOF newlines may be stored separately in raw-log-endings.json to keep diff whitespace checks clean. Concatenating each retained log with its recorded base64 suffix reconstructs the exact raw command bytes and raw hash. Runtime artifact and executable pins are live through independent review. After owned cleanup, they describe historical execution artifacts, not promised files on disk. No native Linux/macOS, full original suite, CI, release, performance or overall-equivalence pass is claimed.

## Commands

Run from an isolated worktree with bounded compiler/browser grants, Cargo -j2 and owned temporary paths on O:.

```sh
node scripts/dsr-option-oracle.mjs
cargo test --locked -p xterm-core --test dsr_option -j2
node scripts/dsr-browser-original.mjs
node scripts/build-demo.mjs --debug
node scripts/e2e-upstream.mjs --suite core --grep 'DSR: Device Status Report' --no-build --timeout-ms 180000
```

Source dependencies are required only for the original oracle and browser suites. Set XTERM_SOURCE_ROOT for the source-only oracle if the supplied read-only reference is outside this worktree. Preserve original assertions and reference hashes. Full equivalence remains NOT PROVEN.
