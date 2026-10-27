# Search engine port

Task #36 implements `crates/xterm-core/src/search/`. Task #59 still owns SearchAddon, SearchResultTracker, DecorationManager, terminal subscriptions and browser/WASM integration. The portable search engine is implemented and tested. The full search addon is not complete.

## Contracts

`SearchBuffer` borrows actual Rust `BufferLine` rows, including blank viewport rows and scrollback. Its column count is positive and every row has that width. `from_rows` accepts references held by live ring-buffer borrow guards, without cloning cells. Buffer positions and search terms use UTF-16 code units, including lone surrogates.

Engine methods return `Result<Option<SearchResult>, SearchError>`. Invalid regexes and columns preserve upstream throwing paths through errors. Selection positions are explicit snapshots. The addon adapter must clear terminal selection before selection-based search and then apply the returned match. Results have signed coordinates and spans deliberately. Source partial matches inside multi-unit combined cells can produce negative coordinates, which must not underflow a Rust integer. The differential corpus retains these cases rather than silently normalizing them.

Wrapped traversal is iterative instead of recursively growing the stack. Searches skip nonexistent negative rows after checking the initial row, preserving observable matches while avoiding an enormous empty scan for `isize::MIN`. Dimensions are validated. Manually supplied malformed cache offsets return `InvalidCacheEntry`, rather than causing an indexing panic. The source cache is internal, so this Rust validation does not change an upstream public API.

`SearchLineCache` uses a caller-supplied monotonic millisecond clock. `init` refreshes last access; `expire` implements the 15,000-millisecond inactivity threshold. Adapters must invalidate on line feed, cursor move and resize, schedule expiry from their host timer and unsubscribe on disposal. Tests of explicit invalidation are not tests of automatic terminal subscription wiring. Task #59 owns that work.

## Dependency decision

Search requires ECMAScript regexes with lookarounds, backreferences, UTF-16 indexing and non-Unicode surrogate-half matching. Rust's `regex` crate does not implement these contracts. `regress =0.10.4` targets ECMAScript and provides a UCS-2 matcher. Default features are disabled; `std`, `utf16` and `prohibit-unsafe` are enabled. The package is MIT OR Apache-2.0, with its selected MIT notice retained in `licenses/regress-MIT.txt`. Its Rust 2024 edition compiles on the declared Rust 1.85.1 toolchain. This dependency choice alone does not prove full JS regex equivalence.

Literal insensitive matching uses Unicode lowercase, not case folding. Whole-string lowercase preserves contextual Greek sigma and expanding mappings. Invalid surrogate units separate independently lowercased valid runs. Tested Unicode behavior is recorded in the corpus; other Unicode-version differences remain a differential-testing concern.

## Commands and observed evidence

Run from the repository root with the reference dependency tree installed:

```text
node scripts/check-search.mjs
```

The script writes exact commands, statuses, tool versions, raw stdout/stderr, source and fixture hashes to `results/search/verification.json` and neighboring logs. Git Bash in this launch environment needs `/c/Users/user/.cargo/bin` prepended to PATH; that is not a requirement on other hosts.

Individual commands are:

```text
node scripts/search-baseline.mjs
node scripts/search-oracle.mjs
cargo test -p xterm-core --test search_differential --test search_cache --test search_integration
cargo clippy -p xterm-core --test search_differential --test search_cache --test search_integration -- -D warnings
```

`search-oracle.mjs` verifies committed fixtures. Use `--generate` only for a deliberate fixture update, inspect the diff and rerun Rust before accepting it. The observer is appended to a generated temporary bundle. Original test bodies, imports and source files remain unchanged. The observer records inputs before the source method and returns the unchanged source result or throws the original error.

Observed native Windows execution on Node 24.15.0 and Rust/Cargo 1.85.1:

| Evidence | Count | Result |
| --- | ---: | --- |
| Unchanged original SearchEngine tests | 58 | Passed |
| Unchanged original SearchLineCache tests | 28 | Passed |
| Rust tests | 26 | Passed |
| Engine call fixtures, original assertions plus fixed and randomized cases | 2,200 | Compared equal |
| Wrapped translation fixtures | 6,116 | Compared equal |
| Optional-state fixtures | 1,024 | Compared equal |
| Scoped Clippy with warnings denied | 1 invocation | Passed |

Seed `0x5ea2c001` generates 2,048 engine cases over actual source `BufferLine` cells and 1,024 state cases. Fixed cases cover every source whole-word delimiter, non-delimiters, rejected first whole-word candidates, contextual/expanding lowercase, regex lookarounds/backreferences/anchors/overlap/empty and invalid patterns, combined cells and surrogate halves, width-two wrapping and selection expansion. Fixtures retain every input and expected output. Rust tests also use actual `CoreTerminal` writes, ANSI parsing, Unicode width, wrapped buffer cells and ring-row references without copying cells. The result checks do not claim browser selection or decorations work.

All 58 original engine tests call the observed engine, and their exact return values or errors are replayed against Rust. Of the 28 cache tests, 10 translation tests are replayed and 13 cache/type operations have direct Rust assertions. Two disposal tests check only portable cache clearing in Rust; actual listener/timer disposal is partial. The three automatic terminal event invalidation assertions remain unported under task #59. Thus the cache suite is not reported as wholly migrated.

The first scoped Rust run passed the earlier 2,118 engine / 6,044 translation corpus. The retained expanded corpus above then passed. Several intermediate compilation attempts failed while other agents changed parser and terminal interfaces; logs under `tmp/search-*.log` are not success evidence for those attempts. The final scoped checker retains its own exact results under `results/search/`.

A workspace `cargo fmt --all -- --check` attempt failed on unformatted files owned by other active tasks. The search-owned formatting check and scoped Clippy pass separately. Workspace-wide fmt, all-target/all-feature Clippy and tests remain project integration gates, not inferred from this scoped result.

## Remaining scope

See the branch checklist in `docs/acceptance-search.md`. Task #59 must implement actual result tracking, public addon activation, selection and scroll mutation, decorations, debounce/event ordering, cache timer/subscription disposal and browser/WASM binding behavior. Original DecorationManager tests and the addon browser suites still require Rust execution. Chromium, Firefox, WebKit and native Linux/macOS runtime results have not been observed for this search implementation. Independent review is required before accepting task #36. Full equivalence remains NOT PROVEN.
