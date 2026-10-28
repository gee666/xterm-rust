# Current terminal integration review

Isaac owns task149. This review inspects current source rather than treating historical4068d85 results as proof for a later tree. Native reruns are paused because the shared drive fell below the1GiB resource preflight threshold. No new native pass is claimed here.

## Inspected code

- `terminal/mod.rs`, `construction.rs`, `state.rs`, `operations.rs`, `dispatch.rs` and `error.rs`.
- Original `common/CoreTerminal.ts`, especially resize187-200 and reset270-276.
- Original `common/InputHandler.ts`, especially attribute reset3434-3437 and cursor save/restore2995-3029.
- Original `headless/Terminal.ts`, clear101-111 and public reset122-132.

## Findings under repair

1. CoreTerminal disposal retained a TerminalState Ref across service disposal and marker callbacks. Coleman confirmed the source ordering also differed. He cloned service handles in a short borrow and changed disposal order to match registration. The latest code still needs native reentry/error tests.
2. Resize held TerminalState RefMut across BufferService resize events. Clear held both terminal and active-buffer borrows across marker disposal. Those callbacks must be able to observe live state without a borrow failure. Coleman is changing these calls to release state and use the shared marker helper, reacquiring the active buffer after callbacks because the source reads that getter repeatedly.
3. Source reset resets InputHandler attributes before BufferService reset, then charset, core and mouse. The inspected Rust reset reset attributes last. Buffer activation observers therefore saw stale attributes or could not borrow state at all. A focused regression now requires default attributes at the activation boundary.
4. Resize and several related paths converted BufferError into a string despite TerminalError already having a typed BufferState variant. This loses the original callback error object. A regression now checks the exact Rc carried by EmitError::Callback after resize.

5. The terminal event handler slot was not cleared on disposal. A normal handler that captures a state handle forms a Rust ownership cycle after terminal drop. A sixth regression checks weak-state release after explicit disposal; callback cleanup must drop the old handler outside all borrows.

`crates/xterm-core/tests/terminal_review.rs` contains six directed regressions for these boundaries. They are source-derived review tests, not unaltered original assertions. They remain unrun until disk headroom is restored. The existing directed and1114 state-projection trace checks must also be rerun on the current tree. Inspection of terminal-snapshot.mjs confirms both buffers, every active cell/UTF16/extended word, cursor, viewport, margins, insert mode and the listed private modes. It does not project saved cursor/charset arrays, tabs, markers, mouse state or decoder internals. Do not label this finite projection complete terminal-state equivalence. Parent task28 remains incomplete until its entire source contract is implemented and demonstrated.

Overall equivalence is NOT PROVEN.
