# Task 555 handoff

Task 555 is accepted without findings. Full xterm.js/Rust equivalence remains NOT PROVEN. This work is only the missing-current-row reverse-wrap backspace guard in `crates/xterm-core/src/terminal/controls.rs`.

Base is integrated main `224d1b33`. Own source commit is `6e731f13`, evidence/skill/docs commit `540cc601`. No private prerequisite copies, dependency changes or main ref operations. Pick these own commits and the later review/cleanup receipts only in the next canonical batch, not frozen task 554. No overlap with pending image 539/528 union or accepted marker changes 537.

The source optional lookup `lines.get(...)?.isWrapped` permits a missing current row; Rust now does the same, then still performs final cursor restriction and dirty-row handling. The previous-row `MissingLine` error after a real reversed wrap is unchanged.

Read `docs/backspace-sparse.md` and `results/backspace-sparse/`. Source 480 deterministic cases and both unchanged original BS suites, 8 tests, passed. Native 480 cases, existing 576 margin cases and all 8 original ports passed. Core all-target/all-feature tests passed 1,590 tests in 256 summaries, no failures/ignored/filtered tests. Strict core Clippy and workspace native all-target/all-feature plus actual WASM all-library/all-feature checks passed. Exact aggregate fmt failed OS206; all 5 complete member format checks passed. Full workspace runtime tests/strict lint/doctests and native Linux/macOS/browser runtime were not run on this producer and remain open. Initial harness mistakes and all failed controls remain disclosed.

Exact base controls.rs negative observed 472 matches and 8 intended missing-current-row errors with lost following BEL/render; fixed file restored by exact SHA before a successful focused rerun. Three generated tracked reports were copied losslessly into the evidence then restored. Independent reviewer ran the read-only 1,912-input/776-reference/3-executable/29-receipt audit and all cached focused tests, no compiler, no findings.

All heavy allocations were released on post 409 comment 4864. No services or browsers launched. The owned worktree is `O:/xterm-rust-swarm-16/tmp/worktrees/ZubaidaBackspace`. Cleanup must inspect regular files/reparse points and live processes, preserve this branch/archive and evidence, remove only that owned worktree normally, then verify the original reference hashes again. A later cleanup receipt records actual removal; do not replay cleanup from this preparation note. Cached O: executables and source-copy paths become historical after removal.

## Cleanup completed

Normal git worktree removal returned exit 0. The owned O: worktree is absent and unregistered. The preflight found 23,538 files and 3,982 directories, zero links, redirects or live workers. All 1,912 input pins and 29 receipt pins were preserved in Git, including exact copies of the two CRLF baseline inputs. All 776 original reference hashes passed AFTER removal. See `results/backspace-sparse/cleanup.json`. Cached executable and source-copy paths are now historical, not reusable build artifacts. Do not replay cleanup.
