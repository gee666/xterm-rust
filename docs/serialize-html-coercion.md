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

Four shared original/Rust probes cover22 raw input kinds, including boxed NaN/fractions, string concatenation, custom primitive hooks, null/booleans, arrays, Symbol and BigInt. Further probes retain default-hint/getter logs, nested serialization during reset/resize, exact thrown-object identity at six phases, captured row/length/column behavior, full public-cell snapshots and warning reset/throw reentry. All assertions compare exact outputs without message normalization or filtering.

Browser baseline/fixed execution is pending its resource grant. Both actual debug WASM packages are built, with `fresh=false`, distinct inputs and artifact hashes. The baseline swaps only the two production files to their exact bytes at `cddae66b`; the probes remain identical. The reproduction helper stages each package and matching build pointer privately, restores source on success/failure, and checks content identities before execution. No failure-path execution is claimed yet.

All16 existing native serialization binaries pass69 test functions. Warning-denied core library/serializer-test Clippy and actual WASM web-library Clippy pass. All five member formatting checks pass. Exact `cargo fmt --all -- --check` fails with Windows OS206 and remains retained, not waived. Full workspace/all-feature/release, native Linux/macOS and other browser suites are unrun here. Quality logs and native binary hashes are in `results/serialize-html-coercion/`.

## Reproduction

Use an owned O: worktree, a private Cargo target and TEMP/TMP/TMPDIR under its `tmp/`, Rust/Cargo1.85.1, Node24.15.0 and wasm-bindgen0.2.100. Set `SOURCE_ROOT` to the unchanged source with installed Playwright dependencies and generated demo bundle. Set `WASM_BINDGEN` to the matching CLI. Obtain separate current compiler/browser resource allocation first. Each helper command checks at least8GiB RAM and10GiB temporary-volume free space and bounds subprocesses.

```sh
CARGO_BUILD_JOBS=2 node results/serialize-html-coercion/reproduce.mjs build
CARGO_BUILD_JOBS=2 node results/serialize-html-coercion/reproduce.mjs quality
node results/serialize-html-coercion/reproduce.mjs proof
node results/serialize-html-coercion/reproduce.mjs audit
```

`build` prepares baseline/fixed packages sequentially. `proof` uses those exact packages without invoking a compiler, runs all82 shared cases in three browsers and the unchanged original120-case source and Rust suites for each phase, and copies only current receipts. `audit` verifies raw source stability, exact fixed traces, all78 inherited Rust traces unchanged, reference manifests and source/artifact pins. Commands must actually pass before their results can be claimed. Review and owned-worktree cleanup follow the complete proof.
