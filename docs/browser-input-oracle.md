# Browser input policy oracle

Tasks #93 and #95 add an independent comparison of `source_repo/src/browser/Clipboard.ts` and `source_repo/src/browser/input/CompositionHelper.ts` against the Rust implementations in `crates/xterm-web/src/input/`. Norman owns the production policy and DOM wiring in task #33. This oracle does not create a terminal and cannot establish live browser or operating-system IME equivalence.

## Original assertions

`node scripts/browser-input-oracle.mjs` erases TypeScript annotations, relocates import/export references, and removes the four constructor dependency-injection decorators. Test bodies, assertion expressions and expected values are unchanged. Every nested `beforeEach` hook runs. The supplied reference discovers three Clipboard tests with six assertions and eight CompositionHelper tests with ten assertions.

The oracle supplies structural collaborators for the fields those tests consume. Buffer starts at x=0/y=0 with a visible cursor and the test's 10x5 dimensions. Render CSS cell dimensions are zero, matching MockRenderService. Options provide the source defaults fontFamily=monospace and fontSize=15. Core forwards each emitted input to the original assertion callback. These are not the complete original MockBufferService/MockOptionsService implementations, so this is explicitly an assertion-preserving isolated policy baseline. The unchanged full upstream runner remains a separate gate.

Original zero-delay callbacks run in FIFO order through a deterministic bounded task queue, not sleeps. Geometry callbacks still execute source code, but style/layout output is not compared here. The trace omits those geometry callbacks and records the policy callbacks they precede. It samples the same live textarea snapshot at deferred execution time. Source callbacks are identified by explicit source-body markers; unknown synthetic callbacks fail instead of being silently dropped. Source hash and fixture drift checks expose changes requiring review.

The six Clipboard calls and 123 composition operations become 129 checked-in fixture rows at `crates/xterm-web/tests/fixtures/browser-input-upstream.tsv`. The Rust fixture test pins that count and compares every row. Original composition assertions still check the final handled-text values, including Korean transitions, Japanese conversion, immediate following input and a composition in the middle of preserved screen-reader text.

## Differential trace contract

The stdin runner `crates/xterm-web/examples/input_trace.rs` shares its parser with fixture tests. It invokes Rust policy only. Each TSV input has four fields: operation, hexadecimal UTF-16 units, numeric argument or -, and secondary flag or -. The ~ marker denotes an empty string. Bad protocol data returns an error instead of ignoring a step.

Clipboard operations compare exact UTF-16 output from prepare, bracket and paste policy. Composition operations reset the model, mutate textarea text/selection, start/update/end a composition, process a keydown, or execute one pending policy callback. Every resulting snapshot compares composing state, continue-processing return, overlay UTF-16, the full ordered list of emitted data events including empty events, and pending policy callback count. Event boundaries are preserved rather than comparing only concatenated text. Private implementation fields are not read; later actions exercise their observable effects.

The seeded corpus includes every BMP unit, lone surrogates, CR/LF/ESC combinations, every paste mode/ignore combination, 2,000 synthetic 35-action composition scenarios, reversed/out-of-range/absent selection endpoints, repeated starts/ends/updates, modifier/229/ordinary keydowns and deferred callback races. Synthetic selection values remain nonnegative integers, matching DOM selection index types. Geometry, native textarea normalization and disposal are deliberately not modeled as proven.

The expanded successful run had 252,412 comparisons: 129 original fixture operations, 65,536 BMP normalization cases, 65,536 BMP bracket cases, 40,000 seeded paste-option cases, 72 directed composition regression steps and 81,139 seeded composition steps. Two Rust fixture replay tests passed, pinning the original129 and directed72 row counts. Seed is 1229800758, hexadecimal 0x494d4536. Directed regressions cover repeated229 coalescing, insertion/deletion/equal-length/no-change textarea diffs, composition-start cancellation, immediate-commit send cancellation, a new composition before delayed send, reversed selections with preserved suffix and lone-surrogate/modifier paths. The current machine-readable report in `results/browser-input/verification.json` is authoritative if the corpus grows.

## Reproduce and review

```
node scripts/browser-input-oracle.mjs
cargo test -p xterm-web --test input_upstream
cargo clippy -p xterm-web --example input_trace --test input_upstream -- -D warnings
```

Run with Node supporting `stripTypeScriptTypes` and cargo/rustc on PATH. `--generate` refreshes the upstream fixture only after reviewing reference changes. Ordinary verification rejects drift. The script records source and Rust/helper hashes, exact case counts, seed, host/tools and build status. First failing cases retain the reproducing trace. A shared build error is a failure, not a skipped test or an oracle pass.

The script reuses `scripts/quality-process.mjs` for bounded cargo process-tree lifetime and free-space monitoring. It retains build.log/build.json, stops below 512 MiB free, uses the existing target directory and honors `.cargo/config.toml`. Temporary inputs stay under `tmp/browser-input/`.

Independent review must inspect scheduler identification, each captured action, fixture-count integrity, event-boundary encoding, random generator distributions and UTF-16 handling, then rerun both Rust and JS commands. A directed test or retained seed is required for any found discrepancy.

Actual clipboard event propagation, copy/context-menu geometry, IME overlay layout, browser scheduling, input cancellation, listener cleanup and Chromium/Firefox/WebKit native composition remain task #33 and end-to-end responsibilities. Linux/macOS runtime and full terminal equivalence remain NOT PROVEN.
