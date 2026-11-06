# Richard 549 pickup

Task 549 is ACCEPTED after independent review of `b8f448b0`, no findings. Own source/evidence chain is `b1b0a742` → `acb63750` → `b8f448b0`, followed by this receipt. Base is main `0442e357`; no private prerequisite copies. Branch `swarm/RichardMarker` preserves the work. Coordinator 309 owns canonical integration. Treat this as NEXT after the frozen 543 batch, not permission to append to or rerun that batch.

Production changes only `crates/xterm-web/src/serialize_addon/options.rs`. Raw ANSI marker coordinates now reach the source end-minus-start coercion before native indexes. End coerces before start with the number hint. Both operands remain raw references, thrown values retain identity, and numeric invalid differences reach the real browser Array constructor. The minimal operator glue preserves source names `endRow`/`startRow` because Firefox includes identifiers in nonprimitive-conversion errors. No terminal serializer was delegated to JavaScript. There is no dependency, unsafe code or source assertion change.

Shared-file integration must preserve all existing cases plus the three new raw-marker cases in `scripts/serialize-browser-cases.mjs`, and keep `serialize-browser-check.mjs` expecting 93 total cases, 61 directed and 32 seeded. If another producer adds cases, make an additive count union. Do not narrow or normalize error comparisons. Keep the source-named operator parameters.

## Evidence and review

`docs/serialize-raw-marker.md` and `results/serialize-raw-marker/` retain commands, raw reports, screenshots, copied build receipts, source hashes and limitations. The old exact 0442/531 package control matches 90/93 per engine and fails exactly the three new groups. Its 75 package files and 1,866 source inputs were verified around transfer and execution. The first fixed attempt passes Chromium/WebKit but fails two Firefox subcases because operand identifiers differ. That actual failure remains under `initial-fixed/`, not hidden by a retry.

Final Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4 each match 93/93, including all 35 new subcases. The final unchanged original suite passes 120 source and 120 Rust cases, 40 each per engine, with no failures, retries, skips or unrun tests. The initial fixed attempt also passes those complete original suites. The seeded corpus remains `0x05e21a11`, 32 traces of 12 operations.

Final native xterm-web library/all-feature tests pass 56, zero failed/ignored/measured/filtered. Strict native/all-feature library and WASM library Clippy pass. Debug WASM build and actual matching package verification pass. Aggregate formatting still fails with Windows OS error 206; all five whole-package checks pass, not a waiver. Three raw-log blank-EOF findings are retained verbatim. The private summary helper's pre-write Array.values error is also retained, with its effect inspection and corrected summarization.

The reviewer independently verified all 95 producer pins and 81 runtime pins, recomputed source identity `331270f568a83bff413d2c729b6cfd104ec78c4b05236c8dae2e01cf3f752de4` across 1,866 inputs, passed live `verifyBuiltPackage`, and checked both 776-entry reference manifests. The reviewer recompared all retained traces, recounted original-suite results and directly reran the pinned native executable, 56 passed. No reviewer compiler, browser, server, ref or cleanup operations occurred. Runtime pins are retained through review and become historical after cleanup.

## Resources and cleanup

Grant 4665 started in comment 4686 only after task 547's actual compiler release 4679. The actual fresh start probe was 30.05 GiB RAM and 2,489.77 GiB free on O:, correcting an earlier approximate comment. Compiler/browser work was sequential, Cargo -j2, with literal owned O: TEMP/TMP/TMPDIR and a private target. ALL compiler/browser/server resources were released in comment 4710, zero relevant owned processes verified.

Owned temporary root is `O:/xterm-rust-swarm-16/tmp/worktrees/RichardMarker`. It includes private targets, three temporary package copies, generated active browser package, temporary helpers and a physical regular reference copy. No dependency/reference junction was created or copied. The three untracked temporary build-report trees are archived under `borrowed-build`, `initial-build` and `final-build` before removal; the tracked live build pointer must be restored after review. Standard serializer output paths were already restored after retaining their new raw results.

This receipt is NOT confirmation of whole-worktree removal. Actual preflight/removal, archive identity, dirty/untracked inspection, process/link checks and the original-reference AFTER check are recorded privately in `.pi/swarm/work/2/richard549-progress.md` and `richard549-cleanup*.json`. Verify those before repeating any cleanup. Never delete other owners' work, reference roots, AGENTS rules, credentials or preserved main/user fixes.

## Remaining scope

Valid arbitrary-object row traversal, pure-BigInt traversal, arbitrary replaced public methods and other raw contracts remain tasks 346/322. Full subsystem closure 312-325, original/demo 327, native quality/platform 304, exhaustive mapping 328 and matched comparisons 329 remain unfinished. These are not external blockers. The complete Rust port and equivalence are NOT PROVEN.
