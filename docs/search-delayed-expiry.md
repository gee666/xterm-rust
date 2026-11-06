# Search cache access before a delayed timeout

Task 552 fixes one cache expiry phase. Full search, orphan-cache ownership in 527, raw matching in 347 and the complete Search/Serialize family in 322 remain open. Full-port equivalence is NOT PROVEN.

## Source behavior and repair

`SearchLineCache.initLinesCache` allocates rows only when absent and updates the last-access timestamp. It does not expire rows merely because the previous deadline has passed. The scheduled timeout compares elapsed time when its callback actually runs. A search that runs before an overdue callback extends the existing cache lifetime.

Rust `SearchLineCache::init` called `expire` before updating access. A throttled or otherwise delayed host timeout could therefore make Rust discard cached rows before source does. The fix removes that eager expiry. Explicit `expire` and event-driven invalidation remain unchanged. No timer ownership, API, dependency, unsafe code or JavaScript matching implementation changes.

The browser fixture warms a cache, then uses ECH to erase cells without changing the cursor, line feed or dimensions. This leaves the source cache intact and makes an unexpected rebuild observable. It virtualizes only cache timeout delays and the two host clocks, not terminal parsing or rendering. The same fixture runs unchanged against the source Terminal/SearchAddon and the actual Rust public bindings.

Six scenarios cover both search directions and access one millisecond before, exactly at and well after the initial deadline. Each checks retained results, one existing timer, exact rescheduling from the latest access, final callback expiry exposing erased cells and disposal cleanup. A direct native cache function expands three deadline scenarios; the real native addon function expands the same six cases. Existing source assertions and retained corpus are unchanged.

## Verification status

Source frozen at `7a1c96e0`, based on integrated main `224d1b33`. Only the source-runner case-count map conflicted during rebase; both the existing 118-case receiver fixture and the new six-case fixture remain registered. No private prerequisite copies were added. Accepted 544 archive `05a4ee6f` and all 88 durable pins were independently checked without rerunning its gates or cleanup.

Native positive/control, unchanged-source browser, actual old/final WASM, Clippy and corpus checks are pending. Whole xterm-core/xterm-web formatting and changed-file whitespace checks passed before execution. This section will be updated with actual receipts, not planned counts.

## Reproduce

Use an owned O: worktree/target and literal O: TEMP/TMP/TMPDIR, the current coordinated resource grant, Cargo `-j2`, Node 24.15, Rust 1.85.1 and wasm-bindgen CLI 0.2.100. Set SOURCE_DEPENDENCIES and PLAYWRIGHT_MODULE to the existing source dependency tree. Never restore the retired build queue.

```sh
cargo test --locked -j2 -p xterm-core --test search_cache --test search_integration --test search_differential --test search_ring --all-features
cargo test --locked -j2 -p xterm-web --test search_addon --all-features
SEARCH_CASES=search-delayed-expiry node scripts/search-call-args-source.mjs
WASM_TEST=wasm_search_delayed_expiry CARGO_FEATURES=acceptance-hooks node scripts/browser-host-check.mjs
```

The old-code control replaces only `crates/xterm-core/src/search/cache.rs` with exact main224d bytes in the owned worktree, then restores the fixed bytes in finally and verifies their SHA256. It must not run in Integration or the original source tree.
