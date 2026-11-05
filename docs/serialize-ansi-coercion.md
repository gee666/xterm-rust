# Nonnumeric ANSI scrollback allocation and missing-line errors

Tasks 521 and 526 repair two parts of raw serialization task 346. Full 346/322 and overall equivalence remain NOT PROVEN.

## Source contract and repairs

The original `_serializeBufferByScrollback` reads buffer length and terminal rows, evaluates raw `scrollback + rows`, clamps that result and computes start/end coordinates. ANSI allocates `new Array(end - start)` before traversal. Boxed primitives, strings, arrays and objects can produce NaN, fractional or zero-retained counts. Those counts must reach the browser's allocation check, with its native RangeError message, before late addon option getters run. Undefined scrollback still traverses the entire buffer.

Accepted task 498 fixed primitive numbers but left nonnumeric inputs on an integral-coordinate wrapper. Accepted 507 factored raw addition/clamp into `scrollback_row` for HTML without changing ANSI. Task 521 uses that shared arithmetic for ANSI, then calls the existing `line_range` allocation validator for every input type. It deletes the unused integral/fixed-message wrapper. HTML traversal and native serializers are unchanged.

Preliminary independent review found a separate failure in the initial successful reentry control. Reset/shrink invalidated captured endpoints; source then read `isWrapped` from an absent next line. Rust constructed V8-specific TypeError text through `serialize_addon/view.rs`. The control was not omitted. It became the dedicated `ansi-scrollback-captured-missing-line` case, with two additional explicit past-buffer ranges. The original preliminary test remains in commit `ec2b22ac`.

Task 526 changes only browser error construction. Rust's `MissingWrappedLine` variant still selects the same failure point. A small WASM host helper deliberately evaluates `nextLine.isWrapped` with `nextLine` undefined and returns the caught native error object. Keeping the source expression matters because Firefox/WebKit include it in diagnostics. This is host error adaptation, not JS terminal-engine delegation. Native Rust errors and traversal remain unchanged.

No dependency, unsafe code, source-reference, assertion or AGENTS changes were made.

## Regression corpus and observed results

The branch starts on main `1106d480`, which already contains 498. Gordon ACKed private copies of 507's exact five commits. Those copies are prerequisites only, not part of the 521/526 producer chain. Their originals remain canonical. The union retains all 85 inherited cases, including 498's three and 507's four.

Five new directed cases bring the corpus to 90 cases, 58 directed and 32 seeded. Four allocation cases cover 21 raw value kinds in normal and alternate buffers, default-hint/getter ordering, nested serialization and reset/resize during coercion, seven thrown-sentinel phases, and full public-cell snapshots. The successful allocation reentry control grows the reset buffer so captured endpoints remain traversable. The fifth case retains the missing-line reentry and two explicit ranges, including native TypeError prototype/constructor checks. The existing seed is `0x05e21a11`, with 12 operations per seeded trace.

Actual browser results use Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4:

| Phase | Chromium | Firefox | WebKit | Unchanged original source suite | Unchanged original Rust suite |
| --- | --- | --- | --- | --- | --- |
| Baseline | 87/90 | 86/90 | 86/90 | 120/120 | 120/120 |
| Fixed | 90/90 | 90/90 | 90/90 | 120/120 | 120/120 |

The three allocation cases fail before the repair in every engine. Missing-line diagnostics additionally fail in Firefox/WebKit. Fixed matches all 90 raw source entries in every engine. The seven thrown-sentinel identity controls already matched before repair and remain intact. All 85 inherited Rust entries are unchanged between phases, and all 90 source entries are stable. The complete baseline runner exits 1; fixed exits 0. No mismatch filtering, normalization, assertion changes, retries or timeout changes were used. Both phases discover, execute and pass all 120 original cases per implementation, with zero failed, skipped or unrun cases. Neither phase reports cleanup errors.

Raw traces, unchanged-suite reports, screenshots and runner/build identities are under `results/serialize-ansi-coercion/{baseline,fixed}/`. `summary.json` independently rechecks every trace. The source-only Node replay in `source-node.json` uses the actual original headless engine and compiled original addon. It supplements, but does not replace, browser/Rust proof. `preliminary-reviews.md` records how the missing-line failure was found and retained.

## Build identity and quality scope

Both actual WASM builds have `fresh=false`, distinct WASM hashes and 1,811 on-disk build inputs. Baseline swaps the three owned production files to `ec2b22ac`; HEAD remains on fixed during controlled swaps. The on-disk source hashes identify what Cargo compiled:

- Baseline source SHA-256 `9a2e2a8b88a42466b42bb3033e8ad3945aedddd3def46aaa321324fd753e6e86`.
- Fixed source SHA-256 `38ed97813604607bb05a4b0e81c5943ff64d9f6b98d8697d3762a61747e25c86`.
- Baseline WASM SHA-256 `1b18a506bc8a9a6fc8186e8bd74f0935204412b0a850b78f2f86436e92ce1207`.
- Fixed WASM SHA-256 `7a01c760b0e6d36a28a66c289ec3f4c9d221e24f1989dafc5deb45ce5086e033`.

Package manifests hash all 71 baseline and 72 fixed files, including inline JS snippets. Baseline lacks the new error helper; fixed pins its exact source expression. Proof verifies the entire installed package before execution and the final audit verifies it afterward. `pins.json` records 76 raw artifact identities, five source/runner identities and all 72 fixed package files. Both original and physical-copy reference manifests match all 776 files before and after execution.

Quality completed on Windows with Node 24.15.0, Rust/Cargo 1.85.1 and wasm-bindgen 0.2.100. All 16 native serializer executables passed 69 tests, zero failed/ignored. Strict scoped native core/library/tests and actual WASM library Clippy passed. All five package format checks passed. Exact `cargo fmt --all -- --check` failed with Windows OS206, retained in `fmt-workspace.log`; individual package checks do not replace that failure. Commands/outcomes and executable identities are in `{quality,native-binaries}.json`. The native generated HTML TSV difference is preserved in `native-html-output.patch` before cleanup restoration.

This work does not claim full workspace/all-features/release/native Linux/macOS or complete browser equivalence. Debug package sizes are build evidence, not matched release comparison results.

## Reproduction and resource lifecycle

Use an owned O: worktree, private target/TEMP/TMP/TMPDIR, Cargo -j2, Rust 1.85.1 and wasm-bindgen 0.2.100. Set SOURCE_ROOT to the unchanged reference with installed Playwright dependencies. Obtain compiler and browser allocations before execution. Run from that worktree root:

```sh
node results/serialize-ansi-coercion/reproduce.mjs quality
node results/serialize-ansi-coercion/reproduce.mjs build
node results/serialize-ansi-coercion/reproduce.mjs packages
node results/serialize-ansi-coercion/reproduce.mjs proof
node results/serialize-ansi-coercion/reproduce.mjs audit
```

The helper reuses the existing source/Rust serializer runner. It swaps only the three owned production files, stages exact packages, verifies identities and failure names, and restores source/package state in finally paths. It rejects invocation from another working directory or main/legacy master. `wrong-root-guard.json` retains an intentional canonical-root invocation failure before any compiler/source swap, with canonical status unchanged.

Compiler grant 4058 started at 4060 and fully released at 4066. Browser grant 4117 started at 4126 and fully released at 4138. Zero jobs remain. Independent final review and process/link-safe owned cleanup follow separately. Full 346/322 remains open.
