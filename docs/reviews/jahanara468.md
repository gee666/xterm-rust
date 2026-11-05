# Independent render scheduler review

Task468 was accepted through 789c42c5 after a source-first review caught an early cancellation-handle read. The final late-resolution hook reads the current pending ID after window and cancellation-property lookup. Reentry can clear the ID to JavaScript undefined or replace it before cancellation. No scheduler borrow survives those host calls, and pending state clears only after successful cancellation.

The reviewer independently reran all five hash-verified native executables. All 74 tests passed without failures, ignored tests or filtering. The 66-program oracle matched unchanged source. Changed Rust formatting and source whitespace checks passed. Both 776-file reference inventories matched their recorded sizes and SHA-256 hashes.

The reviewer verified source, lockfile, executable and report pins, generated WASM/bindings hashes, bundled oracle snippets and raw browser logs. Initial and restored final scheduler tests passed both Rust functions in Chromium, Firefox and WebKit. Existing render-service tests passed their one function per engine. Each negative control failed exactly one differential function while the realWindow RAF test passed. Restoration hashes match current production bytes; initial and final generated WASM/bindings hashes are identical. Recorded strict focused native/WASM Clippy and both WASM builds passed.

The reviewer launched no compiler/browser jobs and changed no files. Exact workspace formatting retains Windows OS206; five package checks passed. Acceptance is limited to468. Full317/318/301, throwing WindowSource, raw JavaScript row values and whole-port equivalence remain open.
