# Search callable arguments

Task344 restores two arguments omitted from the browser binding. The underlying native search and cache already exist.

- `clearDecorations(retainCachedSearchTerm)` passes JavaScript truthiness to the native cache-retention flag. Objects are not converted to primitives. Retaining the cached term changes whether a subsequent same-term search advances beyond the current selection and whether cleared highlights are rebuilt.
- `findNext(term, options, internalOptions)` and `findPrevious(term, options, internalOptions)` read `internalOptions?.noScroll` after engine traversal and selection clearing, immediately before selecting the computed result. Empty terms, use before activation, before-search errors and engine errors skip this read. A missing match still reads the option. Getter errors preserve their JavaScript value identity and prevent selection, scrolling and the after-search event. No buffer, cache or addon state borrow spans this getter, so it can start another search.
- Native callers keep the existing methods. The new `find_next_with_scroll` and `find_previous_with_scroll` methods accept a fallible deferred boolean reader. The existing refresh timer still suppresses scrolling through the same path.
- New optional WASM arguments use an opaque `SearchArgument` with TypeScript type `any`. The locked wasm-bindgen0.2.100 does not support `Option<JsValue>`; this wrapper keeps raw primitive/object values and optional declarations without runtime type conversion. Null and undefined both skip the optional noScroll property read and are both falsy retention values.

These changes do not repair all raw term/options behavior. Overall search/serialization equivalence remains NOT PROVEN.

## Verification commands

Use Cargo jobs2 and an owned O: scratch directory for TEMP/TMP/TMPDIR. `SOURCE_DEPENDENCIES` can identify the unchanged reference checkout containing the installed Node dependencies, without making a dependency junction. The source runner bundles the unchanged source classes rather than executing the Rust engine or modifying original tests.

```sh
node scripts/search-call-args-source.mjs
cargo test --locked -j2 -p xterm-web --test search_addon
cargo clippy --locked -j2 -p xterm-web --test search_addon -- -D warnings
WASM_TEST=wasm_search_addon node scripts/browser-host-check.mjs
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_search_addon -- -D warnings
cargo fmt --all -- --check
```

Both browser runs use `crates/xterm-web/tests/fixtures/search-call-args.js`. It contains81 directed source-derived cases per engine, not81 unchanged upstream tests. It covers raw truthy/falsy retention and noScroll values, inherited getters, omitted/null/primitive internal options, callback and getter exceptions, no-match/empty/regex-error paths, pre-activation errors and nested search. The native addon target adds three focused hook/error tests. Original search290 evidence is retained on main in `results/search-browser`; these scoped additions do not replace its unchanged81-case suite or establish full-family coverage.

## Observed results

Windows, Rust/Cargo1.85.1, Node24.15.0, wasm-bindgen0.2.100. Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4.

| Scope | Executed | Passed | Failed | Skipped/unrun |
| --- | ---: | ---: | ---: | ---: |
| Shared source call-argument cases | 243 | 243 | 0 | 0 |
| Native search addon test functions | 28 | 28 | 0 | 0 |
| Final optional-export Rust WASM test functions | 21 | 21 | 0 | 0 |
| Shared call-argument cases inside final WASM tests | 243 | 243 | 0 | 0 |
| Unchanged upstream Search browser suite | 81 | 81 | 0 | 0 |

The final WASM suite runs seven test functions per engine. One function contains the81 directed shared cases, so do not add that243-case subtotal to the21-function count. The unchanged original81-case suite ran after the behavioral argument repair but before the final declaration-only opaque wrapper. The final wrapper reran the whole seven-function Rust WASM suite across three engines, including all raw argument cases. Both reports and source/build hashes are retained separately under `results/search-call-args/`. The earlier WASM export report is named `wasm-pre-optional.json`, not presented as the final wrapper run.

Scoped native/WASM Clippy passes with warnings denied. Exact full `cargo fmt --all -- --check` passes from the shorter owned Windows worktree path. Main inherited a module-order formatting failure after marker integration; separate task353 fixes only that order in commit559daa9. Its independent reviewer reran full fmt and diff-check, and the canonical batch integrated that fix at main3baa698. This is not the complete workspace all-feature/release/platform matrix.

The source driver verifies all manifest entries before and after execution. It bundles the unchanged source Terminal and SearchAddon classes. Original tests/assertions/timeouts are not edited. Browser processes and local servers are closed by the bounded runners. The sole owned reference dependency junction is explicitly unlinked after the unchanged-suite run. Generated WASM/bindings and copied build artifacts are disposable, not durable evidence.

## Independent review

Task344 accepted reviewed commit9f3aa4a. The isolated reviewer independently reran native28/28, scoped warning-denied Clippy and exact full fmt. It verified six implementation/test hashes, all776 reference entries, final WASM/compiler/binding artifacts, CSS, Cargo.lock and generated optional declarations. Browser evidence was inspected, not rerun during review. Task353 received separate independent full-fmt and diff-check review before its formatting-only integration.

## Remaining search contracts

Task347 is the actionable raw-search followup under full-family322. At task344's checkpoint, `search_addon/options.rs::term` rejected every truthy non-string before native activation and before-search checks. Later task363 retains raw invalid terms, validates length at addon phases and preserves raw cache identity. See `docs/search-invalid-terms.md` for its scope and evidence. Valid boxed strings, overridden matching methods and additional engine length/accessor phases still need source matching semantics.

`request(null)` currently treats null options like undefined. Source optional chaining permits the initial decoration test, but `_findInLine` defaults only undefined and then reads `searchOptions.regex`, so null can throw later after selection clearing. Preserve that phase.

`SearchOptionsSource::read` currently takes one eager snapshot of decorations/caseSensitive/regex/wholeWord. The source reads decorations for highlight eligibility, selection arguments and result notification separately. Each `_findInLine` reads regex twice, caseSensitive twice for a plain search, and wholeWord only when a match exists. Repeated same-term highlighted searches compare getters on the same stored options object twice, which can produce different values. Wrapped reverse rows can return before any matching flag read. Refresh uses object spread, including enumeration/property getter order, not the same narrow snapshot. Empty searches should not read matching flags. Replacing these phases needs callback-safe matching/row traversal; do not hold a Rust buffer or cache borrow across JavaScript getters.

Serialization310 owns the first callable serializer producer. Serialization346 retains raw numeric traversal, reentrant host descriptors and lifetime/GC closure. Passing ordinary typed searches or the new callable argument cases does not close those tasks. Source unit assertions, complete final mapping, randomized raw-option traces and native Linux/macOS evidence still belong to the full-family verification plan.
