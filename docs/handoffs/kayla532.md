# Kayla 532 handoff

Continue the same swarm 2 full Rust port. Overall equivalence is NOT PROVEN.

Task 532 is accepted. Independent review found no problems at `0ac47e05`. Base `11ad71a8`; exact source/evidence commits `ba5e1094`, `11fa184b`, `0ac47e05`. Later review and cleanup receipts contain only documentation/evidence. No private prerequisites, 524/510 replay, original-reference changes or main-ref operations.

The only production change is `crates/xterm-web/src/input/mouse_coords.rs`. Explicit comparisons preserve JavaScript zero ties instead of depending on Rust max/min optimization choices. Native MSRV debug baseline fails at fixture row 1356, actual positive zero versus source negative zero. The old optimized build passes. The fixed debug and optimized builds pass. Actual Cargo exact-base single-file control reproduces the same failure, then a finally block restores the fixed SHA and the final test passes.

`docs/mouse-coordinate-zero.md` and `results/mouse-coordinate-zero` retain commands, source fixture, counts, raw logs, pins and review. Source corpus 3,249 directed cases, both original/copy references match 776 hashes. Native Windows MSRV 1.85.1 xterm-web all-target/all-feature 418 tests pass; focused release 14 pass. Strict native xterm-web all-target/all-feature and WASM workspace all-feature library plus new WASM test Clippy pass. Full workspace fmt OS206 fails explicitly; all five complete package formatting checks pass.

Actual Rust WASM new one-function suite plus retained six/seven-function suites pass in Chromium 148, Firefox 150 and WebKit 26.4, 14 functions per engine, 42 expanded executions, zero fail/ignore/filter/page errors. Five source pins, seven native executable pins and 223 runtime pins were independently verified. All seven executables independently reran 29 tests. This is scoped proof, not complete original-suite, native Linux/macOS or overall equivalence proof.

Canonical integration belongs to task 309, currently Marilyn. This producer is NEXT after the frozen task 531 batch, not permission to alter that freeze. Preserve exact own history once only. Raw mouse registry/callback facade 305 and full port remain open.

All heavy allocations and browser/server processes released at post 409 comment 4335. Owned temporary root is `O:/xterm-rust-swarm-16/tmp/worktrees/KaylaMouseZero`, including private target, tmp and physical reference copy. Cleanup must preserve committed source/evidence, verify archive refs, inspect all reparse points and worker processes, remove only identified generated untracked outputs, then use normal git worktree remove. The private `.pi/swarm/work/2/kayla532-cleanup.json` records the final actual result. Do not repeat cleanup after it records completion.
