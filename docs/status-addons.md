# Program-status and progress port

Implementation is in `crates/xterm-addons/src/program_status/`, `progress.rs`, `status_terminal.rs` and `native_status.rs`. Source attribution is retained. No new production dependency was added. `serde_json` is a test-only dependency for source-generated fixtures.

`ProgramStatusAddon` implements OSC7501 parsing, immutable cached status snapshots, ancestor app lookup, update-order eviction, clear/exit/prompt lifecycle, synchronous feature replies and RIS fallthrough. `ProgressAddon` implements OSC9;4 progress and the numeric setter. Rust `Option` represents omitted record fields. `Rc<[StatusRecord]>` is the immutable snapshot with identity preserved between changes. Events use the core synchronous emitter. Public addon methods return callback failures rather than throwing. Progress setter returns false for an invalid state instead of a console warning; fractional and NaN states remain accepted, as the source comparisons require.

The native adapter connects to the actual `CoreTerminal` parser. Its reply path uses a separately owned `CoreService` to avoid reborrowing the engine during parser dispatch. Original input errors are retained in `take_input_error`, while parser dispatch returns `CallbackFailed`. Addons must retain their adapter. Rust drop unregisters parser handles. Program status disposal silently clears records; progress disposal keeps its last value.

## Reproduction and observed results

Run from repository root with Node24 TypeScript transform and Rust1.85 or newer:

```
node --experimental-transform-types scripts/status-addons-oracle.mjs
cargo test -p xterm-addons --test status_addons_differential --test program_status_store --test program_status_addon --test progress_upstream
cargo test -p xterm-addons --test status_live_terminal
cargo clippy -p xterm-addons --all-targets --all-features -- -D warnings
```

`results/status-addons/oracle.json` records Node v24.15.0, Windows x64, source/test SHA-256 and fixture hash. The unchanged pure upstream ProgramStatus suite executed 38 cases: 38 passed, zero failed/skipped/unrun. Only imports/assertion runner change in a temporary copy under `tmp/status-addons`; source and assertions remain untouched. The suite contains 27 parser and 11 store cases. Rust reproduces all source parser invocations plus boundary/random inputs as 2,215 full-output comparisons. Progress replays 2,048 sequential operations against source. Seed is `0x750109`.

`pure-tests.log` reports 36 Rust tests passed, zero failed/ignored. This is 11 faithful store tests, ten addon registration fixture tests, 12 progress tests including the ten original progress browser assertions run through a fixture, and three differential/fixture tests. These are not claims of browser execution.

`live-tests.log` initially reported eight actual Rust terminal tests, seven passed and one failed. The expanded `tests.log` records ten actual-terminal tests, nine passed and the same DA1 assertion failed. The failure retains the required DA1 ordering assertion: the core currently emits the OSC7501 reply but omits the following DA1 reply. Coleman was notified under core task28. Passing live cases cover every split boundary of a report stream, BEL/ST, cancellation, disableStdin, silent disposal, RIS/core reset versus API reset, soft reset/alternate screen retention, status/progress cooperation and stopping parsing on listener failure. The failing assertion is not weakened.

## Remaining work

Native adapter registration now uses Max/Amy's cloneable weak registrars. The actual terminal reentrant-activation test passes, along with preservation of reply-listener failure causes. `clippy-current.log` records a passing warning-free addon/all-target Clippy command on the current integrated tree; earlier `clippy.log` retains an unrelated shared-tree compile failure. Run formatter and full workspace gates against the final integrated tree. Add the JS/WASM addon API and run the original program-status/progress browser suites against Rust in Chromium, Firefox and WebKit. No browser suite or Linux/macOS runtime result is claimed here. Overall addon equivalence remains NOT PROVEN until these tests and independent review pass. Detailed requirements are in `docs/status-addons-acceptance.md` and criteria35 through37.
