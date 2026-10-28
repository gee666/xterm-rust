# Program-status and progress port

Native implementations are in `crates/xterm-addons/src/program_status/`, `progress.rs`, `native_status.rs` and `managed_status.rs`. WASM bindings are in `crates/xterm-web/src/status_addons/`. They use the actual Rust terminal parser and public loadAddon/write APIs, not the JavaScript terminal engine.

Rust uses enums and Option for status fields, immutable shared snapshots and identity tokens for unchanged records. Browser records are frozen own-property objects. Progress retains JS property-read order, coercion, raw state identity, partial assignment on errors and reentrant setter behavior. Event payloads share one object per dispatch. Native setters return Result rather than throwing; invalid numeric states return false instead of a console warning. Browser setters retain the source warning behavior.

## Verification

Run from the isolated worktree:

```
cargo test -p xterm-addons
cargo clippy -p xterm-addons --all-targets --all-features -- -D warnings
WASM_TEST=wasm_status_addons node scripts/browser-host-check.mjs
WASM_TEST=wasm_status_coercion node scripts/browser-host-check.mjs
```

Observed results:

- Native addon crate: 146 passed, 0 failed/ignored. The focused status/progress targets contain 50 passing tests. Addon Clippy passed.
- Original pure ProgramStatus suite: 38 passed. Source oracle compared 2,215 parser cases and 2,048 progress operations with seed `0x750109`, no mismatches. Reproduce with `node --experimental-transform-types scripts/status-addons-oracle.mjs`.
- Actual browser binding suite: 21 passed per browser, including all four original program-status and ten original progress browser cases.
- JS coercion/identity suite: 9 passed per browser. Task 164 was independently reviewed and accepted after fixing inherited-property construction.
- Browsers: Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4. Results are in `results/browser-host/wasm_status_addons-results.json` and `wasm_status_coercion-results.json`.

Earlier failed logs retain the missing DA1 dependency and test-source escaping defects; both are fixed, with assertions unchanged. Scope and API mapping are in `docs/status-addons-acceptance.md` and `docs/status-addons-api.json`.

This is addon evidence, not whole-terminal equivalence. Workspace quality gates and final native CI remain separate. CI run https://github.com/gee666/xterm-rust/actions/runs/40659278655 failed on the earlier dependency baseline; no native Linux/macOS pass is claimed from that run.
