# ED2 option getter reentry

Task 513 continues the same swarm's full Rust port. It does not close full parser task 313, original-suite task 327, quality task 304 or overall equivalence.

## Source contract and repair

`InputHandler.eraseInDisplay` clamps the cursor before branching on the erase parameter. For ED2 and selective ED2, it reads `rawOptions.scrollOnEraseInDisplay` before capturing the row count, scanning lines or clearing rows. The getter can reset, resize, activate another buffer, change attributes or cursor, and throw. Subsequent operations use the live state. Truthy raw values select scrolling; falsy values select row reset. Getter errors retain their original identity and any committed cursor clamp or callback mutations.

The Rust `terminal/csi_erase.rs` implementation retained an immutable `TerminalState` borrow while calling that getter through `option_bool`. Reentrant callbacks needing mutable state therefore failed with a borrow error. The repair captures the shared options service, releases state ownership, then reads the option. The row count and erase receiver remain live lookups after the getter. No new dependency, public API, unsafe code, rendering policy or duplicate terminal state was added.

## Reproduction

```sh
XTERM_SOURCE_ROOT=/path/to/untouched/source_repo node scripts/erase-option-oracle.mjs
cargo test --locked -p xterm-core --test erase_option -j2
```

`XTERM_SOURCE_ROOT` is optional when the current checkout contains the reference and its installed esbuild dependency. The source script bundles the unchanged headless terminal, verifies all 776 baseline reference hashes before and after, and removes the owned temporary bundle in `finally`. Normal execution rejects fixture drift. `--generate` deliberately captures the fixture; `--no-receipt` replays without rewriting the source receipt.

The 396 deterministic cases cover both initial buffers, selective and ordinary ED2, nine raw values and eleven actions. Values include false, true, zero, NaN, empty and nonempty strings, an object, null and undefined. Actions cover inspection, reset, activation of either buffer, resize, cursor or attribute mutation, getter errors before and after mutation, and scroll listeners that reset or throw. Each case starts beyond the allowed cursor column to verify the pre-getter clamp, including on error paths.

The comparison includes both buffers and every cell's text, width and attributes, cursor, scrollback, margins, dimensions, modes, getter counts, callback checkpoint states, original callback-error identity and ordered scroll, resize, cursor and render events. Empty event lists use `-` in the TSV to avoid trailing-tab whitespace. The generated fixture exceeds 350 lines because each line preserves one complete source case. The fixture contains 84 expected thrown results. Source baseline execution is not counted as a Rust test pass. Native Rust tests execute Rust, not the JavaScript engine.

## Verification status

Initial inspection used main `e64f4870`. Before compilation, the owned branch rebased cleanly onto integrated main `1106d480`. That integration did not change `csi_erase.rs`. The tested source revision is `e0093295`, with the production change in `3f84b373`. Later commits retain documentation, evidence and review only. Raw logs and receipts are under `results/erase-option/`.

- Source generation, non-generating replay and final replay pass all 396 cases and preserve all 776 original hashes before and after each run.
- Native execution passes all 396 cases in one test function. The pinned live executable also passes a direct rerun. `source-pins.json` records ten exact source/build inputs and the live binary hash before cleanup.
- Restoring the old production file from `e64f4870` fails 108 runtime cases. A borrow-free control that captures the row count before the getter fails 24. Both controls restore the exact fixed file in `finally`, check its hash, then pass the full 396-case fixed rerun. `negative-summary.json` records variant, exit status, mismatch count and restored hashes. Neither negative is a compile failure.
- Native Windows MSVC Rust 1.85.1 workspace all-target/all-feature tests pass 2,189 tests across 498 result summaries, with zero failed, ignored or filtered tests. Workspace all-feature doctests pass two tests across five summaries.
- Strict native workspace all-target/all-feature Clippy passes. WASM workspace all-feature library check and strict Clippy pass. WASM compilation is not browser execution.
- Exact `cargo fmt --all -- --check` fails with Windows OS error 206. All five packages pass individually through `cargo fmt -p PACKAGE -- --check`, covering the same workspace members without the oversized aggregate invocation. The exact aggregate command remains a recorded failure, not a green gate.
- Workspace tests regenerate three old result reports. Their changes were inspected, summarized in `generated-effects.json` and restored to their exact baseline hashes. The separately retained buffer summary reports 73 passing traces and 862 snapshots. No unrelated source or reference was changed.

The only pre-test adjustment changed empty-event TSV encoding to `-`, retaining the same empty event assertion. It avoids trailing tabs flagged by `git diff --check`. No assertion was dropped or weakened. The compiler allocation was released after the checks. Independent review accepted `8533c914` with no findings. The reviewer verified all ten inputs, the executable and 28 receipt pins, independently executed the native 396-case test and source 396-case oracle, checked all 776 reference hashes before and after, and recounted the negative and workspace logs. It started no rebuild. See `results/erase-option/review.md`.

## Temporary outputs

The owned temporary root is `O:/xterm-rust-swarm-16/tmp/worktrees/TaharqaErase`. Source, fixtures, receipts and review are preserved on `swarm/TaharqaErase` before normal worktree removal. `cleanup-preflight.json` records 21,503 files, 3,785 directories, no links or redirected paths, no owned workers outside the synchronous preflight processes, zero heavy jobs and all 776 original reference hashes passing. It is a pre-removal observation, not a removal receipt. Final removal and post-removal source verification are reported on task 513's coordination posts.

The executable pin is historical metadata after cleanup, not a promise that the temporary binary remains available. Independent execution completed before cleanup. No reference junction was created or traversed. No AGENTS file, credentials, private runtime data or other owner's path was changed or removed.

These cases are finite, deterministic, source-derived regressions. They are not randomized coverage, a complete original InputHandler suite, or browser binding evidence. No Linux/macOS runtime, CI, release/performance/size comparison or full browser/original-suite matrix has run for this task. Full equivalence remains NOT PROVEN.
