# Ordinary search engine term guard

Task480 follows accepted469. Full search347, family322 and full-port equivalence remain NOT PROVEN.

## Contract and change

The original addon checks `term && term.length > 0` before calling `findNextWithSelection` or `findPreviousWithSelection`. The engine then checks `!term || term.length === 0` again. These are distinct observable property reads and distinct predicates. A raw term can pass the addon check and then return numeric zero, reactivate the addon, reenter search or throw from the engine read.

`SearchTermSource::is_valid_for_engine` represents this second guard. Native text retains the same nonempty predicate. The browser implementation reads raw length with strict equality, without converting the value. Matching and terminal traversal remain in Rust. No dependency, unsafe code or JavaScript search implementation was added.

The ordinary selection helper captures the engine host before this read. An engine-empty term clears that captured host but does not execute the addon's invalid-term branch. It skips matching flags, text conversion and cache initialization, then still evaluates selection decorations and internal noScroll. Result reporting and after-search continue normally. A thrown guard value propagates unchanged before selection clear or any of those later operations. A guard callback that activates another terminal and returns zero clears the old engine host first, then the current selection host during result selection.

Matching text acquisition now follows selection clear, cache initialization and matching-option access. This preserves the failure cutoff for nonzero raw terms whose matching representation is unsupported. It does not implement their boxed or overridden matching methods.

## Verification scope

The shared `crates/xterm-web/tests/fixtures/search-engine-guard.js` has92 directed scenarios per engine, inside one Rust/WASM test function. It covers both directions, numeric positive/negative zero, nested search, activation during the guard, six exact thrown identities with and without activation, late decoration/noScroll throws after an engine-empty result, invalid-addon controls and primitive hit/miss controls. Eleven nonzero/non-numeric lengths and one coercion-trapping length object check that strict engine validation does not coerce them. They subsequently fail matching with TypeError; this test does not assert general raw-term error-message equivalence or successful raw matching.

Three new native functions expand10 scenarios over real terminal selection, decoration and clock services. They check retained highlights, guard-before-clear error ordering, no matching/cache work for empty guards and text failure after selection/cache/options. Existing56 search functions remain unchanged.

`results/search-engine-guard/` records commands, logs, source hashes, inheritance verification and exact restoration hashes. All38 accepted469 summary pins and both776-file reference manifests were verified before reuse. The old469 O: root was absent. New temporary work is owned under `O:/xterm-rust-swarm-16/tmp/R480`.

Observed native Windows results:

- Replacing only `search/find.rs` with accepted469 bytes made all three new native functions fail. Zero passed,56 filtered.
- Exact SHA-256 restoration was followed by59 passing native functions, none failed, ignored or filtered.
- Scoped native and WASM Clippy with `-D warnings` passed.
- Exact `cargo fmt --all -- --check` failed with Windows OS206. This is not a full formatting pass.

- The shared92 cases passed against unchanged original source in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4,276 total.
- Actual Rust/WASM negative control replaced only `search/find.rs` with accepted469 bytes. Chromium passed17 functions and failed the new guard function. A `finally` restored the exact fixed bytes.
- The final actual Rust/WASM suite passed18 functions in each of the three engines,54 total, with no failed, ignored or filtered tests. The92 shared cases are inside one function per engine, not92 extra functions. Both compiler artifacts contain the exact shared fixture bytes; their compiler/WASM/binding hashes were verified before cleanup.
- Changed Rust formatting passed. Node24.15.0 and Rust/Cargo1.85.1 were used on native Windows; wasm-bindgen CLI0.2.100 matched the locked runtime.

Independent review is pending at this checkpoint. Full workspace Clippy/tests, unchanged original Search E2E, seeded corpus, demo, native Linux/macOS and GC were not run for this task.

Reproduce with prepared external tools and a private O: target:

```sh
cargo test --locked -j2 -p xterm-web --test search_addon
cargo clippy --locked -j2 -p xterm-addons -p xterm-web --lib --test search_addon -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_search_addon -- -D warnings
SEARCH_CASES=search-engine-guard node scripts/search-call-args-source.mjs
WASM_TEST=wasm_search_addon node scripts/browser-host-check.mjs
```

Use `SOURCE_DEPENDENCIES`, `PLAYWRIGHT_MODULE` and matching `WASM_BINDGEN`0.2.100 settings for tools outside the checkout. Coordinate heavy jobs with task309 and clean owned temporary outputs after preserving source and evidence.

## Remaining work

Highlight engine-entry validation and repeated highlight `find` calls still snapshot the matching text. Ordinary nonempty raw-term methods, matching flag short circuits and per-line getter phases remain347, as does nonempty engine reactivation with captured line-cache ownership. Task480 only closes ordinary empty/error engine-entry guards and their failure cutoffs. These scoped results do not prove full SearchAddon or full-port parity. Only new480 commits after `archive/Gracie469-complete` are a pickup candidate. Its private prerequisites are not a canonical integration recipe.
