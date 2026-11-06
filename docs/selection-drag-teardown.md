# Selection drag teardown

Task564 fixes the DOM boundary used by `select`, `clearSelection` and mouseup. It does not complete task321 or establish full selection/link parity.

## Source contract and repair

`SelectionService._removeMouseDownListeners` removes the persistent document `mousemove` callback, then `mouseup`, then calls `window.clearInterval` with the current timer slot. Only after that call succeeds does it set the slot to `undefined`. It repeats both removals even if a previous call already removed the listeners. A synchronous exception stops at its original phase. A nested selection observes and mutates the same timer slot.

Rust previously canceled and cleared the timer first. It then took disposable listener tokens out of a vector. Successful removal lost their identity for subsequent calls. A failed first removal dropped the second token and removed its listener during error unwinding, despite the source stopping immediately. Recursive selection could not reproduce the source teardown trace.

`selection/dom.rs` now constructs two persistent JS-owned `Fn` callbacks through `Rc::new_cyclic`. Each captures only `Weak<State>`, so there is no strong ownership cycle. `selection/dom_schedule.rs` registers and removes those same functions. It clones their handles before calling the host, without a callback-spanning `RefCell` borrow. Removal resolves the screen's current owner document, as the source does. The timer is sampled after both removals, passed as its real handle or `undefined`, and cleared only after the host returns successfully.

A small private catch-enabled wasm-bindgen method import preserves `clearInterval` exceptions. The web-sys import returns `()` and cannot carry a patched host exception back through the Rust selection event error boundary. This import invokes the real window method with the real receiver. There is no replacement selection service, JS terminal delegation, new dependency or unsafe code.

## Verification

- `scripts/selection-drag-teardown-trace.js` is the same program for the original Terminal and actual Rust/WASM Terminal. Nine cases cover plain, throw-once and nested-selection behavior at each of `mousemove` removal, `mouseup` removal and `clearInterval`.
- Every case starts a real DOM drag. It checks persistent callback identity, call order, timer argument, text and endpoint snapshots during callbacks, exact thrown object or Symbol identity, partial state, retry behavior and identical repeated selection event deduplication.
- The source runner checks embedded source-map contents and source hashes before and after execution. Nine cases passed in each of Chromium, Firefox and WebKit. These are custom differential cases, not upstream test definitions.
- Before the production patch, the identical Rust test and fixture failed in Chromium with zero page errors. Its first normal trace canceled the drag timer before the two removals; retries made no removal calls. The complete assertion failure is retained in `rust-old.json`.
- Final Rust/WASM has one test function expanding the nine cases, passing in all three engines. Retained `wasm_selection_teardown` has one function with eighteen cases; `wasm_selection_live_geometry` has one with eight; `wasm_selection` has eight functions; `wasm_terminal_selection` has one. Each retained suite passed in all three engines.
- Native `cargo test --locked -j2 -p xterm-web --all-targets --all-features` passed 432 test functions across 185 test executables, including native zero-test WASM targets and examples. Counts are not 185 substantive suites or browser passes.
- Strict native web all-target/all-feature Clippy and WASM web library plus the five relevant browser test targets passed. The web member formatting check passed. Exact workspace `cargo fmt --all -- --check` failed with Windows OS206. No workspace formatting pass is claimed.

Commands, versions, hashes, compiler/served-WASM/binding identities, raw source/browser traces and gate logs are under `results/selection-drag-teardown`. The old and final compiler, served WASM and binding hashes differ, while the shared program, expected fixture and Rust test remain identical. Runtime files are temporary and their pins become historical after owned cleanup.

## Remaining gaps

The pre-existing `drag` path still stops an earlier drag before registering the new one. Source repeated-mousedown behavior can leave an older interval alive after overwriting its timer slot. This task does not introduce that leak into Rust or claim its lifetime semantics are equivalent. Callback disposal after terminal destruction, the full timer/RAF lifetime model, raw options/numeric cases, all unchanged selection/link browser suites and native OS matrices remain task321 and the broader acceptance work. Full equivalence is NOT PROVEN.
