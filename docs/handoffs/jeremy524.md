# Jeremy task 524 handoff

Continue the SAME swarm 2 full Rust xterm.js port and all 14 addons. Overall equivalence is NOT PROVEN.

## Exact producer history

Base `1106d480`. New owned commits only:

1. `85aafab3` adds native raw numeric actions, shared-field use, numeric pair rejection, source oracle, fixtures and tests.
2. `d9dcf57f` retains native quality, source replay, counts, source/executable pins and exact one-file policy control/restoration proof.
3. `18eb6cfa` records and restores three known workspace-generated output side effects without changing tests or assertions.
4. The final documentation-only tail containing this handoff records isolated review and safe cleanup preflight.

There are no private prerequisite copies. Do not replay accepted task 510 or other mouse producers. Canonical integration belongs to current task 309 owner Gordon, not this producer. Preserve the branch/archive before cleanup and cherry-pick only this new chain through normal integration gates. The current integration main advanced independently while these tests ran, so these results are scoped to the producer based on `1106d480`, not an untested later union.

## Accepted behavior and evidence

`MouseAction::RawNumber(f64)` represents values previously rejected or unrepresentable at the native enum boundary. Named/raw equality follows strict JavaScript numeric comparison; NaN is not `Eq`. Existing shared policies retain action values until explicit bitwise conversion. Native report filtering compares raw 2/3 numerically, and raw/named 32 share move debounce. Validated `TryFrom<u8>` behavior remains unchanged. No dependency or unsafe code was added.

- Original eight source assertions, 12,226 original core traces, 4,200 signed-button traces and three registry sequences replayed successfully.
- New unchanged-source fixtures contain 19,328 core rows at seed `0x524ac710`, 2,160 normalized report rows and a directed custom action-mutation assertion.
- Native debug focused core 33 and web 17 passed. Focused release core five and web 17 passed.
- MSRV 1.85.1 native workspace/all-target/all-feature tests passed 2,193 tests across 499 summaries, with zero failures, ignored tests, filtering or warnings. Two workspace doctests passed.
- Strict native workspace/all-target/all-feature Clippy and WASM workspace/all-feature library Clippy passed.
- Exact `1106d480` `mouse_policy.rs`, retaining the new action representation, failed the intended normalized-report test and passed the separate debounce test. Finally-restoration hashes matched, then both new web tests passed.
- Exact workspace formatting still failed with Windows OS206. All five complete package checks passed. Do not upgrade that scoped evidence to a full green format gate.

Independent review accepted `18eb6cfa` with no findings. The reviewer verified all nine working/committed source pins, all 21 executable hashes, both 776-entry references, raw counts and control/restoration hashes. It independently replayed every pinned executable and the unchanged-source oracle without compiling, launching browsers or modifying files. See `results/mouse-raw-actions/review.md` and the task review record.

## Cleanup and remaining scope

All compiler/browser/server allocations were explicitly released. Preflight scanned 21,729 files and 3,832 directories, with zero links, redirected directories or owned workers. Only the new review receipt was untracked before the final documentation commit. All temporary artifacts, target, scripts and the physical reference copy are owned under `O:/xterm-rust-swarm-16/tmp/worktrees/JeremyMouseAction`; no reference junction or shared tooling is in that deletion set. Final actual removal and post-removal original hashes belong in the private cleanup receipt, not a premature completion claim here.

Full raw ordinary-object mouse registries, inherited keys, nonnumeric action values, raw browser callback returns, actual DOM integration and native Linux/macOS execution remain outside task 524. Task 305 and the full port remain open. No new browser matrix or native-other-OS result is claimed.
