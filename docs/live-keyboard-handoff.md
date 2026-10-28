# Live keyboard work checkpoint

Victor owns task151. Tasks54 and94 were accepted earlier; those earlier passes do not prove the new generic/live-field refactor. Do not repeat implementation writes from an interrupted session.

## Commits and ownership

- a8bb69d has the independent Kitty oracle, fixtures, reports and skill.
- f1cbddb has typed browser dispatch, source-method traces and docs.
- 3434570 adds core KeyboardEventSource, UTF-16/codepoint/number helpers.
- 1955127 adds generic Kitty/Win32 policies, lazy dispatcher and DOM adapter, protocol getter fixture/replay, initial real DOM tests and retained failures.
- Boleslav's 8d0f6d9 provides shared `crates/xterm-core/tests/support/keyboard_live.rs` and the legacy replay.

After1955127, Victor added a source-correct unchecked Event wrapper for registered cross-realm keyboard/input listeners and a foreign-document regression. There are now12 wasm_keyboard_live assertions, not10. These changes may still be uncommitted. Norman separately changed support/input_dom.rs fine-grained option hooks and input-host lease behavior; preserve his edits.

## Current proofs

- Source-only protocol-live oracle passed4,521 success paths and74,171 exact throw-marker boundaries,78,692 rows. Fixture/provenance report is results/protocol-live/oracle.json. Core Rust replay passed all rows plus7 Kitty fixture/edge tests at results/build-slots/1793149821671-29876-53e209ee-9d6d-41cc-9779-95a7a44427c7.
- Protocol replay then adopted Boleslav's shared Source helper. That test-only change still needs rerun with the legacy replay.
- Native dispatcher14 tests passed at results/build-slots/1793150026739-39584-84b3bc72-e5a8-4219-86d2-7cb8fc66bda6.
- Win32 original64 tests and13,904 ordinary differential cases passed at results/build-slots/1793149970440-34500-98cee0ae-3920-40c5-8da9-a28517b7a4f8.
- Kitty508,249 rerun failed on an in-progress buffer export, not a protocol mismatch. It must be rerun.
- Initial wasm_keyboard_live build failed on renderer/service/utility integration errors. Owners reported fixes, but real browsers remain unrun. Native Clippy was denied before starting by the1GiB resource reserve. Own rustfmt passes.

## Next validation

Use task190's shared `scripts/build-slot.mjs`, not direct Cargo or an alternate target directory. Set PATH for local Rust tools if missing. CLI accepts owner/lane/wait-ms/timeout-ms and `-- COMMAND ARGS`. Wait for resource headroom rather than retry a denied lease.

1. Native core `cargo test -p xterm-core --test protocol_live --test keyboard_live --test kitty_upstream --test kitty_edges`, confirming the actual legacy test target name first.
2. `node scripts/verify-kitty-keyboard.mjs`, then `node scripts/keyboard-dispatch-oracle.mjs`; retain current production hashes and counters.
3. Scoped warning-denied core/web Clippy including new tests and the WASM target. Fix actual causes, not warning suppression.
4. With WASM_TEST=wasm_keyboard_live, run `node scripts/browser-host-check.mjs` through the wasm lane. It executes all installed Chromium/Firefox/WebKit tests with actual versions and hashes. Runtime outcomes must be inspected, not inferred from compile.
5. Rerun shared IME/callback tests after Norman's phased-composition changes and actual engine task141 wiring.

## Resource state

At last check P: had under1GiB free and C: had102GiB. The native/WASM slot manager correctly denied new builds. Kassa/Chloe were notified; Victor suggested coordinated relocation of the SINGLE existing1.5GiB Cargo cache, not creating duplicate caches or hardcoding platform paths. No relocation was performed by Victor. Source-only Node fixture generation below20MiB was explicitly approved and completed.

## Remaining separate work

177 is lazy InputEvent/nullable payloads.184/185 are Win32 inherited raw lookup/coercion and independent evidence.202 is decorated fields outside declared DOM primitive types.207 is per-realm Kitty map values/coercion. Contracts are docs/keyboard-map-lookup.md and docs/live-keyboard-event.md. Do not hide those gaps behind151 or call the browser port complete. No full equivalence claim, Linux/macOS runtime claim or original complete browser-suite pass is warranted.
