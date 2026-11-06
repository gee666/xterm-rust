---
name: marker-event-property-check
description: Use when changing exported Marker.onDispose property descriptors, event replacement or native marker disposal bindings.
---

# Marker event property checks

Reuse `scripts/terminal-marker-oracle.mjs` and the Rust `wasm_terminal_markers` browser target. The shared `markerEventPropertyCase` checks the source own writable/enumerable/configurable data descriptor, raw value identity, direct replacement calls, deletion without a prototype fallback, throwing getter replacement, retained event functions, trim, resource ordering and cached handle identity.

Obtain a compiler/browser allocation from the task-309 resource owner first. Keep Cargo at `-j2`, four heavy jobs team-wide and two browser matrices maximum. Use an owned O: worktree and regular target. Set TEMP first and TMP/TMPDIR separately to the owned worktree's tmp directory.

```sh
REFERENCE_ROOT=/absolute/unchanged/source_repo node scripts/terminal-marker-oracle.mjs
WASM_TEST=wasm_terminal_markers node scripts/browser-host-check.mjs
cargo clippy -p xterm-web --target wasm32-unknown-unknown --test wasm_terminal_markers --all-features -j2 -- -D warnings
cargo fmt -p xterm-web -- --check
```

Set `PLAYWRIGHT_MODULE` and `WASM_BINDGEN` to existing matching tools where necessary. Do not reinstall dependencies per task. The source runner reads its reference dependencies and writes its bundle only to owned tmp storage. The browser driver retains actual compiler/served-WASM/binding identities and closes its browsers/server in finally blocks.

Run the same tests on exact old `marker_api.rs` as a negative control, retain the expected descriptor failure, and restore exact production bytes in finally before a final matrix. Keep original reference manifest checks before/after. This is scoped marker event evidence, not full task331 raw marker arrays/fields, decoration or GC equivalence.
