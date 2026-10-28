# Current terminal integration review

Isaac owns task 149. This review inspects the current engine, not just historical commit `4068d85`. A disk reserve failure initially stopped new builds. After Kassa's reviewed cleanup in post 94, bounded native checks resumed with the same shared target directory and a 1 GiB pre/during/post watchdog.

## Inspected code

- Rust `terminal/mod.rs`, `construction.rs`, `state.rs`, `operations.rs`, `dispatch.rs` and `error.rs`.
- Original `common/CoreTerminal.ts`, especially resize at lines 187-200 and reset at lines 270-276.
- Original `common/InputHandler.ts`, especially attribute reset at lines 3434-3437 and cursor save/restore at lines 2995-3029.
- Original `headless/Terminal.ts`, clear at lines 101-111 and public reset at lines 122-132.

## Findings and repairs

1. Disposal retained a TerminalState borrow across service disposal and marker callbacks. Its service order also differed from the source. Coleman now clones service handles in a short borrow, releases it and follows registration order. The marker callback can borrow live state during disposal.
2. Resize retained a mutable TerminalState borrow across resize events. Clear retained both terminal and active-buffer borrows across marker disposal. Coleman moved those calls outside state borrows and used the shared marker helper. Clear reacquires the active buffer after callbacks because the source reads that getter repeatedly.
3. Source reset resets InputHandler attributes before BufferService reset, followed by charset, core and mouse. Rust reset had reset attributes last. The shared reset now follows source order, and activation observers see default attributes without a held state borrow.
4. Resize converted BufferError to a string even though TerminalError has a typed BufferState variant. The repaired path retains the exact callback error Rc instead of replacing its identity with text.
5. An event handler that captured a state handle formed a Rust ownership cycle after terminal drop. Disposal now clears its callback slot and drops the old callback outside all borrows.

## Executed checks

`crates/xterm-core/tests/terminal_review.rs` has six directed regressions for these boundaries. These are source-derived review tests, not unaltered original assertions. All six pass in debug and release. The eight existing directed engine tests and one replay test also pass in both profiles. Regenerating the original JavaScript projection verifies the existing fixture without drift: 172 traces, 1,114 snapshots, seed `0x4354524d`.

Focused core Clippy and scoped formatting pass. Full commands, raw logs, counts and disk observations are in `review.json`, `controlled-native.json`, `controlled-quality.json` and `controlled-lint.json`. The last whole-workspace input hash changed during the scoped lint run. This evidence is therefore a retained checkpoint observation, not an immutable whole-workspace pass.

The projection includes both buffers, every active packed cell and UTF-16/extended word, cursor, viewport, margins, insert mode and the listed private modes. It excludes saved cursor/charset arrays, tabs, markers, mouse state and decoder internals. Do not call this finite projection complete terminal-state equivalence. Parent task 28 remains incomplete until its entire source contract is implemented and demonstrated.

Overall equivalence is NOT PROVEN.
