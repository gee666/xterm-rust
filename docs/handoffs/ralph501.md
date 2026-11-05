# Ralph 501 continuation

Continue the same swarm #2 complete Rust xterm.js and all 14 addons port. Full equivalence is NOT PROVEN.

Task 501 is accepted. The application source commit is `7ed21ee3` on `swarm/RalphMouseNames`, based on `5088f8fd`. There are no private prerequisite copies or replay dependencies. Only task 501's later review/cleanup receipt follows it. Discover the exact branch tail before integration; task 309 owns canonical main integration. Do not run a second main-ref writer.

The change makes native mouse `protocol_names` and `encoding_names` list canonical numeric indices first and other names in insertion order. It shares the existing UnicodeService key-order algorithm through a crate-private helper. Registry storage, callback authority, mode state and browser code are unchanged. Original source assertions and accepted producers were preserved.

Evidence is `results/mouse-registry-names/`, `docs/mouse-registry-names.md` and the immutable scoped checker attempt linked there. Eight unchanged source tests, 12,226 existing mouse traces, 178 new registration operations and 187 snapshots per registry passed. Native core all-target/all-feature 1,571 tests, focused 29 debug and 29 release tests, Unicode release 20 tests, strict full native workspace all-target/all-feature Clippy and WASM workspace all-feature library Clippy passed. Exact base 5088 getter code failed all three new runtime tests; finally-restored fixed bytes passed all three. Exact workspace formatting failed with Windows OS206; all five member checks passed.

The isolated reviewer found no concrete defects. It independently replayed the source oracle in memory, ran 3+15+5+2 native tests, checked 11 working source pins and four executable pins, explicitly accounted for two CRLF-only Git differences, verified negative/restoration hashes and both 776-file reference trees. The review receipt is retained. Runtime EXEs are temporary and will be removed, not claimed as durable binaries.

Grant 3727 was fully released in comment 3766. No browser, server, main-ref or CI operation was used. Cleanup preflight is retained in results. The final removal receipt and current owned-resource accounting are in the launch checkout's private `.pi/swarm/work/2/ralph501-cleanup.json`. Before repeating an action, check that receipt and the current worktree list. Never traverse reference links or delete another owner's paths.

Task 305 retains raw inherited/prototype registries, `__proto__` writes, non-string/object keys, raw callback returns, borrowed receivers, browser method replacement and complete live facade behavior. Full workspace runtime, native Linux/macOS, full source/browser/demo suites and matched comparison evidence remain open.
