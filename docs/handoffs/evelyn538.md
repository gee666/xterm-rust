# Evelyn 538 pickup

Task 538 is ACCEPTED after resumed independent review of `9d863af8`, no findings. Exact own chain is `cee7075a` → `332e2fea` → `6407b56e` → `9d863af8`, followed by this receipt. Base main `0442e357`. No task-530 or other private prerequisite copies. Coordinator 309 owns canonical integration. This work is NEXT after the frozen 543 batch, not permission to replay its gates.

Production changes only `font_gsub_other.rs` generic types 2/3/4 zero coverage pointers and `font_build_helpers.rs` reached missing-coverage error family. Direct generic lookup construction tolerates absence; reached scalar/range references throw the source format-property TypeError instead of accepting an invented empty table. Type-7 outer references have no coverage even when their inner table is valid, so their error family is corrected too. Later set, optional filter and META ordering remain intact.

Future shared-file integration MUST preserve `subtables`, task-530 `single`, and task-538 `noncontext` supported corpus names/recipes, load comparisons and report families in both shared oracle/browser scripts. Preserve the missing-property mappings, task-530 invalid type-1 format mapping, and task-538 `format` TypeError mapping in shared native assertions. Keep both production changes in `font_gsub_other.rs`; they are distinct functions. Do not replay 517/530 source commits as private prerequisites.

## Evidence

`docs/font-gsub-noncontext.md` and `results/font-gsub-noncontext/` contain recipes, exact commands, raw logs, pins, counts and limitations. The new deterministic SFNT/WOFF corpus has 1,044 cases. Unchanged-source parser accepts 480/rejects 564; FontImpl accepts 408/rejects 636. Node generation followed by immutable replay passes. Source parser, FontImpl and actual addon each match all 1,044 cases in Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4.

Actual pre-fix native production fails 72 comparisons: 24 incorrectly accepted direct noncontext references and 48 wrong-family extension references. Final native focused test replays 1,384 retained plus 1,044 new cases and passes. Final addon all-target/all-feature suite passes 211 tests across 51 summaries, with zero failed, ignored, measured, filtered tests or warnings. Strict native addon/all-target/all-feature and new WASM-test Clippy pass, as do dependent font/web WASM library checks.

The actual Rust browser suite passes two functions in each engine, zero failures/ignored/filtered, in 14.66 s / 143.01 s / 18.06 s. Its managed outer command exits normally with code 0 in 185.63 s. No retries or post-pass source/assertion/compiler changes. The independent reviewer ran the pinned native executable and checked all 16 producer pins, 17 browser runtime pins, actual/prewarm artifact equality, both 776-file reference manifests and the actual OpenType dependency hash. No reviewer compiler, browser, bundler, server or cleanup launches.

Exact aggregate formatting still fails with Windows OS error 206 on the long physical path; each of five whole-package checks passes, not a waiver. Raw native/browser runner blank-EOF diff findings are retained verbatim, not trimmed. Rust addon diagnostics retain Chromium wording on Firefox/WebKit, so exact cross-engine error messages remain unproven. These are component-host loading/cache/fallback/refresh tests, not full Terminal/original-suite equivalence.

## Resources and cleanup

Compiler grant 4464 released in 4515. Browser grant 4567 started in 4576 only after the previous holder's actual release and a fresh 29.88 GiB RAM / 2,490.55 GiB O: probe; ALL released in 4586. Zero relevant owned processes verified. Both final native and browser commands use explicit O: `TEMP`, `TMP` and `TMPDIR`, private O: target, Cargo -j2. Initial compiler commands assigned `TMP` from its inherited `/tmp` value due shell expansion in the same export command. This deviation is documented, earlier transient routing is not verified and no unknown system temporary paths were deleted.

Owned temporary root is `O:/xterm-rust-swarm-16/tmp/worktrees/EvelynCoverage`, including private target/tmp, generated browser artifacts and physical reference copy. Live binary/runtime pins are available through review only and become historical after cleanup. This committed receipt is NOT confirmation of removal. Actual final preflight, removal and original-reference recheck are recorded privately at `.pi/swarm/work/2/evelyn538-cleanup.json` and `.pi/swarm/work/2/evelyn538-progress.md`. Do not repeat removal after those confirm completion. Preserve the reviewed source branch/archive and durable evidence, original source_repo/AGENTS/assertions/credentials, canonical main and every other owner's paths.

Full task 342 metadata/classes/shaping, subsystem closure 312-325, original/browser/demo 327, native quality/platform 304, mapping 328 and comparisons 329 remain open. Full functionality and equivalence are NOT PROVEN.
