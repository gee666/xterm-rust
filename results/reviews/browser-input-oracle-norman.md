# Independent browser input oracle review

Reviewed task 93 and 95 source loader, original tests, production Composition/Clipboard policies, Rust trace protocol and fixture tests after commit 1ad9eb8. No source assertions are rewritten. Scheduler classification separates geometry from observed state callbacks and rejects unknown synthetic callbacks. UTF-16 output/event boundaries and callback counts are compared, not only concatenated text. Fixture tests pin all 129 original and 72 directed operations.

Executed on Windows with Node 24.15.0 and repository Rust 1.85.1, shared target and reduced dev/test debug profiles unchanged:

- `node scripts/browser-input-oracle.mjs` passed 11 original tests, 16 original assertions, 252412 differential comparisons, seed 1229800758.
- `cargo test -p xterm-web --test input_upstream` passed both fixture replay tests, none ignored.

Earlier attempts failed on active unrelated shared-tree compile changes, retained in prior build logs and conversations. Current rerun passed. The native run emitted existing dead-code warnings from headless/events.rs with_queued_parse and program_status refresh_identity, so this is not a warning-free workspace claim.

This review accepts the isolated policy oracle and runner work, not DOM geometry, native browser IME input, complete keyboard/mouse integration or full equivalence. Task 33 remains in progress. Browser component tests are tracked separately under results/browser-host/wasm_input.
