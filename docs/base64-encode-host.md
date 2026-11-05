# Base64 encoding host compatibility

Overall utility/status equivalence is NOT PROVEN. Task394 repairs one observable exported Base64 boundary. Task323 still owns complete Attach/Clipboard/Fit/WebLinks/ProgramStatus/Progress API, original suites and demo closure.

## Defect and repair

`Base64.encodeText` previously called the portable Rust codec directly and returned a typed JS string. The reference first UTF-8 encodes the input, checks `bytes.toBase64`, then invokes that method if it is defined. Otherwise it creates a binary string and calls `btoa`. These host calls are observable. The old binding ignored getter effects, overridden returns and thrown values.

The repaired browser binding retains Rust's UTF-16 replacement and UTF-8 conversion, then uses the browser-owned Uint8Array. It checks the capability once and reads it again for invocation, matching source receiver and ordering. If absent, fallback reads the same array after the getter, so byte mutations affect its binary string. A small FFI adapter invokes the standard host `btoa`, as the decoder already invokes `atob`. This is not a terminal-engine fallback. Portable native encoding is unchanged. No dependency or unsafe code was added.

The Rust browser method returns `Result<JsValue, JsValue>` rather than `Result<JsString, JsValue>`. Normal host methods still return strings. Overridden methods may return objects, numbers, null or undefined, which source returns without coercion. The clipboard handler interpolates this raw value later, preserving its existing activation and coercion order. JS errors retain their original identity.

The focused contract does not establish arbitrary TextEncoder constructor replacement, every mutable host intrinsic, full clipboard permission behavior, or complete original-suite equivalence.

## Assertions and evidence

`crates/xterm-web/tests/fixtures/base64-encode.js` supplies16 deterministic traces to both the unchanged original Base64 and the Rust browser export. The fixture covers UTF-8 bytes and lone-surrogate replacement, capability getter order, receiver/argument count, raw object/undefined/null/numeric returns, getter/method exceptions, noncallable values, a changing getter, fallback return/error identity, byte mutation during capability lookup, input conversion order, empty undefined input and Symbol rejection. Every prototype/global override is restored in `finally`.

`scripts/base64-encode-oracle.mjs` bundles the unchanged ClipboardAddon source with its existing esbuild installation. It verifies all776 reference manifest entries before and after execution, writes source/fixture/trace SHA-256 hashes and removes its scratch bundle in `finally`. The expected JSON is source-generated, not hand-edited. This is a source-method run, not the original complete browser suite.

`wasm_base64_encode` has two test functions per browser. The first compares all16 traces. The second loads a ClipboardAddon into an actual Rust Terminal, parses a queued OSC52 query and checks UTF-8 bytes plus raw-return interpolation into the exact outgoing reply. Existing `wasm_clipboard_values` and `wasm_utility_bindings` suites remain unchanged.

Retained evidence is under `results/base64-encode/`:

- `before.json`: both focused test functions fail in Chromium, Firefox and WebKit against the old binding. Its initial fixture had15 traces, before the getter-mutation regression was added. The report retains expected/actual output and source/artifact digests.
- `source.json`: final16 source traces discovered/executed/passed, zero failed/skipped/unrun. Reference manifest is unchanged before and after.
- `after.json`: both test functions pass per browser, including48 source-matched trace executions and3 live OSC52 executions across all engines.
- `clipboard-values.json` and `utility-bindings.json`:5 and3 existing test functions pass per browser. Together with the focused suite,30 Rust browser test executions pass, zero fail or skip. Versions are Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4.
- `native.log`:10 test functions pass across utility_clipboard2, utility_live_terminal5 and utility_oracle3. The latter retains the existing seeded base64/UTF-16/malformed-UTF-8 corpus. No native encoder behavior changed.
- `clippy-wasm.log`: warning-denied WASM checks pass for all three affected suites.
- `fmt.log` records Windows OS206 from the long worktree path. After moving only this owned regular worktree from `tmp/Ezana` to `tmp/E`, the exact `cargo fmt --all -- --check` passes in `fmt-short.log`. No junction, assertion change or lint suppression was used.

Reports identify disposable compiler/browser artifacts by digest. Paths from before the worktree move are historical. Generated WASM/JS bundles are removed during owned cleanup and are not distribution downloads. Full workspace all-feature/platform quality and original suites are separate gates, not implied by these scoped receipts.

## Reproduce

Use Node24, locked Rust1.85.1, wasm32-unknown-unknown, wasm-bindgen0.2.100 and installed Playwright engines. Existing tool installations can be selected through SOURCE_ROOT, WASM_BINDGEN and PLAYWRIGHT_MODULE. Keep temporary work and builds on O: under current team resource limits.

```sh
node scripts/base64-encode-oracle.mjs
WASM_TEST=wasm_base64_encode node scripts/browser-host-check.mjs
WASM_TEST=wasm_clipboard_values node scripts/browser-host-check.mjs
WASM_TEST=wasm_utility_bindings node scripts/browser-host-check.mjs
cargo test -j2 -p xterm-addons --test utility_clipboard --test utility_oracle --test utility_live_terminal
cargo clippy -j2 -p xterm-web --target wasm32-unknown-unknown --test wasm_base64_encode --test wasm_clipboard_values --test wasm_utility_bindings -- -D warnings
cargo fmt --all -- --check
```

Use `--generate` only after reviewing a real source/trace change. Source files and original assertions must remain untouched.
