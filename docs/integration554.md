# Reviewed public API and renderer integration

Task 554 assembles 73 reviewed producer commits on main `224d1b339a43eddf899c2944d9dad734d8c44234`. The frozen source assembly is `6cfd03d7`. The source is not yet integrated into main at this writing. Later producer code is excluded. This document will record actual integration and review results when they occur.

Full xterm.js and fourteen-addon equivalence remains NOT PROVEN. This integration does not accept the unfinished parent families, full upstream/demo matrices, native-platform proof, complete mapping or comparisons.

## Frozen inputs

`results/integration554/plan.json` identifies each accepted producer and the exact commit chain. `originals.json` records full object IDs, parents, subjects and paths. `applied.json` maps every original to its applied commit in the same order. Public decoration task 537 was applied first.

The batch includes 537, receipt-only 533 and 543 tails, 536, 538, 539, 540, 541 and 548, 544, 545, 546, 547, 549, 550, 551 and 552. It excludes all fifty originals already integrated by 543 and all private prerequisite copies. Later 553, 555, 556, 557, 559 and 560 source is not part of this assembly. Post-freeze review/cleanup receipts from 547, 548, 551 and 552 remain for a later batch.

The immutable-object audit derives expected content from the pinned base and producer Git objects, not from the assembly being checked. It checks 167 changed source files, 1,582 receipt pins and 576 preserved source pins. All match. No unexpected production/script/workflow/skill changes were found. Fifteen shared files required either an additive merge or independent three-way composition.

The shared merges preserve:

- GSUB subtable and single-substitution corpora alongside new noncontext coverage.
- Raw palette behavior, coordinate behavior and all ten palette tests alongside thirteen raw pixel tests. The image union has 87 Rust test functions.
- Renderer/storage snapshots before each protocol constructor conversion, pixel multiplication before palette capture, and delayed decoder palette conversion.
- Existing WebGL canvas-layer exports, new context lifecycle and actual model/GPU frame composition.
- Retired search timeout phases, delayed cache expiry and both source-oracle fixture selectors.
- Existing public marker capabilities needed by the decoration binding.

The palette/pixel combination removes the final unused typed options field from the read-only option inspector and its unused numeric projection closure. The `imageOptionNumber` ABI remains because the SIXEL palette adapter still calls it. No dead-code suppression, source-assertion edit, dependency, unsafe operation or JavaScript-terminal delegation was added during assembly.

## Native Windows checks

All checks use owned O: targets and explicit O: TEMP, TMP and TMPDIR. Cargo uses two jobs. The recorded toolchain is Rust 1.85.1, Cargo 1.85.1 and Node 24.15.0.

- Warning-denied native workspace all-target/all-feature Clippy passed.
- Warning-denied WASM workspace library all-feature Clippy passed.
- Native workspace all-target/all-feature tests passed 2,264 functions across 540 summaries, with no failures, ignored, measured or filtered tests and no warnings.
- Workspace all-feature doctests passed two tests across five summaries.
- The exact `cargo fmt --all -- --check` failed with Windows OS206. Five complete member-format checks passed. The aggregate command is not waived or reported green.
- Eleven existing E2E transport/bootstrap/server unit tests passed.

Three known generated native result files changed during the tests. Their exact binary patch and new hashes were retained before restoring those generated checkout copies. Nine cached executables containing 168 tests were copied without modification to owned review scratch, with original and copied path/hash records. This preserves independent execution after the canonical helper removes the build target.

## Browser and original-suite checks

A matching debug production package was built and its source/artifact identities retained. Seven existing dependency packages were physically copied for unchanged-suite execution. All 313 regular files were SHA-verified. No dependency install or reference junction was created.

The initial integrated image matrix ran all 87 functions in each engine. Chromium 148 and WebKit 26.4 each passed all 87 without page or console errors. Firefox 150 passed 86 and failed one exact diagnostic comparison in `raw_pixel_iip_throws_at_each_comparison_without_catching_payload_error`.

Rust's native TypeError says `can't convert this._opts.pixelLimit to number: its [Symbol.toPrimitive] method returned an object`. The unchanged source instance says `can't convert ({[Symbol.toPrimitive](){return {}}}) to number: its [Symbol.toPrimitive] method returned an object`. The error-name assertion passes before the exact-message assertion fails. The complete failure, console stack, compiler/served-WASM/binding hashes and raw logs are retained in `image-initial-results.json` and `browser-raw/image-initial`.

This is not a green image matrix. No assertion was normalized, ignored or weakened. No browser optimization setting or production warmup workaround was added. Separate task 556 also observed Firefox changing its own nonprimitive diagnostic operand from a field name to an inlined object expression after warming unchanged source code. That finding does not waive this failure. Full image tasks 311 and 324 retain the diagnostic and broader behavior obligations. Post 710 links this exact integrated failure to the work graph.

After preserving the complete failed image matrix, execution resumed explicitly at the public-decoration suite. No image engine or passed earlier suite was replayed. The remaining fourteen suites passed 90 functions in each of Chromium, Firefox and WebKit, with zero failed, ignored or filtered functions, page errors or console errors. These include public decoration 1, Attach removal/lifetime 2, search 22 plus delayed expiry 1, selection teardown 1, accessibility component/Terminal 33 plus 8, WebGL context/frame 4 plus 11, Progress 1, GSUB single/noncontext 2 plus 2, and CFF2 font dictionaries 2. The combined fifteen-suite accounting is 531 discovered functions, 530 passed and one failed.

The matching production package passed unchanged decoration suites with 33 discovered case-projects, 27 passed and six original WebKit skips, with no failures or unrun bodies. The serializer passed all 93 directed/seeded traces in each engine. Both unchanged original SerializeAddon suite executions, source and Rust, passed all 120 case-projects with no failures, skips or unrun bodies. This does not replace the full core/addon matrix. Raw reports, output traces and screenshots remain under `results/integration554`.

The source input hash stayed unchanged through all browser checks. Generated shared browser reports were losslessly patched and hash-recorded before restoring their checkout copies, including the historical production-build pointer. Both original and physical reference manifests are retained. No failing assertion or production source was changed to make the matrix pass.

## Main, review and cleanup

Canonical gated merge, isolated independent review and final owned cleanup have not occurred at this writing. They must be recorded from actual receipts, not inferred from successful source assembly or native checks.

The only owned temporary root is `O:/xterm-rust-swarm-16/tmp/worktrees/Mackenzie554`. Preserve committed evidence and branches, verify runtime hashes and both reference manifests, inspect links/redirects and owned processes, then use normal no-force Git worktree removal after acceptance. Do not clean another owner's paths, caches, credentials or the legacy dirty P: checkout. No remote push or CI launch is authorized by this document.
