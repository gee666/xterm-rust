# Horizontal tab option callbacks

Task491 repairs HT execution against the unchanged `InputHandler.tab` in `source_repo/src/common/InputHandler.ts`, lines850-859. Full parser, browser and workspace equivalence remains NOT PROVEN.

## Contract and implementation

HT returns without reading `screenReaderMode` when the cursor is at or beyond the right edge, including pending wrap. Otherwise it records the original column, advances to the next tab stop, and then reads the raw option. That getter may synchronously reset, resize, switch the active buffer, change the cursor, or throw. When truthy, the accessibility event contains the current active-buffer column minus the original column, not a distance saved before the getter.

`terminal/controls.rs::tab_state` separates these phases and releases all terminal and buffer borrows before the getter and event callbacks. `register.rs` uses this shared-state path for HT. The old borrowed implementation is removed. Getter and accessibility callback failures propagate without replacing the original callback error or undoing the partial state.

`TerminalEvent::AccessibilityTab` and browser `AccessibilityEvent::Tab` now carry `i64`, replacing `usize`. Source callbacks can move backward and produce negative counts. This is an intentional public Rust representation change. The browser accessibility loop announces no spaces for nonpositive counts, matching `AccessibilityManager._handleTab`. Checked conversions reject columns outside signed integer range instead of wrapping. Ordinary terminal dimensions remain bounded by the existing API validation.

Accepted484 is an explicit private dependency for its source CoreTerminal scroll dirty hook, needed when a getter activates another buffer. Its source2f673724 and receiptfe62f169 are preserved as private cherry-picks7299e6cb and07b3c1a8 on main735e114a. They are not part of the owned491 producer change and must not be picked twice by integration. Existing475 BS/RI restrictions remain intact.

## Reproduction and evidence

From the task branch, with the existing source dependencies and Rust toolchain:

```sh
XTERM_SOURCE_ROOT=/path/to/source_repo node scripts/tab-option-oracle.mjs
cargo test --locked -j2 -p xterm-core --test tab_option
cargo test --locked -j2 -p xterm-core --all-targets --all-features
cargo test --locked -j2 -p xterm-core --doc --all-features
cargo clippy --locked -j2 -p xterm-core --all-targets --all-features -- -D warnings
cargo check --locked -j2 --workspace --all-features --lib
cargo clippy --locked -j2 --workspace --all-features --target wasm32-unknown-unknown --lib -- -D warnings
```

`results/tab-option/` retains exact logs. The untouched-source oracle verifies all776 reference files before and after capture and replay. Its160 cases cover both buffers, option truth values, columns0/3/11/12, getter inspection/reset/buffer activation/resize/cursor mutation/throws and synchronous accessibility callback mutation/throws. Rust compares getter counts, callback snapshots, full final state including both buffers and every cell, ordered events, and error identity.

- Source generation and replay PASS160/160. Native fixed corpus PASS160/160.
- Old main native production fails120 cases. The40 right-edge early-return cases pass, proving the getter must remain skipped there.
- Keeping the unborrowed getter but computing the distance before it returns fails18 cases. Exact source bytes and SHA are restored before the fixed rerun, which passes.
- Native core all-target/all-feature run passes1567 test functions in248 result summaries, with0 failed and0 ignored. Core doctests pass2/2.
- Strict core Clippy, native all-feature workspace library check, strict all-feature WASM workspace library Clippy and accessibility test Clippy PASS.
- Core formatting and changed browser-file formatting PASS after correcting task formatting. The initial combined formatter and required whole-workspace formatter hit Windows OS206. Those logs remain failures, not a claimed workspace pass.

The accessibility component assertion extends the existing pending-output test with negative and zero Tab events while retaining all previous expected output. `WASM_TEST=wasm_accessibility node scripts/browser-host-check.mjs` passes25 functions in each Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. `WASM_TEST=wasm_accessibility_terminal node scripts/browser-host-check.mjs` additionally passes7 real-terminal functions per engine. Each report records0 failed,0 ignored and0 filtered. Full command/build/bindgen/raw browser receipts are under `results/tab-option/browser-component/` and `browser-terminal/`. These are Rust/WASM tests using real DOM and live terminal services, not JavaScript-only passes. The browser reports' `componentOnly` label describes the runner and does not turn these scoped tests into a full original-suite pass.

The final post-format whole-core run again passes1567 functions/248 summaries and strict core Clippy. Browser execution used accepted495's owned-server cleanup helper as private commit e010a881. It is another separate integration prerequisite, not an owned491 production change. Temporary compiler/WASM/JS artifacts are hash-pinned for independent review and then removed during owned-worktree cleanup, while raw receipts remain committed.

No full original suite, native Linux/macOS run, CI, performance comparison or final integrated matrix has run for this task. Full tasks313/319/304/327 remain open.
