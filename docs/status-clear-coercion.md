# Program status clear argument compatibility

Overall utility/status equivalence remains NOT PROVEN. Task340 fixes one browser API boundary. Task323 retains complete family closure, including the unchanged addon suites and demo interactions. Fit's Windows failures remain source-identical as recorded in `docs/fit-browser-geometry.md`.

## Defect and change

The previous `ProgramStatusAddon.clear` binding accepted `Option<String>`. It treated `null` as the omitted argument and cleared every record. Boxed strings could select an exact id incorrectly. Other raw values could throw binding errors before source coercion ran.

The original API defaults only an omitted/undefined argument to `''`. `ProgramStatusStore.clear` then evaluates `id + '/'`, even for an empty store, before selecting records. It uses strict equality for the exact id and string prefix matching for descendants. Thus `clear(null)` can clear `null/x` but not `null`, while `clear(new String('a'))` clears `a/b` but keeps `a`.

The binding now accepts a raw JS value. A one-expression JS operator adapter evaluates addition with its default ToPrimitive hint and preserves original exceptions. Rust's native store receives the optional strict string id and the separately evaluated prefix, removes matching records and emits one snapshot only when records changed. No terminal engine runs in JS. Coercion runs before any native store borrow, allowing recursive clear and disposal safely. Protocol ids are restricted to ASCII, so UTF-16 replacement in the binding cannot create an accidental match.

`ProgramStatusAddon::clear(&str)` remains unchanged for native callers. The new `clear_with_prefix(Option<&str>, &str)` documents the boundary representation. Ordinary and foreign clears share mutation/event dispatch. No dependencies were added.

## Tests and evidence

`crates/xterm-web/tests/fixtures/status-clear.js` defines17 operation traces shared by the source oracle and Rust browser test. They cover omitted/undefined/empty/string, null, number, BigInt, boolean, boxed string, array, custom ToPrimitive, thrown-value identity, Symbol, invalid primitive conversion, recursive clear, disposal during coercion and coercion of an empty store. Each row compares remaining ids, event count, snapshot identity, conversion hint/call log and error identity/type.

`scripts/status-clear-oracle.mjs` bundles the unchanged original addon/store/Event/Lifecycle using the existing reference esbuild dependency. It executes the actual original `ProgramStatusAddon.clear` method and store with a labelled parser-registration fixture. It verifies the complete776-entry source manifest before and after, records source hashes and removes its scratch bundle in `finally`. This source run is not a real original-terminal browser suite.

`wasm_status_clear` replays the same17 traces against actual Rust Terminal instances through `open`, `_loadAddon`, queued OSC7501 writes and public browser methods. Each browser has one Rust test function containing17 cases, not17 test functions. The generated `status-clear.json` expectations are never hand-edited.

Evidence under `results/status-clear/` includes:

- `source.json`:17 discovered/executed/passed, zero failed/skipped/unrun.
- `before.json`: baseline Rust test fails in Chromium, Firefox and WebKit. `before-diagnostic.json` retains the exact Chromium expected/actual rows, including null clearing all records and boxed string exact-id removal.
- `after.json`: final-source focused test passes in all three engines,51 matching operation traces total. Browser versions and source/artifact hashes are retained.
- `wasm_status_addons.json` and `wasm_status_coercion.json`:19 and7 test functions pass per engine. Together with the focused clear test, all81 browser test executions pass. The original rerun found9 preexisting Progress assertion ports failing at JS compilation, not terminal execution. Their ordinary Rust strings consumed the JS ST escape, leaving a backslash before the closing quote. Task350 changes only those9 literals to raw strings, preserving all sequence text and expected assertions. `wasm_status_addons-failed.json` retains the10-pass/9-fail baseline per browser; it is not hidden by the later pass.
- `native.log`:51 passing test functions across program_status_addon12, program_status_store12, progress_upstream12, status_addons_differential3 and status_live_terminal12. The differential tests replay the retained2215 parser and2048 progress operations. A new native assertion checks foreign-prefix exact-id exclusion, single-event dispatch and no-op snapshot identity.
- `clippy-native.log`: warning-denied xterm-addons all-target/all-feature check passed. `clippy-wasm.log`: warning-denied WASM status suites check passed.
- `fmt.log`: the initial long-path full fmt hit Windows OS206. `fmt-short-path.log`: the exact `cargo fmt --all -- --check` succeeded after moving only this owned regular worktree from `.../tmp/worktrees/Angela16` to `.../tmp/A`. No junction or suppression was used.

Browser reports identify generated artifacts by digest. Disposable WASM/JS outputs are removed during owned-worktree cleanup; they are not distribution artifacts or permanent download links.

## Reproduce

Use a current Node24 installation, the locked Rust toolchain, wasm32 target, wasm-bindgen0.2.100 and installed Playwright engines. `SOURCE_ROOT` can name the supplied reference checkout with existing esbuild dependencies. `PLAYWRIGHT_MODULE` and `WASM_BINDGEN` can name existing installations without copying dependencies.

```sh
node scripts/status-clear-oracle.mjs
WASM_TEST=wasm_status_clear node scripts/browser-host-check.mjs
cargo test -j2 -p xterm-addons --test program_status_addon --test program_status_store --test status_addons_differential --test progress_upstream --test status_live_terminal
cargo clippy -j2 -p xterm-addons --all-targets --all-features -- -D warnings
cargo clippy -j2 -p xterm-web --target wasm32-unknown-unknown --test wasm_status_clear --test wasm_status_addons --test wasm_status_coercion -- -D warnings
cargo fmt --all -- --check
```

Use `node scripts/status-clear-oracle.mjs --generate` only after reviewing an actual source change. Task340's isolated reviewer independently replayed all17 source traces in memory, rechecked all776 source manifest entries and verified fixture/report hashes and changed-file formatting. That review inspected native and browser receipts but did not rerun heavy commands because no slot was allocated. The final-source browser refresh above ran after acceptance.

The focused results do not establish full workspace/all-platform quality, complete status addon equivalence, or complete browser/demo acceptance. Those gates remain separate tasks.
