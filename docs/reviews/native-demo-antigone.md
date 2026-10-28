# Independent native example review

Reviewer Antigone inspected task 132, owned by Aaliyah. The example is a line-oriented inspector over the real Rust headless terminal and cells. It is not a PTY shell, browser renderer or full-equivalence proof.

## Source findings and fixes

Read `examples/native.rs`, every `examples/native/*.rs` module, the deterministic command fixture, `scripts/check-native-demo.mjs` and `docs/native-demo.md`.

The implementation sends decoded input into the Rust parser, reads actual buffer cells/attributes/cursor, uses the Rust serializer and keeps outbound input separate from display echo. ANSI color output is derived from cell attributes. Script assertions inspect real state, not fixed output strings. Viewport allocation policy is explicit and configurable; malformed escapes are rejected before terminal mutation. Explicit fallible cleanup and output errors propagate.

Three independent pre-review findings were corrected:

1. `std::env::args` could panic on non-Unicode native arguments. Parsing now uses `args_os`; script paths stay lossless PathBuf values and unsupported option text is a recoverable error. Unix and Windows argument boundary tests are cfg-gated.
2. Expected-failure checks previously allowed a null exit code from a signal or process failure. They now require a real positive exit code, the expected diagnostic and absence of panic, signal, timeout, spawn/resource/termination/abort errors.
3. Script mode and EOF alone did not exercise the interactive command loop's error recovery. The checker now pipes input, rejects one bad command, continues through three state assertions and quits cleanly.

The example also installs a live fallible native console sink instead of accumulating inaccessible default diagnostics. Constructor records are forwarded once. UTF-16 observations are lossless; native display replacement for lone surrogates is documented rather than mistaken for port behavior.

## Executed independent check

Ran `node scripts/check-native-demo.mjs` under the accepted native build-slot queue. The successful wrapper receipt is:

`results/build-slots/1793174105146-37708-0f6ad3af-68fc-4993-a4d5-dc439899ccf2/receipt.json`

The detailed immutable example report is:

`results/demos/native/runs/2026-10-28T07-55-07-496Z-c879f3a9-3910-4db1-b5c6-b0a3c5aac689/summary.json`

Observed all eighteen planned checks and no missing or mandatory-failed checks:

- Ten native example unit tests passed.
- Warnings-denied Clippy, rustfmt, target/metadata validation and a current Cargo build passed.
- Plain and ANSI executions each passed 21 actual state assertions, including parsed colors, Unicode widths, cursor replacement, reflow, dimensions and outbound data.
- The actual stdin-driven interactive loop recovered from one rejected command, then passed three assertions and explicit cleanup.
- EOF and help paths passed.
- Six negative programs returned exit 1 with their expected diagnostics. These process failures remain recorded as failures; the expected-failure assertions passed. None was a panic, timeout, signal or resource failure.

Source hashes were stable and the executed artifact hash stayed unchanged before and after runtime checks. The tested Windows debug binary was 2,636,800 bytes, SHA-256 `a896d254228d0a280e67a713448e06e0cf17671ae0e1b72c9dcc1389370c6b99`. This is an artifact identity, not a matched-feature distribution-size or performance comparison.

Earlier independent attempts are retained. They failed during the shared cleanup-factory API migration before runtime checks could execute. They are not relabeled passing and do not replace the successful receipt above.

## Verdict

Accept task 132's working native example on observed Windows evidence. Native Linux/macOS runtime checks, real browser rendering, addon parity and the full workspace/feature/quality matrix remain separate gates. Overall xterm equivalence remains NOT PROVEN.
