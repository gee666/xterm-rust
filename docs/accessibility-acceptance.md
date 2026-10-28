# Accessibility acceptance and ownership

The reference is `source_repo/src/browser/AccessibilityManager.ts` and `source_repo/src/browser/LocalizableStrings.ts`. Browser mode activation additionally follows `source_repo/src/browser/CoreBrowserTerminal.ts`. Task72 owns `crates/xterm-web/src/accessibility/`, `terminal/accessibility_host.rs`, component/oracle tests and actual accessibility DOM tests. Task34 integrates shared selection/viewport services. Task31 owns the terminal coordinator, runtime and static JS strings facade. Component checks do not establish a complete browser terminal.

## A11Y1. Announcements and exact UTF-16 echo

- Preserve every UTF-16 unit, including isolated surrogates, astral pairs, combining text, NUL and BOM. A key is queued as one complete string, including empty and multichar strings. The queue is not a per-codepoint queue.
- A key clears live DOM text and resets only the live line count. Pending announcement data and existing queued keys remain. Blur has the same clearing behavior without adding a key.
- Reject a key from the echo queue if its string contains Unicode General_Category=Control anywhere. The source regex covers U+0000..001F and U+007F..009F, not format characters, BOM, surrogates or arbitrary Rust whitespace.
- Each received string consumes at most one queued key. Equality suppresses announcement; inequality still consumes the key but appends output. Empty strings, multichar keys, astral strings, mismatches and interleaved streaming output must be covered.
- Only a string exactly equal to LF increments the count. Embedded LF within another string does not. At LF21, replace live text with the current localized overflow value. Further input is ignored until the source clears the count. Tab events deliver exactly their number of individual spaces.
- Rendering flushes pending text. An empty pending buffer must not read or write live DOM text or evaluate an overflow label. If current live text strictly equals the current raw overflow value, clear live text and line count first. Then append pending text and clear pending only after the DOM write succeeds.
- Test the overflow label changing between LF21 and flush, including empty labels, numeric/boolean/null/undefined values, objects, throwing coercions and localized lone surrogates. Stringified nonstring labels do not compare strictly equal to their raw values; do not normalize this source behavior away.
- DOM getter/setter or coercion failures preserve the original JS thrown object, the source's partial line count and pending data, and stop subsequent processing. Reentrant getters/setters must not overlap a Rust RefCell borrow. The small JS addition shim implements only JavaScript value coercion; Rust owns all announcement state.
- Native property tests and seeded JS traces compare pending text, queue contents, line count and live text after every operation. Browser tests additionally prove actual DOM coercion, typed callback events and errors. Do not count generated comparisons as migrated upstream tests.

## A11Y2. Row tree, metadata and dimensions

- Direct manager construction without an opened terminal returns `Cannot enable accessibility before Terminal.open`. Setting a terminal's screenReaderMode before open is allowed; the coordinator creates the manager when open creates services. Do not confuse these two paths.
- Attach one `div.xterm-accessibility` before existing root children. It contains the role=list tree followed by the aria-live=assertive live region. Each visible row is a div with role=listitem and tabIndex=-1. Do not add an empty class attribute to rows that lack one upstream.
- Capture the active buffer identity once per render batch, as source does. Read its display offset and lines live for each row; a reentrant reset must not redirect the remaining rows to a replacement buffer. Read line text and UTF-16-to-cell maps from BufferLine. Empty or absent text becomes NBSP with `[0,1]` mapping. Combined/astral/wide cells retain their exact maps. Store map associations in a real weak DOM map so a retained detached row behaves like the source.
- `aria-posinset` is absolute buffer row plus one; `aria-setsize` is captured full line count for that render batch. Check scrollback, alternate buffers, trim, blank rows and resize.
- Set row height from CSS cell height. A dimensions refresh skips only JavaScript-false height values, including zero and NaN, not every negative number. Otherwise update container width/font-size, reconcile row count and update every row's height/alignment.
- Alignment clears transform before measuring the DOM rectangle. If the last mapped column is false/missing, stop. Otherwise use scaleX of last-column times CSS cell width divided by measured width. Retain source CSS parsing for zero width, nonfinite values and absent mappings rather than fabricating a scale of1.
- Resize removes/readds only the bottom focus listener, preserving top listener identity/order. It grows/shrinks real DOM rows, then refreshes dimensions. Source terminal rows are committed before resize delivery. The Rust host rejects an inconsistent private host event instead of indexing sparse memory or panicking.
- Use the existing source-compatible one-second TimeBasedDebouncer with fallible render callbacks. Coalesce/clamp ranges and preserve trailing timer flags on callback error. Disposal cancels owned timers. Do not replace debounce with sleeps or arbitrary polling.

## A11Y3. Boundary focus and DOM selection

- Boundary focus does nothing at the respective buffer limit or unless relatedTarget is the adjacent row. Cover entry from outside, programmatic focus, top/bottom, one-row/two-row viewports and both scrollback limits.
- A handled boundary focus removes the opposite end row, detaches old boundary listeners, creates/inserts a new row at the correct end, reattaches listeners, scrolls one row through the actual viewport-aware terminal method, focuses the new adjacent row, then prevents default and immediate propagation. Assert ordering and node identity, not only final text.
- No selection or an empty disposed tree leaves terminal selection unchanged. Collapsed selection clears terminal selection only when its anchor is inside the tree.
- Missing anchor/focus logs the source error. Sort points by document order and by offset within one node. Clamp begin/end across surrounding DOM exactly. Read boundary rows at the source's access points, including the last row only after comparison and begin-clamping callbacks that may resize the tree. A pre-render row with no child node follows the source contains(undefined) early-return path.
- Map a Text node to its parent using the source module realm's Text instanceof check, not an unconditional nodeType substitution. Cover same-document and iframe nodes explicitly.
- Parse aria-posinset with JavaScript parseInt. NaN or missing column metadata warns and returns without altering terminal selection. Retain weak maps for detached nodes observed during reentrancy.
- Use offset-indexed columns when available; otherwise last mapped column plus one. A column at/past terminal width becomes column0 of the next row once, not arbitrary modulo wrapping.
- Equal or reversed mapped ranges throw `invalid range`. A valid range calls the existing shared SelectionService with exact start and cell count. Test reverse DOM selections, row/text-node endpoints, EOL, wide/combined text, offsets beyond map length, stale attributes, outside-tree spans and clearing.

## A11Y4. Runtime localization and disposal

- Preserve both exact default English strings. All terminals in one binding runtime share one owner, including different documents. Separate explicit runtimes remain independent. The portable core must not own browser globals.
- Public JS string getters/setters retain raw values and identity without coercing on set. Coercion occurs at actual setAttribute/textContent use, preserving thrown values and timing. Rust's explicit typed UTF-16 getters return an error when a raw override is not a string; they must not return stale defaults.
- Prompt labels are applied at textarea construction. Changing the global label later does not rewrite existing textareas. Overflow announcements read the current value at their actual use.
- Mode enable/disable/re-enable maintains exactly one tree and the existing textarea/root identities. Disposal removes only owned DOM, subscriptions, selection listener, boundary listeners and timers. No Closure::forget, strong self-reference cycle, ignored Result or post-disposal browser callback may masquerade as completion.
- Host callbacks share actual terminal buffers, options, dimensions, selection, scroll, onKey, blur and DPR services. No parallel fake buffer or input echo is allowed. Test reentrant write/resize/dispose and callback failure with the real opened terminal.

## A11Y5. Executable evidence

- Inventory existing applicable source assertions. There is no separate AccessibilityManager unit suite in the supplied tree; this is not permission to invent an upstream case count. Relevant browser/IME assertions and additional direct-reference comparisons must be listed separately.
- Original methods and assertions stay unchanged. A reference loader may remove module/type/decorator boundaries or bundle with the reference's esbuild dependency, but record every source hash and adaptation.
- Native tests prove pure UTF-16 state/range contracts. Browser component tests prove DOM behavior. Real exported-Terminal tests prove coordinator/service integration. Record each category separately, with failed/skipped/unrun counts and retained counterexamples.
- Run actual Chromium, Firefox and WebKit against built Rust WASM. Assert DOM, events, errors, focus/selection and cleanup. A WASM compilation, native model test or configured workflow is not a runtime/browser pass.
- Run focused formatting, warning-free native/WASM Clippy, tests and independent review. Keep current shared-build failures and resource pauses explicit. Do not launch builds during the shared disk pause or create duplicate target directories.

## Observed checks

On native Windows with rustc/cargo1.85.1, Node24.15.0 and wasm-bindgen0.2.100:

- `cargo test --locked -p xterm-web --test accessibility_state --test accessibility_differential` passed11 tests; the same command with `--release` passed11.
- `XT_SOURCE_ROOT=<preserved-reference> node scripts/accessibility-oracle.mjs` matched268 traces with32,871 operation snapshots and all65,536 BMP key classifications. Seed10559090. There are no upstream AccessibilityManager unit cases to claim. `XT_SOURCE_ROOT` is optional when source_repo has its own installed esbuild dependency.
- `CARGO_FEATURES=acceptance-hooks WASM_TEST=wasm_accessibility node scripts/browser-host-check.mjs` passed25 Rust DOM/component tests in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4.
- The same runner with `WASM_TEST=wasm_accessibility_terminal` passed5 real exported-Terminal tests in every engine. These exercise real writes and UTF-16 announcements, onKey echo suppression, mode disable/re-enable, shared raw localization, prompt construction and disposal. Two review regressions additionally assert that reset during the first row setter retains the original batch buffer, and resize during selection comparison clamps to the current last row.
- Warning-denied native and wasm32 Clippy passed for the library and these targets. Scoped rustfmt passed. Full-workspace/native-platform gates remain separate.

When using external tooling, set `WASM_BINDGEN` to the matching CLI and `PLAYWRIGHT_MODULE` to the reference's Playwright index.mjs. The runner closes browsers and its server. Source report is `results/accessibility/oracle.json`; browser summaries and raw logs are under `results/browser-host/wasm_accessibility*`. The retained failed Terminal build predates the explicit JsValue closure annotation. The initial WASM Clippy failure was fixed with named callback aliases, not lint suppression.

These are observed accessibility component and real-Terminal results, not full xterm.js equivalence or native Linux/macOS evidence. The broader selection/scroll integration remains task34/239.
