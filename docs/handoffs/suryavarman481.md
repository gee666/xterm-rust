# Suryavarman task481 handoff

Task481 is accepted. Full raw serialization346, family322 and overall equivalence remain NOT PROVEN.

## Exact owned chain

Branch `swarm/SuryavarmanHtmlScrollback` starts at main3a4f5e8f. Own commits are e0e941b7 probes/intent, ac5db73c production repair,21d194bf proof, then a review/preflight/handoff receipt. The final tip is posted on617 and409 after normal owned-worktree cleanup. No main ref was changed.

Do not replay accepted467/456/446. Main already contains456/446. This producer has72 cases, main3a4f's70 plus2. Accepted467 has2 oversized-column probes not in this branch. Later integration must keep both additions for74. The production changes are disjoint:481 handles default-range primitive NaN scrollback;467 handles explicit oversized startCol.

## Scope and proof

Numeric primitive NaN HTML scrollback makes source row traversal empty but still invokes header getters. Rust used to raise RangeError too early. The fix reuses the empty reversed-row range and unchanged shared native HTML serializer, preserving header/footer, getters, effects and exact throws. No ANSI/helper changes. Raw object/string coercion and full serialization gaps stay346.

Read docs/serialize-nan-scrollback.md and results/serialize-nan-scrollback/{summary.json,commands.json,review.md}. Baseline70/72×3 fails only2new probes; fixed72/72×3. Both runs pass unchanged source120+Rust120 expanded suite cases,0fail/skip/unrun. Original776reference hashes intact before/after.16native binaries69tests pass. Strict scoped native/tests/WASMlib Clippy+webfmt pass. Workspacefmt WindowsOS206, no full workspace/release/Linux/macOS claims.

Independent review accepted21d194bf after verifying46artifact/6source/16binary pins, reference integrity, all traces and suite counts. It independently reran all16native69tests, no compiler/browser. The fixed build input hash is bbe0503468ef723897da58c447bc316f70ffbf3bf020adb7e4c8f09db26f09cc. Use the task fixed/build-pointer.json and build-report.json, not the restored global latest pointer, to audit the package. Later review/cleanup receipts are outside the pre-review46 artifact pins.

## Resources and cleanup

The third heavy lane was released on409/3364 before review. No jobs/grants/servers remain. Scoped intent registered then global intent restoredr170. Initial baseline-build TMP/TMPDIR export may have inherited external TEMP; exact values unrecorded, disclosed in docs/commands. Later commands set all three to owned O: runtime. Do not delete unknown external paths.

Only known owned temporary root is O:/xterm-rust-swarm-16/tmp/worktrees/SuryavarmanHtmlScrollback, including private target, copied reference, generated package and tmp runtime. Generated tracked default reports and native TSV were restored after review. Source and evidence are committed before deletion. Cleanup preflight records link/process/reference checks. Normal worktree removal follows, with absence/unregistration/reference integrity posted on617. Do not replay deletion if a postcheck fails.
