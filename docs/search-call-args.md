# Search callable arguments

Task344 restores two arguments omitted from the browser binding. The underlying native search and cache already exist.

- `clearDecorations(retainCachedSearchTerm)` passes JavaScript truthiness to the native cache-retention flag. Objects are not converted to primitives. Retaining the cached term changes whether a subsequent same-term search advances beyond the current selection and whether cleared highlights are rebuilt.
- `findNext(term, options, internalOptions)` and `findPrevious(term, options, internalOptions)` read `internalOptions?.noScroll` after engine traversal and selection clearing, immediately before selecting the computed result. Empty terms, use before activation, before-search errors and engine errors skip this read. A missing match still reads the option. Getter errors preserve their JavaScript value identity and prevent selection, scrolling and the after-search event. No buffer, cache or addon state borrow spans this getter, so it can start another search.
- Native callers keep the existing methods. The new `find_next_with_scroll` and `find_previous_with_scroll` methods accept a fallible deferred boolean reader. The existing refresh timer still suppresses scrolling through the same path.

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

## Remaining search contracts

Task347 is the actionable raw-search followup under full-family322. Current `search_addon/options.rs::term` rejects every truthy non-string before native activation and before-search checks. Source `SearchState.isValidSearchTerm` instead evaluates `!!(term && term.length > 0)` during search phases. For example, an activated source addon returns false for `findNext(1)`; an unloaded source addon throws the not-loaded Error before inspecting the term. The current Rust binding throws TypeError first in both cases. Boxed strings, overridden methods, getter-backed length and identity-cached objects require preserving the raw value, not one eager string conversion.

`request(null)` currently treats null options like undefined. Source optional chaining permits the initial decoration test, but `_findInLine` defaults only undefined and then reads `searchOptions.regex`, so null can throw later after selection clearing. Preserve that phase.

`SearchOptionsSource::read` currently takes one eager snapshot of decorations/caseSensitive/regex/wholeWord. The source reads decorations for highlight eligibility, selection arguments and result notification separately. Each `_findInLine` reads regex twice, caseSensitive twice for a plain search, and wholeWord only when a match exists. Repeated same-term highlighted searches compare getters on the same stored options object twice, which can produce different values. Wrapped reverse rows can return before any matching flag read. Refresh uses object spread, including enumeration/property getter order, not the same narrow snapshot. Empty searches should not read matching flags. Replacing these phases needs callback-safe matching/row traversal; do not hold a Rust buffer or cache borrow across JavaScript getters.

Serialization310 owns the first callable serializer producer. Serialization346 retains raw numeric traversal, reentrant host descriptors and lifetime/GC closure. Passing ordinary typed searches or the new callable argument cases does not close those tasks. Source unit assertions, complete final mapping, randomized raw-option traces and native Linux/macOS evidence still belong to the full-family verification plan.
