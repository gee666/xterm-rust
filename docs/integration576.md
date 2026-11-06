# Frozen integration batch 576

Overall equivalence is NOT PROVEN. This checkpoint combines reviewed producer work on main `14139449bae2792afb019f42e8438b074938cfe7`; it does not close the full source, native-platform, browser, addon, demonstration or comparison requirements.

## Source and ownership

`results/integration576/plan.json`, `originals.json` and `applied.json` identify 88 accepted original commits. They were already assembled before Lydia took over from Sydney. No producer commit was replayed during finalization. Later application producers remain NEXT, including 575 and 580 onward. Private prerequisite copies and the duplicate 572 implementation are excluded. The original 584 commits are recorded as original equals applied, not cherry-picked again.

The frozen source assembly is `361637e0`, freeze metadata `91d730b3`, and final checkout-contract adjustment `d450e5b1`. Seven exact-path LF rules preserve accepted process/fixture proof bytes; they do not alter Rust behavior or assertions. The immutable audit independently reconstructs twelve composed source files, the LF suffix and the refreshed live compatibility hash list. It verifies 214 non-result, 1,038 receipt and 707 preserved-base pins with no mismatches or unexpected production files.

Sydney explicitly transferred tasks 309/576 and the entire owned `O:/xterm-rust-swarm-16/tmp/worktrees/Sydney576` branch, cache, physical reference and runtime artifacts to Lydia. No other worktree, cache, reference or credential is included. No remote push or dispatch permission was inherited.

## Completed local quality checks

The retained final Windows Rust/Cargo 1.85.1 results are under `results/integration576/final-quality/`:

- Native workspace all-target/all-feature tests passed 2,281 tests across 554 summaries, with zero failed, ignored, measured or filtered tests.
- Two doctests passed across five summaries.
- Warning-denied native workspace all-target/all-feature Clippy passed.
- Warning-denied WASM workspace all-target/all-feature Clippy passed.
- Explicit WASM workspace all-target/all-feature check passed.
- Exact aggregate `cargo fmt --all -- --check` failed with Windows OS206. Five whole-member format checks passed separately. They do not waive the aggregate failure.

The first two observed Clippy runs lost their original raw logs when a controller reused output names. `earlier-clippy-observations.json` labels them as observations. Final same-source cached reruns have retained raw logs and snapshots. Output overwrites are now refused by the temporary controller.

The accepted 584 hosted-root change preserves every target, command and the 512-character non-file reserve. The final 554-root model fits at `D:/q` with 243 characters remaining after the reserve; the old `D:/a/q` model exceeds the limit by 865. This is a model, not a hosted formatting pass. Final task 304 remains open.

## Browser and unchanged original checks

All final runtime checks used the actual Rust WASM engine and the matching 2,001-input digest `3254bda5ec7cb216604596802b691b24aa9d75e12905cadd0bfb0c576872c24b`. The production debug build is retained under `results/integration576/production-build/`; no rebuild was done just to replace valid artifacts. Built JS SHA-256 is `4d27a5f8927fce072ec7abee87318c4c7ac8f5b0ed931a0e125c7f1bdc264fe4`, WASM `1010bbc0eb2da8b7234c317ad6eb77c572c19dc1b93f8b89b731786492eb2aba`.

| Check | Actual result | Retained evidence |
| --- | --- | --- |
| Thirteen component suites | 33 functions per browser, 99 total passed, no page/console errors | `results/integration576/browser/` |
| Canonical image component suite | Chromium 87 passed, Firefox 86 passed and 1 failed, WebKit 87 passed | `browser/image-canonical/` |
| Unchanged original image private-accessor test | 3 discovered/executed/passed, zero failed/skipped/unrun | Original run `2026-11-06T20-09-33-298Z-0a67330c-1910-4501-b731-378626486a4e` |
| Full unfiltered original core matrix | 1,434 discovered, 1,194 executed/passed, 240 original skips, zero failed/unrun | `original-core/` |
| Serializer directed/seeded corpus | Chromium 96/96, Firefox 95/96, WebKit 96/96 | `original-serialize/raw/` |
| Unchanged original serializer suites | Source 120/120 and Rust 120/120, zero failed/skipped/unrun | `original-serialize/raw/source-suite/` and `rust-suite/` |

Paths in the last column are relative to `results/integration576/` unless marked as original run IDs under `results/e2e/upstream/`. The serializer retention controller copied its existing output directory, so `original-serialize/retention.json` distinguishes 16 current-run files from 22 inherited historical copies verified against immutable Git bytes. Those historical logs and before-* reports are not new executions or additional passes. Core and serializer ran with the original assertions, two workers and zero retries. The 32 seeded serializer traces retain seed `0x05e21a11`. All 776 original reference files and all 776 physical-copy files matched before and after both final runs. Cargo input identity stayed unchanged.

The image Firefox failure remains the exact native TypeError message comparison: Rust reports `this._opts.pixelLimit`, while source reports an inlined object literal. The serializer Firefox failure is `html-raw-row-subtraction-native-errors`: Rust reports `endRow` and `startRow`, while warmed source reports its object literal. Error types and other phase/identity observations match, but these exact message assertions fail. Neither result was normalized, suppressed or counted as a pass. Full image 311/324 and serialization 346/322 remain open.

Two earlier setup/controller failures are retained separately. The marker suite passed before a result-retention wrapper failed; recovery is labeled recovery-time identity, not a saved pre-run snapshot. An initial direct image invocation omitted its required source globals and failed badly. The corrected canonical setup used identical compiler WASM, served WASM and bindings. Neither failed attempt was hidden or presented as the final valid matrix.

Original generated `client-bundle.js` was reused only after matching accepted 554's byte count and hash. Its optional separate-map copy failed after the JS copy; no separate or inline map exists. No fresh original build is claimed. Physical dependency-copy manifests retain exact hashes and no installation, junction or authentication copy occurred.

## Preservation and reproduction

`generated-current/` retains Sydney's exact generated outputs and binary patch. `generated-after-originals/` retains Lydia's fourteen changed result files, patch and hash receipts before restoring only those generated files to exact HEAD bytes. The historical main build pointer is restored, not replaced by the temporary package's pointer. Runtime package/binary paths remain temporary and become historical after guarded cleanup.

From an owned checkout with the matching package and installed reference dependencies:

```sh
CARGO_BUILD_JOBS=2 node scripts/e2e-upstream.mjs --suite core --no-build
SOURCE_ROOT=/owned/source_repo node scripts/serialize-browser-check.mjs --suite
node results/integration576/audit.mjs
node scripts/worktree-merge.mjs
```

The serializer command currently returns nonzero for its retained Firefox diagnostic mismatch. The canonical helper is unchanged and must complete before main advances. Raw finalization controller text, commands, logs, source/reference checks and exact reports are retained in `results/integration576/`. Independent review, the actual canonical gate and owned cleanup are recorded separately when performed. None is claimed by this pre-gate document.
