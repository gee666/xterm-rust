# Shared Buffer callback checkpoints

Task 122 implements callback-safe resize/reflow. Its latest review fix also preserves sparse original rows through smaller-layout application, so an undefined row fails at the source cell-access phase rather than an earlier invented validation. Base task 27 and its 61-source-case, 1,111-transition evidence remain separate historical observations. New code does not inherit a passing runtime claim from that baseline.

## Implementation

`Buffer::resize` and `Buffer::resize_shared` now execute one algorithm through `ResizeTarget`. Owned callers retain Rust's normal exclusive borrow. Terminal services use the shared method, which ends each short state borrow before an observer can run.

- Capacity trimming runs on a shared CircularList handle outside the root Buffer borrow. Viewport adjustment reads the actual state again after the callback returns.
- Widening helper calls capture a list independently. After deletion callbacks, layout application retrieves the current list again, including a replacement installed by the callback.
- Smaller reflow prepares cells, row references and cursor adjustments without emitting observers. Its layout plan then applies rows and emits insert/trim events outside the Buffer borrow. Every event retrieves the current line list.
- Raw scrollback conversion and Windows PTY property access run outside root state borrows. Windows build truthiness, repeated getters, strict backend comparison and getter errors follow the source call order.
- Cleanup scheduling uses a shared emitter after releasing state access. `BufferCleanup::with_queue` allows an actual host-configured dispatcher rather than an unconnected scheduler token. Injected queues must be dedicated and empty.
- Root Buffer disposal sets its idempotence flag before callbacks. Repeating disposal after a late marker registration does not dispose that late marker, matching the observed source behavior.

Task 130 owns corresponding service-scroll and BufferSet marker/disposal checkpoints. Task 99 owns actual terminal/parser/addon and real-host integration.

## Directed source comparison

`scripts/buffer-resize-reentry-oracle.mjs` uses the same read-only esbuild source-bundle method as the existing buffer baseline. It defines eleven deterministic scenarios:

1. A trim callback reads full state and changes cursor/base/display/saved position.
2. A trim callback performs a nested resize.
3. A widening deletion callback reads and modifies the buffer.
4. A widening deletion callback performs a nested resize.
5. A smaller-layout insertion callback modifies cursor state and row text.
6. A smaller-layout insertion callback performs a nested resize.
7. Scrollback object conversion reads and modifies the root buffer.
8. Windows PTY getters read and modify root state in source order.
9. A trim observer throws, retaining the partial state and actual error object.
10. A widening observer clears/replaces the complete line list. Undefined source slots must remain holes rather than be rejected by an invented layout validation rule.
11. After a widening callback clears/refills the list, a subsequent smaller resize preserves sparse original slots, applies all layout/events and resizes the nine present rows before failing on the final undefined row. Cursor/base/display, cached dimensions and logical length match the retained source partial-state counterexample.

Each snapshot records dimensions, cursor, base/display, saved positions, scroll region, capacity, full packed cells, extended attributes, UTF-16 text, row identities and undefined slots.

## Execution state

The source generator executed all eleven scenarios successfully and saved their source hashes and expected traces to `results/buffer-reentry/oracle.json` and `tests/fixtures/buffer-resize-reentry.tsv`. The mirrored Rust test module contains eleven passing cases, including nested resizes, thrown callback partial state and replacement-list holes. Tests and their shared snapshot helper are installed under `crates/xterm-core/tests`.

A granted targeted `cargo check -p xterm-core` completed with exit 0 and no warnings after the shared refactor. The log is `tmp/buffer-wave2-core-check.log`. This is compilation evidence only, not a callback-behavior pass. No new release, WASM, browser, Linux or macOS runtime claim is made.

The latest focused native Windows batch passed all 34 tests: eleven directed reentry cases, the ordinary 1,111-transition corpus executed through both owned/shared methods for 2,222 transitions, eleven host/marker contracts, five raw Windows PTY cases and six injected-queue/disposal/error cases. The first 30-case log remains `results/buffer-reentry/native-focused.log`. The current 34-case receipt is `results/build-slots/1793172557926-41596-004d3796-d9f1-4044-8a12-0d4a81d45dc5/receipt.json`. Current scoped Clippy reports only unfinished task-179 owner-slot consumer wiring. Independent re-review is required before task 122 can be accepted. No release, browser or other native-platform pass is inferred. Full xterm.js equivalence remains NOT PROVEN.
