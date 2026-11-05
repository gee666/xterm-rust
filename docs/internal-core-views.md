# Native-backed internal core and buffer views

Task380 repairs two missing read paths used by the unchanged browser InputHandler suite:

- `terminal._core.coreService.decPrivateModes` for DECSCUSR cursor style and blink.
- `terminal._core._bufferService.buffer.scrollTop` and `scrollBottom` for DECSTBM scrolling margins.

These are real views of the Rust services used by parsing, not test-page fixtures or another mode/buffer authority. `CoreServices` now receives the native terminal, clones its existing Unicode/mouse/core/buffer services while borrowing state, then constructs the wrappers after releasing that borrow. Leif's375 mouse wrapper remains intact. The constructor remains under350 lines.

## Read-only contract

The DEC view exposes all12 native private-mode fields using their source names. Source `undefined` cursorStyle/cursorBlink values remain JavaScript `undefined`, not null. Reads observe in-place parser changes and return a stable object until reset. Reset replaces the source mode object. A retained old view then reports its pre-reset state, while a subsequent `decPrivateModes` access returns a new object with current defaults.

The native `OwnedPrivateModesCapture` shares the existing borrowed capture's freeze-on-reset mechanism. It holds an owned service reference rather than a callback-spanning borrow. The service retains only weak capture states, so this introduces no reference cycle. Retaining a mode wrapper retains the native CoreService; precise JavaScript GC/descriptor equivalence remains outside this scoped contract.

The buffer service exposes read-only `cols`, `rows` and `buffer`. Each native normal/alternate buffer has a stable wrapper. The active wrapper changes on buffer switching, returns to the previous normal wrapper on normal activation, and changes when reset replaces the native buffer. Retained wrappers continue to read their original buffer. Scalar fields are `x`, `y`, `ybase`, `ydisp`, `scrollTop` and `scrollBottom`. Buffer borrows end within each getter, and borrow failures propagate as JavaScript errors rather than panics.

This does not implement source-writable internal objects, property descriptors/enumeration, every CoreService/BufferService method, or the full private ABI. Other `_core` members remain absent rather than successful no-ops. Full public/options parity stays in315, and raw mutation/GC work remains explicit there.

## Directed comparison and unchanged tests

`scripts/core-view-trace.js` is the shared directed program. It runs through both the original production Terminal and exported Rust Terminal. Its16 snapshots cover defaults, all seven DECSCUSR values, other parser mode changes, scrolling margins, cursor movement, normal/alternate activation, resize and reset. Each snapshot checks values plus retained/current object identities. The generated JSON fixture exceeds350 lines because it records those complete independent source snapshots, not production code.

`scripts/core-views-source.mjs` checks the original bundle's embedded CoreService/BufferService/BufferSet sources against the read-only TypeScript files before running. All16 snapshots matched across Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4 on Windows. The Rust WASM test compares its entire trace to that fixture, with no fixture services or JS terminal-engine fallback.

Before binding the new views, the same Rust test failed in Chromium because coreService was absent. The retained baseline is `archive/Jeffrey17-core-views-before` at921d02f, actual mainff0bec8 plus the isolated375 mouse producer and the identical380 test files. Baseline Firefox/WebKit execution was not attempted. After the fix, the test passed1/1 per engine, covering48 Rust snapshots. The unchanged original DECSCUSR and DECSTBM Playwright assertions passed6/6 expanded cases across the three engines, with zero skips or unrun cases. The existing375 mouse suite passed5/5 per engine as a composition regression.

Native core+web all-target/all-feature tests passed1876 test functions across359 summaries. That includes the new owned-capture test and all three prior borrowed-capture tests. Native warning-denied Clippy for both crates, scoped WASM library/two-suite Clippy, and full workspace rustfmt passed. These are Windows results, not a workspace release gate or native Linux/macOS proof.

The initial combined cold-build command hit its180-second shell deadline during native Clippy. No owned child process remained. The native tests and Clippy were rerun in separately bounded commands and passed; partial and final logs are retained distinctly.

## Reproduction

Use Rust/Cargo1.85.1, Node24.15.0 and the locked wasm-bindgen0.2.100 CLI. Set `SOURCE_ROOT` to the read-only built original reference, and `PLAYWRIGHT_MODULE`/`NODE_PATH` to its existing dependencies. The unchanged-suite runner uses the physical source copy and its dependency junction; unlink that owned junction before whole-worktree cleanup. Put Cargo target/TEMP/TMP/TMPDIR on owned O: paths and coordinate compiler/browser grants.

```sh
node scripts/core-views-source.mjs
# Explicit generation is only needed to reproduce the source fixture:
node scripts/core-views-source.mjs --generate
WASM_TEST=wasm_core_views node scripts/browser-host-check.mjs
WASM_TEST=wasm_terminal_mouse_modes node scripts/browser-host-check.mjs
node scripts/e2e-upstream.mjs --suite core --grep 'DECSCUSR|DECSTBM' --timeout-ms 120000
cargo test --locked -j2 -p xterm-core -p xterm-web --all-targets --all-features
cargo clippy --locked -j2 -p xterm-core -p xterm-web --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_core_views --test wasm_terminal_mouse_modes -- -D warnings
cargo fmt --all -- --check
```

Reports, exact commands, source/binary hashes, the failed baseline and concise logs are retained in `results/core-views/`. Disposable build/browser binaries are removed after integration or handoff. These scoped passes do not establish full xterm.js equivalence.
