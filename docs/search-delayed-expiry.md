# Search cache access before a delayed timeout

Task 552 fixes one cache expiry phase. Full search, orphan-cache ownership in 527, raw matching in 347 and the complete Search/Serialize family in 322 remain open. Full-port equivalence is NOT PROVEN.

## Source behavior and repair

`SearchLineCache.initLinesCache` allocates rows only when absent and updates the last-access timestamp. It does not expire rows merely because the previous deadline has passed. The scheduled timeout compares elapsed time when its callback actually runs. A search that runs before an overdue callback extends the existing cache lifetime.

Rust `SearchLineCache::init` called `expire` before updating access. A throttled or otherwise delayed host timeout could therefore make Rust discard cached rows before source does. The fix removes that eager expiry. Explicit `expire` and event-driven invalidation remain unchanged. No timer ownership, API, dependency, unsafe code or JavaScript matching implementation changes.

The browser fixture warms a cache, then uses ECH to erase cells without changing the cursor, line feed or dimensions. This leaves the source cache intact and makes an unexpected rebuild observable. It virtualizes only cache timeout delays and the two host clocks, not terminal parsing or rendering. The same fixture runs unchanged against the source Terminal/SearchAddon and the actual Rust public bindings.

Six scenarios cover both search directions and access one millisecond before, exactly at and well after the initial deadline. Each checks retained results, one existing timer, exact rescheduling from the latest access, final callback expiry exposing erased cells and disposal cleanup. A direct native cache function expands three deadline scenarios; the real native addon function expands the same six cases. Existing source assertions and retained corpus are unchanged.

## Verification status

Production and test source frozen at `66c072ce`, based on integrated main `224d1b33`. Only the source-runner case-count map conflicted during rebase; both the existing 118-case receiver fixture and the new six-case fixture remain registered. No private prerequisite copies were added. Accepted 544 archive `05a4ee6f` and all 88 durable pins were independently checked without rerunning its gates or cleanup.

Native Windows execution passed all 106 functions with no failures, ignored or filtered tests. The four core targets passed 19 cache, three retained differential, five integration and eight ring functions. The addon target passed 71 functions. The differential functions retained their assertions for 2,200 engine inputs, 6,116 translations and 1,024 state cases. No corpus regeneration or expected-output changes occurred.

The exact main224d cache failed the designated new core function, zero passes, one failure and 18 filtered, and the new addon function, zero passes, one failure and 70 filtered. The addon failure is the intended `previous=false delayed=15000` retained-cache assertion. Finally restored the exact fixed source SHA256, after which the full focused native targets passed.

The identical shared fixture passed six scenarios in each unchanged-source engine, Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4. Actual old-code WASM in Chromium produced a completed zero-pass, one-failure summary at the same delayed=15000 assertion. The fallible new test returned its error normally, without a secondary test-context panic. After exact source restoration, actual fixed WASM passed its one function in each engine, with six shared scenarios inside that function. No browser page errors or error-console entries occurred in the Rust runs.

Both WASM variants were real rebuilds, `fresh=false`, with distinct compiler and served-WASM hashes. Both compilers contain the exact shared fixture, SHA256 `5435287b79d002172a57756d4e3fc5fdb9e99b473719e398cbf8af110e5851c3`. Both used `acceptance-hooks`. Compiler artifacts, served WASM and generated bindings remain available for independent review before owned temporary cleanup.

Scoped native and WASM library/test Clippy passed with `-D warnings`. All five complete-member formatting checks passed. The exact workspace `cargo fmt --all -- --check` failed with Windows OS206 before formatting, and is not reported as passed. Full workspace tests/release, new original-suite execution, full demo/GC and native Linux/macOS remain unrun for 552. Reference manifests checked all 776 files in each original and physical-copy root before and after the runs; all 1,910 compiled-source/build input pins remain unchanged.

Raw commands, logs, reports, restoration receipts and manifests are under `results/search-delayed-expiry/`. The native allocation was released at coordination post 409/comment 4734. The later browser/compiler/server allocation was released at 409/4749. No 544 gates, cleanup or private prerequisite commits were replayed.

## Reproduce

Use an owned O: worktree/target and literal O: TEMP/TMP/TMPDIR, the current coordinated resource grant, Cargo `-j2`, Node 24.15, Rust 1.85.1 and wasm-bindgen CLI 0.2.100. Set SOURCE_DEPENDENCIES and PLAYWRIGHT_MODULE to the existing source dependency tree. Never restore the retired build queue.

```sh
cargo test --locked -j2 -p xterm-core --test search_cache --test search_integration --test search_differential --test search_ring --all-features
cargo test --locked -j2 -p xterm-web --test search_addon --all-features
SEARCH_CASES=search-delayed-expiry node scripts/search-call-args-source.mjs
WASM_TEST=wasm_search_delayed_expiry CARGO_FEATURES=acceptance-hooks node scripts/browser-host-check.mjs
```

The old-code control replaces only `crates/xterm-core/src/search/cache.rs` with exact main224d bytes in the owned worktree, then restores the fixed bytes in finally and verifies their SHA256. It must not run in Integration or the original source tree.
