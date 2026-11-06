# Attach disposal reentry

Task535 fixes the live iterator in `crates/xterm-addons/src/attach.rs`. Full utility/status family323 and full-port equivalence remain NOT PROVEN.

Upstream AttachAddon.dispose uses `for (const d of this._disposables)`. A socket's removeEventListener can synchronously call activate again, appending handles to that same array. The active disposal must visit those new handles too. Rust previously cloned a snapshot, leaving appended socket and terminal subscriptions active. The repair reads one owned handle at a time from the live vector and releases its borrow before invoking disposal. No new dependency, public API or unsafe code is needed.

The same code runs for explicit dispose and the addon's close/error callbacks. The shared fixture has 54 scenarios covering those three entry points, appending during message/close/error removal, zero/one/three reactivations, and both bidirectional settings. It records socket registration/removal order, remaining reactivations, surviving listeners, terminal writes and socket sends after disposal. The native function expands the same 54 combinations over actual portable AttachAddon adapters. The WASM function uses actual Rust Terminal and AttachAddon exports, not a JS terminal engine. A recording socket supplies deterministic synchronous reentry; it is not a real network transport test.

## Reproduce

Use Node24.15.0, Rust/Cargo1.85.1 and wasm-bindgen CLI0.2.100, plus installed Playwright engines. Use an owned O: worktree, regular private target and O: TEMP/TMP. Obtain a current bounded compiler/browser grant before builds.

```sh
node scripts/attach-disposal-reentry-oracle.mjs
cargo test --locked -j2 -p xterm-addons --test attach_disposal_reentry --test utility_attach
cargo clippy --locked -j2 -p xterm-addons --all-targets --all-features -- -D warnings
WASM_TEST=wasm_attach_disposal_reentry node scripts/browser-host-check.mjs
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_attach_disposal_reentry -- -D warnings
cargo fmt --all -- --check
```

The source oracle strips TypeScript syntax only and runs unchanged original addon methods. It verifies all776 reference hashes before and after execution. The expected trace JSON is source-generated; do not regenerate it to fit Rust. The original assertions and source_repo files are unchanged.

## Remaining contracts

Task536 explicitly retains the existing BrowserSocket one-shot removal and panic-on-remove-error behavior. Upstream repeatedly calls the current removeEventListener and propagates raw exceptions. This repair does not change `socket.rs` or claim that broader removal/lifetime contract is solved.

The inspection also found that ProgressAddon binding hardcodes clamping after unary numeric conversion, whereas source resolves Math.min and Math.max before the value getter and can retain arbitrary replacement-method returns. That broader raw-host contract remains full323. No partial Progress repair is included.

Full original addon suites, native Linux/macOS, complete browser/demo behavior, final coverage/comparison deliverables and stable whole-workspace quality gates remain outside this scoped repair. Function counts and expanded scenario counts are different measures and must not be added together.
