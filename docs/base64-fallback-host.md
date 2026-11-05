# Base64 fallback host compatibility

Overall equivalence is NOT PROVEN. Task400 repairs the raw fallback boundary left open by task397. Task323 still requires full Attach, Clipboard, Fit, WebLinks, ProgramStatus and Progress source/API, original-suite and demo closure.

## Defect and repair

The old fallback declared the `atob` result as a JS string, collected UTF-16 units in Rust and copied their low bytes into a typed array. It cached the length and bypassed observable `Uint8Array` construction, raw constructor results and index setters. Null and undefined replacements could throw through an uncaught js-sys string import rather than the source catch. In the retained baseline these imports also left the exported WASM receiver borrowed, so later `free` calls failed.

The repaired binding keeps `atob`, constructor and decoder results as `JsValue`. Rust controls the fallback loop using a JS-number-compatible f64 index. Three small FFI expressions construct the byte container, compare the current index against the current length, and call charCodeAt followed by strict index assignment. They preserve constructor selection before evaluating its length argument, repeated getter/coercion effects, receiver identity and raw assigned values. Every fallible host operation uses a catch binding. The existing Rust source catch returns an empty string on conversion/decoder failure. The initial capability lookup remains outside that catch. TextDecoder still receives the raw resulting BufferSource or replacement object.

Rust retains capability branching, loop control, exception policy, OSC52 parsing, provider calls and parser continuation. These FFI expressions perform host codec/property operations only. No JavaScript terminal engine, dependency, unsafe code or native API change was introduced. Native codec logic and the modern decoder path are unchanged. This does not claim arbitrary hostile intrinsic replacement throughout unrelated APIs, clipboard permissions or complete addon lifecycle parity.

## Assertions and observed results

`crates/xterm-web/tests/fixtures/base64-decode-fallback.js` supplies25 deterministic cases to unchanged ClipboardAddon.ts and the Rust export. Cases cover primitive/boxed/object atob values, changing length, raw length conversion, charCodeAt getters/receivers/raw results, throwing conversions, constructor mutation/selection, raw length arguments, raw setter values, setter exceptions, strict frozen-object assignment and global/capability getter timing. Every override is restored in finally. A baseline receiver-cleanup error becomes an extra asserted trace field, not an ignored failure.

The existing source oracle accepts `--fallback` and writes a separate report. It checks all776 reference-manifest entries before and after, records source/fixture/trace SHA-256 hashes, and removes its temporary bundle in finally. Expected JSON comes from unchanged source, not handwritten results. This is a source-method differential, not a full original terminal suite.

`wasm_base64_fallback` contains two Rust test functions per browser. One compares all25 source cases. The other loads ClipboardAddon into a real Rust Terminal and queues OSC52 with a changing-length atob object. Source allocates four bytes but stops after the first charCodeAt call, producing A followed by three NULs. The old binding instead decoded all four bytes as A€.

Evidence is retained in `results/base64-decode-fallback/`:

- `before.json`: both focused test functions fail in Chromium, Firefox and WebKit on the old binding. Expected/actual traces, live-provider mismatch and source/artifact hashes are retained.
- `source.json`:25 cases discovered/executed/passed, zero failed/skipped/unrun;776 reference entries unchanged before and after.
- `after.json`: both functions pass in all three engines, including75 source-matched trace executions and three real OSC52 writes.
- `wasm_base64_decode.json`, `wasm_base64_encode.json`, `wasm_clipboard_values.json`, `wasm_utility_bindings.json`:2,2,5 and3 existing functions pass per engine. Together with the new suite,42 browser test executions pass, zero failed/ignored. Engines are Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. The existing25 decode and16 encode source oracles also pass unchanged.
- `native.log`: ten native utility_clipboard/utility_live_terminal/utility_oracle tests pass, including existing seeded codec assertions.
- `fmt.log`: exact full-workspace `cargo fmt --all -- --check` passes from the short owned U worktree.
- `clippy-wasm.log`: warning-denied WASM Clippy passes for all five browser suites.

Node24.15.0, Rust/cargo1.85.1 and wasm-bindgen0.2.100 were used on native Windows. Full workspace all-feature quality, Linux/macOS runtime, complete unchanged original suites and full utility/status parity remain separate gates. Scoped passing tests are not full-port proof.

## Independent review

The isolated task400 reviewer accepted b3a93e33 with no findings and no tracked edits. It independently reran all25 new fallback,25 existing decode and16 encode source cases; all42 browser test executions; ten native utility tests; full-workspace formatting and focused warning-denied WASM Clippy. The baseline and passing reports remain scoped evidence, not full utility or port equivalence.

## Reproduce

Use existing source-only Node dependencies, Playwright engines and wasm-bindgen0.2.100 through SOURCE_ROOT, PLAYWRIGHT_MODULE and WASM_BINDGEN. Keep new builds and scratch on O: under current team resource limits.

```sh
node scripts/base64-encode-oracle.mjs --fallback
node scripts/base64-encode-oracle.mjs --decode
node scripts/base64-encode-oracle.mjs
WASM_TEST=wasm_base64_fallback node scripts/browser-host-check.mjs
WASM_TEST=wasm_base64_decode node scripts/browser-host-check.mjs
WASM_TEST=wasm_base64_encode node scripts/browser-host-check.mjs
WASM_TEST=wasm_clipboard_values node scripts/browser-host-check.mjs
WASM_TEST=wasm_utility_bindings node scripts/browser-host-check.mjs
cargo test -j2 -p xterm-addons --test utility_clipboard --test utility_oracle --test utility_live_terminal
cargo clippy -j2 -p xterm-web --target wasm32-unknown-unknown --test wasm_base64_fallback --test wasm_base64_decode --test wasm_base64_encode --test wasm_clipboard_values --test wasm_utility_bindings -- -D warnings
cargo fmt --all -- --check
```

Use `--generate` only after reviewing a real trace/source change. Never edit the original source or assertions to clear a failing test. The owned temporary worktree is O:/xterm-rust-swarm-16/tmp/U. Its private target, scratch and browser artifacts will be deleted after review/integration; durable reports retain their hashes and historical paths.
