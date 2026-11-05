# Equal numeric ANSI row ranges

Task 456 repairs a scoped part of raw serialization task 346. Full search/serialization family 322 and overall equivalence remain NOT PROVEN.

## Source contract and repair

`SerializeAddon._serializeBufferByRange` captures both marker values and the terminal column count before traversing. Equal finite numeric rows allocate a zero-length row array, then traverse one row. `BufferApiView.getLine` passes the raw number to CircularList. Negative, fractional and large numbers can be missing or alias an existing ring slot after rotation. They are not native integer indexes and cannot be rejected or truncated on that assumption.

The browser adapter now sends equal finite rows in the JavaScript safe domain through `serialize_numeric_row`. Its raw lookup, captured-cell reads and diagnostics retain the original number. It shares the existing Rust ANSI handler for cell/style processing, row completion and late attribute restoration. Native integer range serialization keeps its existing API and traversal.

The shared handler uses a local origin for cell processing because explicit ANSI ranges omit final cursor movement and there are no non-final rows or wrapping lookahead. Trimming must still use the original floating row. For example, `3.429227000030392 + 1 - 3.429227000030392` exceeds one, so the source output loop reads an absent second array entry and appends literal `undefined`. The numeric finish path preserves that behavior, then shares ordinary output assembly and late style restoration. The two small traversal loops intentionally differ in integral versus numeric lookup and diagnostic methods; cell/style logic is not duplicated.

The raw helper rejects nonfinite and out-of-safe-domain coordinates. Equal infinite bounds and other nonprogressing source loops remain unresolved, not compatible successful no-ops. Unequal numeric traversal, late object coercion, raw replacement hosts, palette reentry and full lifetime/GC proof remain open under 346/322. No source reference, AGENTS.md, dependency or unsafe code changes are made. Rust remains the terminal engine.

## Regression coverage

Two shared original/Rust browser probes exercise thirteen equal numeric bounds on a rotated two-row ring, colored output, full public-cell snapshots, double marker getter reads and late option getters. Warning probes narrow columns from an end getter, perform nested serialization, reset and resize during the first warning, and compare exact thrown-object identity. Existing `raw-numeric-boundaries` remains unchanged.

Five native tests cover direct and public buffer ring aliases, missing fractional rows, native integral/negative-zero equivalence, explicit rejection of nonfinite and out-of-domain coordinates, raw diagnostic coordinates, late attribute restoration and warning failure propagation.

Baseline production is main `3aa04692` with probe commit `e389f45d`. Fixed production is `30f08e0e`, including the initial repair `7f6b89f0` and floating-trim repair `69578585`. No private 437/446 prerequisites are included or required, and their pending producer chains must not be replayed.

- Baseline source SHA-256 `61c363303888d7ab93499a95bcaa352e904056bbfbea22984707e31ce80a61de`.
- Fixed source SHA-256 `acaf8a10bfe3440d22b693b2d6c141cc1c14efe37cdf0f78e02caf93aff4325a`.
- Baseline build `results/demos/build/2026-11-05T19-26-05-431Z/report.json`.
- Fixed build `results/demos/build/2026-11-05T19-37-34-817Z/report.json`.
- The earlier build `2026-11-05T19-27-19-763Z` is superseded and was never used for browser evidence.

Supplemental source-only Node evidence uses the actual original headless Terminal and compiled original SerializeAddon. All thirteen bounds and exact strings are retained in `results/serialize-ansi-numeric/node-source.json`, including two literal `undefined` outputs followed by late blue style restoration. This is not Rust browser proof.

The baseline matches 63/66 cases in Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4. Both new ANSI probes and the retained `raw-numeric-boundaries` probe fail in every engine. The fixed package matches 66/66 in all three, with no mismatches. The runner honestly exits 1 for the negative control and 0 for fixed.

The unchanged original SerializeAddon suite passes 120/120 source cases and 120/120 Rust cases, with no failed, skipped or unrun cases. Both directed runs verify all 776 source manifest files before and after execution without mismatches. The 32 seeded restoration traces retain seed `0x05e21a11` with 12 operations each. Raw traces, screenshots, source/build/runner hashes and suite reports are under `results/serialize-ansi-numeric/{baseline,fixed}/`. Neither run reports cleanup errors. The baseline does not rerun original suites; the fixed run does.

This branch starts from main `3aa04692` without pending 437/446 HTML probes, so its 66 cases are 34 directed and 32 seeded. Their later union must retain those four HTML probes too and update the runner total coherently. Passing this bounded corpus is not full 346/322 closure.

The baseline source identity includes the initial native fixture bytes, which are not compiled by the library-only WASM build. The fixed identity includes the final fixture with decimal cases. The negative-control helper swaps those exact bytes along with the production sources, package and latest-build pointer, then verifies identities before execution and after `finally` restoration. HEAD remains fixed during this byte-level control; the recorded source SHA and baseline build revision identify the actual baseline.

## Quality scope and reproduction

Windows Rust/Cargo 1.85.1, Node 24.15.0 and wasm-bindgen 0.2.100. All 16 native serialization binaries pass 69 tests with no failures or ignored tests. Warning-denied focused native and WASM library Clippy pass. Web crate formatting and changed core file formatting pass. Exact `cargo fmt --all -- --check` fails with Windows OS206. Full workspace/all-feature/release and native Linux/macOS passes are not claimed.

Use an owned O: worktree with private target/TEMP/TMP/TMPDIR. Set SOURCE_ROOT to the unchanged reference with installed Playwright dependencies, and WASM_BINDGEN to the locked CLI. Obtain current resource allocation before compilers or browsers.

```sh
CARGO_BUILD_JOBS=2 node scripts/build-demo.mjs --debug
node scripts/serialize-browser-check.mjs --suite
cargo test --locked -j2 -p xterm-core --test serialize_ansi_numeric_row --test serialize_ansi --test serialize_native
cargo clippy --locked -j2 -p xterm-web --lib --target wasm32-unknown-unknown -- -D warnings
cargo fmt -p xterm-web -- --check
cargo fmt --all -- --check
```

Initial compiler slot was released in post 409 comment 2943. Follow-up allocation 2988 reran all 69 native tests and strict scoped Clippy after the decimal repair, then was released in comment 2998. Browser allocation 3032 ran the baseline/fixed matrices and original suites, then was released in comment 3036. Review is source/light only unless separately authorized. Existing native executables may be rerun directly, excluding the differential binary if avoiding writes to tracked HTML output.

Native logs, exact formatting failure, source/package/native executable identities and the differential output patch are under `results/serialize-ansi-numeric/quality/`. The bounded swap and supplemental Node helpers are retained as `.mjs.txt` evidence beside `summary.json`. No builds were replayed during swaps. Review and cleanup receipts follow separately.
