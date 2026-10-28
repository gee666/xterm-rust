---
name: csi-erase-check
description: Use when checking ED, EL, ECH, DECSCA, selective erase, BCE, erase marker ordering or the private ED3 browser scroll notification.
---

# CSI erase checks

Run from repository root with the unchanged source reference and its npm development dependencies installed. Reuse the native-quality resource guard and the shared Cargo target. Do not build during a coordinated disk pause or override `.cargo/config.toml`.

```sh
node scripts/csi-erase-mapping.mjs
node scripts/csi-erase-oracle.mjs
node scripts/csi-erase-fixtures.mjs
cargo test -p xterm-core --test csi_erase_upstream --test csi_erase_cases --test csi_erase_playwright_assertions --test csi_erase_edges --test csi_erase_differential --test csi_erase_vt
cargo clippy -p xterm-core --test csi_erase_upstream --test csi_erase_cases --test csi_erase_playwright_assertions --test csi_erase_edges --test csi_erase_differential --test csi_erase_vt -- -D warnings
```

Use `--generate` on either fixture generator only after an intentional reviewed source-reference change or for initial fixture capture. Never hand-edit expected trace values to match Rust.

The full-state oracle uses the actual unchanged headless terminal, not mocked methods. Seed `0xe2a5e111` produces 337 traces, 3,937 snapshots, 2,140 writes and 424 marker callbacks. It shares complete buffer/cell/attribute/event projections with the whole-terminal oracle. The source mapping inventories 22 common and six browser cases. A negative private-coordinate ECH half remains task 152 until a faithful numeric representation passes it. Native browser-content assertion ports are not actual browser execution.

The five VT inputs/expected outputs are upstream fixtures. The generator models PTY ONLCR explicitly, records the original Windows PTY skip and EL compatibility skip, and still compares Rust with real JS output. It must not describe that replay as an observed native PTY run. Source copies remain byte-for-byte intact.

`results/csi-erase/counterexamples.json` retains the ED3 event bug and a driver mistake. ED3 must emit private `InputScroll`, not public `Scroll`. Browser selection refresh and queued viewport synchronization require actual Rust-backed browser evidence. Buffer observer access alone does not prove synchronous terminal-level reentry during marker disposal. See tasks 176 and criterion 403.

Run the unchanged baseline with `node source_repo/bin/test_unit.js out-esbuild/common/InputHandler.test.js --reporter=json`. It has 194 source tests. Raw output can contain terminal reports before the JSON object; preserve those bytes in raw evidence and parse only the reporter object for counts. See `docs/csi-erase-status.md` for observed versus unrun checks and `docs/acceptance/csi-erase.md` for the exhaustive checklist.
