# Live charset integration checks

These checks use the actual source headless terminal and the Rust `CoreTerminal` parser/engine. They complement the static maps and service oracle in `docs/charset-oracle.md`. Browser/WASM verification remains a separate task, not an inferred result.

## Commands

From the repository root, after installing reference dependencies:

```sh
node scripts/build-slot.mjs --owner charset-source --lane native -- node scripts/charset-terminal-oracle.mjs --generate
node scripts/build-slot.mjs --owner charset-verify --lane native -- node scripts/charset-terminal-oracle.mjs
node scripts/build-slot.mjs --owner charset-native --lane native -- cargo test -p xterm-core --test charset_terminal
node scripts/build-slot.mjs --owner charset-clippy --lane native -- cargo clippy -p xterm-core --lib --test charset_terminal -- -D warnings
node scripts/check-oracle-startup.mjs --charset
```

The generation flag writes source-derived fixtures. Normal execution rejects drift. Keep source and expected fixtures unchanged when Rust disagrees. Use the shared Cargo target and configured profiles, and check disk headroom before real bundling/build commands. The isolated startup checker does not start a compiler or bundler.

## State corpus

`scripts/charset-terminal-oracle.mjs` bundles the unmodified source terminal and charset tables under `tmp/charset-terminal/`. It reuses `scripts/terminal-snapshot.mjs`. The report in `results/charset-terminal/oracle.json` records exact reference hashes, tool/platform identity, fixture hashes, source runs and writes. The generator's last-attempt receipt must agree with its actual command exit status. Source parser diagnostic logs are retained in the report rather than suppressed.

There are 134 state cases, run through three source input profiles: one UTF-16 string, single UTF-16-unit chunks, and single UTF-8-byte chunks. The source confirms identical final state for these non-reentrant cases, giving 402 source runs. Rust checks the same 402 profiles and separately compares their exact ordered bell/binary/cursor/linefeed/scroll/data/title/resize/render/accessibility events. Event sequences are compared per profile, not assumed invariant across different write boundaries. Native public write-queue `onWriteParsed` is a separate wrapper gate.

The cases cover:

- All 16 charset designators and all seven registered intermediates, including the intentionally unsupported slash form. Each receives all 95 printable ASCII characters.
- SI, SO and all five locking-shift escape forms, with different maps designated in G0 through G3.
- Default charset selection by `ESC % G` and `ESC % @`, including later G1, G2 and G3 invocation.
- Unknown designators and the slash form's no-change behavior.
- Saved charset identity, designated-slot shallow copies, active level, later designation changes and extra-slot retention after restore.
- Soft reset, full reset, DECSET mode 2 and alternate-buffer save/restore.
- Wide wrapping, supplementary characters and custom maps with empty, NUL, multi-unit and lone-surrogate replacements. Custom `ij` maps to one printed `i`; a supplementary replacement maps to its first UTF-16 unit rather than a complete scalar.
- Soft hyphen, DEL and a C1 boundary value. DEL emits a recoverable source parser diagnostic, which remains recorded. Diagnostics' chunk-local positions can differ across profiles and are not equated with state failures.

Each snapshot compares both real buffers, every packed cell and exact UTF-16 content, widths, attributes and extended data, wrapping, cursor/base/viewport/margins, terminal modes, active charset identity, the complete designated slot array, both buffers' saved charset state and accessibility character values. Known aliases use a stable canonical designator. Unexpected native map cloning is rejected by pointer-identity checks rather than hidden by content equality.

## Counterexample and fix

The initial Rust run failed nine of 402 profiles. `ESC % G` and `ESC % @` called `CharsetService::reset()`, incorrectly clearing G1 through G3 and changing the designated-array length. The source invokes G0 and replaces only its map. The engine handler now does the same and propagates growth errors.

The expected fixture was not changed. Initial failures remain under `results/charset-terminal/failures/default-charset-before-fix.*`; `initial-rust-tests.log` retains the failing command. After the handler fix, all 402 state/value profiles passed in the targeted debug run. This is not a claim that callback timing passed.

## Pre-print callback contract

A second fixture, `charset-callback.tsv`, records three real source runs for `ESC ( 0 q q`. An accessibility listener observes the cursor and existing cell before the write, then changes G0 to the British map on its first callback.

For a whole print run, the source emits two line-drawing characters because it captures the charset once at run start. With unit/byte chunks, the later print run captures the changed map, so the output is a line-drawing character followed by ordinary `q`. The callback must see cursor positions zero and one, not the final position two. This deliberately demonstrates why reentrant chunk profiles cannot be assumed equivalent.

The second callback scenario edits `q` on a captured custom map itself. All three profiles must then print a line-drawing character followed by `X`. This requires live mutable object identity; an immutable table or a per-run content snapshot cannot pass. `Charset` uses an encapsulated Rc/RefCell map with fallible scalar lookup and no escaping guards.

`callbacks_observe_pre_write_cells_and_live_captured_map_identity` checks all six exact source observations and final snapshots. The initial engine queued accessibility callbacks after the complete print run. Task 153 implements safe pre-write callbacks and task 211 the live mutable handle. Neither can be replaced with a queued-after shortcut, ignored test or expected-output edit. Current scoped Windows debug evidence passes all 408 state/callback profiles and 402 ordered event comparisons. `results/charset-terminal/native-verification.json` records exact lease receipts, failures and limits. Independent task acceptance and browser task 133 remain required.

## Scope boundaries

The source's direct internal `selectCharset` method also guards malformed string lengths. The registered parser identifiers cannot generate that invalid call shape in the Rust typed dispatch. This test corpus exercises the actual input paths, not arbitrary JavaScript calls through private `_core` objects. Keep such representation differences visible in compatibility documentation.

The Rust public headless wrapper's queue/gating/lifecycle checks and the browser/WASM bindings have separate owners and tests. Task 133 must run charset cases through the actual Rust-backed browser entry points, including events and disposal. Native Linux/macOS and browser runtime results are still unrun here. Local release/full-matrix commands are not authorized by the controlled build policy; they need separately coordinated or remote execution. The attempted WASM acceptance-hooks check failed in unrelated width/scroll/scheduler integration, so it is not a passing web result. Finite state fixtures, even when green, do not establish full xterm.js equivalence.
