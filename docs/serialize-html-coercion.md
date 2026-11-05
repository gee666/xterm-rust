# HTML scrollback coercion

Task507 repairs a scoped part of raw serialization task346. Full346/322 and overall equivalence remain NOT PROVEN.

## Source behavior

The original `_serializeBufferAsHTML` captures the active view and length, reads `scrollback`, then evaluates `scrollback + terminal.rows` before its numeric clamp. Objects use the default primitive hint. String results concatenate before conversion by `Math.min`. A resulting NaN makes the row comparison false, but does not skip header getters. A fractional starting row uses the existing numeric CircularList lookup. Throwing getters, coercion hooks and header hooks must retain their original JavaScript value and evaluation order.

Previously only primitive NaN and primitive fractions selected the correct HTML traversal. Strings and boxed/custom objects producing those same numbers went through the ANSI integral/error helper. This could raise a RangeError before the HTML header, discard fractional ring aliases, or replace a header exception.

The repair factors the existing addition/clamp into `options::scrollback_row`. HTML chooses empty, fractional numeric or integral traversal from the computed row, not the input's primitive type. ANSI's existing wrapper retains its integral/error behavior. The portable Rust HTML row/cell implementation is unchanged, and no terminal logic delegates to JavaScript. Existing host operators supply only raw value addition/coercion. No dependency, unsafe code, reference or AGENTS changes are made.

## Scope and integration

Base is main `e64f4870`. Probe commit `cddae66b` and production/helper commit `53f6a0c7` are owned. No private prerequisites were replayed. Accepted498 remains a separate NEXT producer and must not be repeated. Its primitive ANSI branch calls the existing `line_range`, which this patch leaves untouched. This producer retains all78 cases from current main, including487's HTML probes, and adds four cases for a total of82. A later union with498 must retain its three ANSI cases too and run85 cases, not claim the old81-case union.

Explicit raw range objects, arbitrary host method replacement, palette-array reentry, broader ANSI numeric/coercion and full GC/lifetime proof remain346 work. This repair does not change those contracts or claim their completion.

## Verification

Four shared original/Rust probes cover22 raw input kinds in both normal and alternate buffers, including boxed NaN/fractions, string concatenation, custom primitive hooks, null/booleans, arrays, Symbol and BigInt. Further probes retain default-hint/getter logs, nested serialization during reset/resize, exact thrown-object identity at six phases, captured row/length/column behavior, full public-cell snapshots and warning reset/throw reentry. All assertions compare exact outputs without message normalization or filtering.

Baseline matches77/82 cases in each of Chromium148, Firefox150 and WebKit26.4. Fixed matches81/82 in each. All four new cases fail in the baseline and match source exactly after the repair. All78 inherited Rust traces remain byte-for-byte unchanged, and all82 source traces are stable across phases. The unchanged original suite passes120/120 against source and120/120 against Rust in each phase, with zero failures, skips or unrun cases,40 per engine per implementation. Both reference manifests verify776 files without mismatches.

The complete browser runner honestly remains failed, exit1. Its sole fixed mismatch is the inherited `html-fractional-scrollback-clamps` probe from487, which also compares ANSI RangeError messages. This branch lacks accepted498, so that already-repaired separate ANSI defect remains visible here. It is not a new HTML failure, and the raw probe is neither omitted nor normalized. The initial helper incorrectly assumed all78 inherited cases already matched. Its guard stopped after the baseline, after all three engines and both original suites had completed. That policy failure and all raw receipts remain retained; fixed then ran once against the identical corpus. The scoped audit now explicitly requires exactly this unchanged inherited failure and exact matches for the other81 cases. It does not turn the full runner green.

Both actual debug WASM packages report `fresh=false`. Baseline WASM hash is recorded in `baseline/build-identity.json`; fixed begins `1e395446`, with1791-file input hash `a79ddda6`. The baseline swaps only the two production files to their exact bytes at `cddae66b`; the probes remain identical. The reproduction helper stages each package and matching build pointer privately, restores source on success/failure, and checks content identities before execution. The actual baseline policy failure exercised the finally path, but no separate injected build/browser failure or universal cancellation guarantee is claimed. Both full input identities are independently rederived by the audit.

All16 existing native serialization binaries pass69 test functions. Warning-denied core library/serializer-test Clippy and actual WASM web-library Clippy pass. All five member formatting checks pass. Exact `cargo fmt --all -- --check` fails with Windows OS206 and remains retained, not waived. Full workspace/all-feature/release, native Linux/macOS and other browser suites are unrun here. Quality logs and native binary hashes are in `results/serialize-html-coercion/`.

## Reproduction

Use an owned O: worktree, a private Cargo target and TEMP/TMP/TMPDIR under its `tmp/`, Rust/Cargo1.85.1, Node24.15.0 and wasm-bindgen0.2.100. Set `SOURCE_ROOT` to the unchanged source with installed Playwright dependencies and generated demo bundle. Set `WASM_BINDGEN` to the matching CLI. Obtain separate current compiler/browser resource allocation first. Each helper command checks at least8GiB RAM and10GiB temporary-volume free space and bounds subprocesses.

```sh
CARGO_BUILD_JOBS=2 node results/serialize-html-coercion/reproduce.mjs build
CARGO_BUILD_JOBS=2 node results/serialize-html-coercion/reproduce.mjs quality
node results/serialize-html-coercion/reproduce.mjs proof
node results/serialize-html-coercion/reproduce.mjs audit
```

`build` prepares baseline/fixed packages sequentially. `proof` uses those exact packages without invoking a compiler, runs all82 shared cases in three browsers and the unchanged original120-case source and Rust suites for each phase, and copies only current receipts. Its optional `baseline` or `fixed` argument runs only that phase. The retained run used `proof`, which stopped on the initial overly strict guard after baseline, then `proof fixed` after correcting that guard. No original assertion or runner failure handling changed. `audit` requires the exact81/82 fixed result, four repaired cases, all78 inherited Rust traces unchanged, stable source traces, reference manifests and source/artifact pins. `summary.json` labels the result `scoped-pass` and the full runner `failed`. Review and owned-worktree cleanup follow this scoped proof.
