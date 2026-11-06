# Paul525 handoff

Task 525 is independently accepted at `008e24ce`, based on main `11ad71a8`. Full selection/link task 321 and complete Rust xterm.js/all-14-addon equivalence remain NOT PROVEN.

## Pickup

The reviewed own chain is `f6cab31b` → `774a6360` → `2fe5e8bc` → `e19e74a1` → `008e24ce`, on `swarm/PaulLinkRender`. Review and cleanup-only tails are reported in post 409. There are no private prerequisites or task 514 copies. Preserve original commits, including source-equivalent refinements, rather than replaying the archived pre-rebase source attempt.

Production changes are confined to `links/browser_linkifier.rs`, `browser_linkifier_events.rs` and `browser_payload.rs`. The repair repeats render invalidation's range predicate at the source `_clearCurrentLink` phase, preserves falsey-bound short circuiting and reads the current link again for each endpoint. Getter exceptions stop requery. A false second predicate keeps the hovered link but still re-asks providers. No state borrow spans JavaScript execution. Native typed APIs and task 500 underline geometry remain intact.

The public source runner adds `--render-recheck`. When composing with task 514, keep both its `--mouse-path` flag and this flag, plus task 500's existing `--underline-geometry` flag and all other modes. Task 514's mouse-path production hunk is separate from the render changes. There is no delegated JavaScript terminal engine in the Rust tests.

## Evidence

Read `docs/link-render-recheck.md` and `results/link-render-recheck/index.json`. Source methods passed 12 cases. Opened original and Rust Terminals passed the same eight public cases in Chromium, Firefox and WebKit. Rust's second function passed twelve direct-controller cases per engine. Initial and restored-final Rust runs passed both functions per engine. The exact three old production files from `11ad71a8` compiled and failed Chromium with zero passed and two failed at the intended missing second reads. Three source hashes were restored in `finally`. Initial/final compiler WASM, served WASM and bindings hashes match; all negative hashes differ. Each new-test build has `fresh=false` and embeds the full current trace, component program and fixture. The negative compiler path is historically reused, not a surviving raw compiler-file pin. Its separate served WASM is inspectable until cleanup.

Eight retained browser suites passed 20 functions per engine, including task 500 underline geometry. Native web all-target/all-feature tests passed 417 tests across 174 summaries with no failures, ignores, measurements, filtering or warnings. Strict native workspace and WASM library/new-test Clippy passed on Rust 1.85.1. Five whole-member format checks passed. Exact workspace formatting failed with Windows OS error 206 and remains explicitly unproven. The initial source-browser adapter error race is retained, with its completion fix and unchanged deadlines/assertions documented.

The clean reviewer reran all 12 source cases and 11 cached native executables containing 47 tests. It verified all 2,246 input pins, 137 receipt pins, 803 runtime pins and 11 executable pins, committed/current bytes with newline conversion accounted for, browser/control/restoration identities and member formatting. No findings. See `review.json`. Both original and physical reference trees matched all 776 baseline hashes before and after tests.

## Resources and cleanup

All compiler, browser and server allocations were released in post 409 comment 4196. No owned jobs remain. The only owned temporary root is `O:/xterm-rust-swarm-16/tmp/worktrees/PaulLinkRender`, including its regular target, tmp and physical source copy. It remains solely for accepted-work cleanup. No main-ref, remote CI, browser, build or gate replay is authorized by this handoff.

Before removal the owner will preserve this branch and review receipt, inspect every path for reparse points/redirects and every owned process, remove only verified new generated browser outputs, and use normal non-force Git worktree removal. The subsequent post 409 report and private `.pi/swarm/work/2/paul525-cleanup.json` establish actual absence, unregistration and original reference hashes after removal. This pre-removal document does not claim those actions already happened. Never delete another owner's worktree or traverse a reference junction.
