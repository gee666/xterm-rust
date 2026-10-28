# Berenice integration handoff

Full equivalence remains NOT PROVEN. I completed task267, not the broad task28.

## Current branch

Use C:/Users/user/.xterm-rust-target/7144d30a1fc3-worktrees/Berenice on swarm/Berenice. Main remains561b705 in the Integration worktree. Dirty primary master is preserved. No other agent's dirty files were stashed or reset.

Branch commits after main include Coleman accepted162 and its prerequisite full-trace runner, Antigone immediate parser disposal and virtual reset override, my accepted Windows ordering fix0bb8aba, and Willie diagnostic phase assertion/formatting fixes4f34bd0/04cddcf as bb87967/9d4c0a0. These are cherry-picks, so future rebases may skip duplicate patches.

The worktree has an ordinary private target and a physical read-only source reference copy. Its source has no node_modules. For source runners I passed the original primary source_repo path explicitly, without changing reference contents. The Windows source command accepts that root.

## Accepted Windows ordering work

Task267 replaces inline LF and CSI H effects with real source-time registrations. The shared common/native LF emitter preserves subscription order. CSI H returns false and has actual registrar priority. Enabling again is idempotent, disable removes both handles, and reenable appends registrations.

Independent review found a genuine disposal snapshot bug in my first patch. Fixed native disposal to remove Windows registrations before clearing the shared LF emitter, including inside a parser callback. Both before/after-dispose cases now match unchanged source. The reviewer accepted after rerunning15 traces,20 scoped tests,61 extra regressions and Clippy. See docs/windows-wrapping-order.md and results/windows-wrapping/.

## Integration evidence

I read the carried artifacts instead of replaying their side effects. The merge helper rebased without conflicts, built xterm-core and ran1478 passing tests across221 core test/example suites. It then stopped at dependent all-target web compilation:

- xterm_web::character_joiner is missing from the crate root for character_joiner_upstream.
- tests/support/cells.rs passes u32 where ExtendedAttrs::new now requires a u64 URL id.

Main was not advanced. The exact transcript is results/integration/berenice/gated-merge.log. Xavier task31 now points to b956eeb with a newer working Root WASM producer. Coordinate that branch rather than editing competing copies. Native test imports/fixture correction still need actual integration.

The initial full-core logger assertion failed at core_services_integration.rs188. I inspected and cherry-picked Willie4f34bd0: the strengthened test checks all four records and x positions0,0,2,2. That clears the failure without dropping assertions. Task264 tracks the main merge. Full-core Clippy still fails pre-existing move_to_cell format_collect and input_handler_bounded single_range_in_vec_init on local1.85.1. Scoped changed-code Clippy passes. No blanket allows or skips were added.

## Observed Coleman CI

I retrieved run40662131350 logs and all eight artifacts with existing scripts/github-quality.mjs. Credentials stayed in memory. Raw receipts are results/quality/ci-40662131350/.

Observed status: six native Windows/Linux/macOS jobs failed global quality/build gates, WASM compilation failed, focused Miri succeeded. This is NOT native passing evidence. Concrete failures include:

- Stable1.99 Clippy Iterator.last on a DoubleEndedIterator in search/matching.rs65. Jeremiah's newer branch already fixes this.
- Native web tests import missing character_joiner/search_host exports and use the old u32 URL fixture.
- WASM lacks root js_error/terminal/character_joiner exports and their dependent paths on that old branch.
- Formatting differences and the wider gates are retained in the uploaded job artifacts.

These are code/integration failures, not external platform blockers.

## Remaining task28

-194 depends on live Params producer226. Both are new/unowned at my last read. Diagnostic helpers alone are not the final host identity behavior.
-257 remains the genuine asynchronous resize replay defect. Antigone's untracked failing crates/xterm-core/tests/headless_async_resize.rs is still in the Antigone worktree, with a safety copy under this worktree tmp/. Source async A BEL B with Bell→resize produces AABB and two Bell/resize events; Rust produces AB. Do not replay by adding characters or weaken its expectations. The mutable engine/parser borrow must be redesigned for actual safe recursive parsing. I did not attempt an unsafe alias or commit a fake fix.
-212 slow warning races are separate.
-229 decoder overflow policy remains a documented source hang/safety deviation needing an explicit compatibility decision. No user approval was observed.
- Final browser/full upstream/OS suites and the entire user scope remain required. No helper smoke suite closes them.

Task28 now records dependencies194/257/267.267 is accepted, the other two are implementation work, not blockers. Use the guarded merge script once the dependent web producer fixes are integrated. Restore only known generated result reports after failed test runs, never source or other agents' work.
