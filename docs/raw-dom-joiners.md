# Raw DOM character-joiner rows

Task348 removes the browser service's eager `render_ranges` projection. The shared Rust row planner now has typed native and raw browser inputs. The raw input retains the outer array and shifts it only when the source row would consume its head. It repeats endpoint reads, strict numeric start checks, selection comparisons, string concatenation, numeric increment and subtraction in source order. The JavaScript imports implement operators and host property access, not the terminal engine or row policy.

Raw zero-length, reversed, fractional, string, undefined and out-of-bounds endpoints no longer become an invented integer-validation error. They follow the source's row traversal and CSS spacing behavior. Unreached endpoint getters stay unread. `shift` retains its receiver and returned range identity. Thrown values propagate unchanged. A persistent work cell also preserves recursive row creation's mutation of the source factory's reusable cell. The width captured before getters remains separate from later work-cell contents.

The renderer releases line, selection and theme RefCell borrows before invoking a raw getter or coercion. Browser columns remain Numbers, including fractional columns, and packed cell reads reuse the core's numeric-address implementation. Native typed rows retain their validated integer range contract. `RowContext.decorations` now receives an f64 column so malformed browser ranges do not silently round decoration coordinates.

## Commands

Use an owned O: worktree and private target, Cargo jobs two, and explicit existing TEMP, TMP and TMPDIR directories under its tmp folder. Node24.15.0 and Rust1.85.1 produced the scoped Windows results. wasm-bindgen0.2.100 must match Cargo.lock. Source dependencies and Playwright can be reused by absolute path without a junction.

```sh
SOURCE_PACKAGE_JSON=/absolute/reference/package.json node scripts/joiner-row-oracle.mjs
cargo test -j2 -p xterm-web --test character_joiner_upstream
cargo fmt --all -- --check
cargo clippy -j2 -p xterm-web --lib --test character_joiner_upstream -- -D warnings
cargo clippy -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_joiner_row --test wasm_joiner_row_live --test wasm_joiner_raw --test row_factory_browser -- -D warnings
WASM_BINDGEN=/absolute/wasm-bindgen PLAYWRIGHT_MODULE=/absolute/playwright/index.mjs WASM_TEST=wasm_joiner_row node scripts/browser-host-check.mjs
```

Repeat the browser command for wasm_joiner_row_live, wasm_joiner_raw, row_factory_browser, dom_font_browser, wasm_font_terminal and wasm_font_shaping. The runner closes browsers and servers on success and failure. `--generate` on the source oracle writes the shared input fixture only after executing the unchanged source. Do not regenerate it merely to obtain a pass.

## Scoped evidence

`results/joiner-row/source.json` records freshly bundled source hashes, all38 passing unchanged DomRendererRowFactory tests, and295 finite shared raw cases. The corpus has39 directed cases and256 seeded cases, seed0x348C0DE. The differential compares complete HTML, getter/coercion traces and thrown-value identity categories. It covers strict head matching, zero and reversed ranges, fractional widths, skipped wide cells, late errors, array-like shift identity, changing endpoints, selection and cell mutation, and recursive work-cell reuse.

The raw WASM suite has two test functions. One checks all295 finite cases. The other checks two explicit nontermination safety errors, not source equivalence. The separate live Terminal test exercises five paths through the actual renderer: an unmatched range with a throwing endpoint, fractional endpoints, cell mutation during a getter, original error identity, and disposal during a getter followed by that original error.

A negative control copied the identical live test and support module into a separate detached worktree at4a38b52, without changing production source. Chromium discovered one test function and it failed on the first unmatched-range path, where the old eager projection invokes the unreachable throwing endpoint. The runtime aborts after that panic; the remaining paths were not reported as passed. `baseline-live.json` retains the failure and artifact hashes. The temporary baseline tree was owned by Herbert and must be removed after preserving this evidence.

Existing regressions separately replay565 captured original row calls and27 captured original joiner calls. Counts for source tests, captured calls, directed cases and WASM functions describe different checks and are not summed into one assertion total. Browser receipts under results/joiner-row retain commands, engine versions, test names and artifact hashes. Generated WASM bundles are disposable outputs.

## Limits and safety decisions

Overall equivalence is NOT PROVEN. Task364 retains broader raw renderer callback behavior, including live theme/options/decorations and host DOM mutation, missing packed attributes and source-hang decisions. Task342 retains complex font metadata. Full original suite and native platform proof remain separate open work.

The source can loop forever when a consumed endpoint is positive Infinity, or when a negative row column or text/selection increment cannot change its Number value. Rust returns an explicit error instead of hanging. Rejecting an Infinity value read from a getter can also reject a source getter that would return a finite value on a later call. That compatibility difference is deliberate and unresolved, not an equivalence result. The finite differential does not execute these source hangs. Safety tests prove only Rust's explicit error paths. Task364 must retain bounded source reproductions and resolve the broader compatibility decision.

Independent review found a finite cursor-state defect despite the first294-case pass. A changing endpoint can pass cursor validation and later cover it, after which a reversed range revisits earlier cells. The source changes its local cursor column at the joined span. Rust now preserves that column and its source-timed mutation, including later join validation. The retained cursorMutationRevisit case raises the corpus to295 cases. Review counterexample details are in results/joiner-row/review-counterexample.json.

The initial browser run also exposed a test-probe ABI mistake: i64 parameters expect JavaScript BigInts, but the shared selection inputs are Numbers. The probe now accepts f64 and converts its bounded test coordinates to i64. No source assertion or production behavior was changed to hide that harness error. An attempted wasm_font_addons target did not exist; it was unrun, then the actual wasm_font_terminal and wasm_font_shaping targets passed.
