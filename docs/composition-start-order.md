# Composition-start overlay setter ordering

Task399 continues the complete input/accessibility audit319. The source is `source_repo/src/browser/input/CompositionHelper.ts`, `compositionstart`. It assigns the live suffix, clears the overlay's textContent, clears _dataAlreadySent, then adds the active class. A decorated text setter may throw or reenter composition before the sent-data offset is reset.

Previously the Rust DOM adapter cleared the sent-data offset while recording the suffix, before assigning textContent. After an earlier standalone229 sent X, a failing overlay setter caused the next deferred send to emit YZ instead of the source's Z. The retained before-fix Chromium report contains both the plain throw and reentrant compositionend throw counterexamples. It also records a WASM test-runner table-index exception after the intentional assertion failures. It is a failed run, not a passing test receipt.

## Implementation

`input/composition.rs` separates suffix capture from clearing overlay/sent-text state. Native start still performs both phases in order. The DOM adapter clears the model's text and sent-data only after a fallible Reflect.set succeeds. The same private setter helper serves compositionupdate without changing its scheduling. No RefCell borrow spans the DOM setter, so nested composition events remain callable. A thrown value returns through the existing fallible listener without losing identity.

The earlier compositionend fix395 is unchanged. This patch does not claim every decorated selection getter, geometry getter, DOM style setter, timer, focus or full-Terminal input path is equivalent.

## Verification

Run from the producer worktree with Cargo target and TEMP/TMP in its owned O: directory, CARGO_BUILD_JOBS=2.

- `node --test scripts/composition-source-fixture.test.mjs` checks five original method bodies against the read-only source. The existing keydown fixture is reused unchanged. No original source or assertions were modified.
- `cargo test -p xterm-web --lib --test input_upstream --test input_event_getters` includes the new phased native state regression.
- `node scripts/browser-input-oracle.mjs` reruns unchanged clipboard/IME cases and deterministic seeded comparisons without fixture regeneration.
- `WASM_TEST=wasm_input_callbacks node scripts/browser-host-check.mjs` runs seven test functions per engine, including four new source-body scenarios across two functions. Plain success, throwing setter, nested compositionstart, and nested compositionend followed by throw compare identical state/event/error traces.
- `WASM_TEST=wasm_input node scripts/browser-host-check.mjs` checks existing DOM input behavior in all three engines.
- Native and wasm32 callback-target Clippy pass with `-D warnings`. Changed Rust files pass rustfmt check. Full-workspace gates and native Linux/macOS remain outside this scoped producer run.

The producer and independent reviewer each passed 63 native tests, five source-body integrity tests, scoped rustfmt and native/WASM warning-denied Clippy. Callback tests passed seven functions per engine on each run. The producer also passed input seven per engine and 252,412 policy comparisons with seed1229800758, including 11 unchanged cases with 16 assertions.

Task399 was independently accepted. The reviewer found no concrete scoped defects and changed no source.

Results are in `results/composition399/`. The before-fix mismatch is retained. Browser run reports contain tool versions, source/artifact hashes, expanded test names and exact counts. Component assertions are not original full-Terminal E2E coverage. Overall equivalence NOT PROVEN.
