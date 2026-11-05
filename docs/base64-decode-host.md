# Base64 decoding host compatibility

Overall utility/status equivalence is NOT PROVEN. Task397 repairs the exported Base64 decoding boundary. Task323 still requires full Attach, Clipboard, Fit, WebLinks, ProgramStatus and Progress API, original-suite and demo closure.

## Defect and repair

The old browser binding forced `Uint8Array.fromBase64` results into a Uint8Array and decoded a copied byte vector in Rust. The unchanged source passes the result directly to TextDecoder. ArrayBuffer, DataView with offsets and non-u8 typed views are valid BufferSource inputs, but the old binding silently returned an empty string for them. It also bypassed observable decoder construction, decode getters, method calls and overridden return values.

Rust still owns the capability branch, source catch boundary, OSC52 parsing, provider calls and parser continuation. Two small FFI expressions call standard browser TextDecoder and base64 APIs. These are host codec operations, not a JavaScript terminal engine. The modern expression preserves decoder construction and decode-property lookup before evaluating the second fromBase64 getter/call. Its original receiver, raw BufferSource and raw decoder result are retained. The initial capability getter remains outside the catch; failures inside the conversion/decoder expression return an empty string, exactly as source does. No production dependency or unsafe code was added.

The browser `Base64.decodeText` Rust binding now returns `Result<JsValue, JsValue>`, rather than `Result<JsString, JsValue>`. Standard decoder calls still return strings. An overridden decoder may return objects, undefined, null or numbers without coercion. Portable native UTF-8/base64 code is unchanged.

This focused repair does not establish arbitrary replacement of every intrinsic. In particular, the fallback still expects atob to return a JS string and constructs its byte array through js-sys. Non-string atob replacements and arbitrary Uint8Array constructor replacements need separate closure. The full clipboard permission/lifecycle and original-suite gates are not implied by these tests.

## Assertions and observed results

`crates/xterm-web/tests/fixtures/base64-decode.js` supplies25 deterministic traces to both unchanged source and the Rust export. They cover Uint8Array/BOM/malformed UTF-8, ArrayBuffer, offset DataView, Uint16Array raw bytes, undefined/invalid inputs, decoder and fromBase64 getter order, receivers, argument counts, raw decoder results, outer thrown-value identity, inner exceptions, noncallable decode and fallback decoder behavior. All host overrides are restored in finally.

The existing source oracle script accepts `--decode` to reuse bundling, manifest verification and reporting rather than duplicate them. It executes unchanged ClipboardAddon.ts, verifies all776 manifest entries before and after, records source/fixture/trace SHA-256 hashes and removes its temporary bundle in finally. Expected JSON is generated from source, not handwritten. This is a source-method run, not the full original browser suite.

`wasm_base64_decode` contains two Rust test functions per browser. One compares all25 source traces. The other loads ClipboardAddon into an actual Rust Terminal, queues an OSC52 set sequence, receives a DataView from the modern host hook and verifies that the live provider receives selection c and text A€.

Evidence is retained in `results/base64-decode/`:

- `before.json`: both focused test functions fail in Chromium, Firefox and WebKit against the old binding. Expected and actual traces, the live clipboard failure and artifact/test hashes are retained.
- `source.json`:25 source cases discovered/executed/passed, zero failed/skipped/unrun, unchanged manifest before and after.
- `after.json`: both focused functions pass per browser. This includes75 source-matched trace executions and3 live OSC52 writes.
- `wasm_base64_encode.json`, `wasm_clipboard_values.json` and `wasm_utility_bindings.json`:2,5 and3 existing functions pass per browser. Together with the focused suite,36 browser test executions pass with zero failures/skips. Engines are Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4.
- `native.log`:10 native tests pass across utility_clipboard2, utility_live_terminal5 and utility_oracle3. Existing seeded base64/UTF-16/malformed-UTF-8 assertions are unchanged.
- `clippy-wasm.log`: warning-denied WASM Clippy passes for all four browser suites.
- `fmt.log`: the exact full-workspace fmt command initially hits Windows OS206 at the owned Ib worktree path. Moving only that regular worktree to short J fixes the command; `fmt-short.log` records success. No junction or reduced formatting gate was used.

Report paths from Ib are historical after the owned worktree move to J. Artifact digests remain valid. Temporary browser/compiler output is retained only until review and integration, then removed. These files are not distribution downloads. Full workspace all-feature/native-OS quality and complete unchanged suites remain separate work.

## Independent review and integration

Task397's isolated reviewer accepted commits57dbceed and fca4b80c with no findings. The reviewer independently executed25 decode and16 encode source traces, all36 browser test executions, ten native tests, full fmt, focused warning-denied WASM Clippy and diff whitespace checks. This independently confirms the scoped tests, not overall addon equivalence.

The canonical `scripts/worktree-merge.mjs` integrated fca4b80c into actual main. `results/base64-decode/merge.log` records changed-crate build/all-target tests, affected-dependent checks, workspace check and applicable WASM library checks. The helper removed the owned private Cargo target. This merge gate is not the full all-feature/native-OS workspace matrix.

## Reproduce

Use Node24, locked Rust1.85.1, wasm32-unknown-unknown, wasm-bindgen0.2.100 and installed Playwright engines. Select existing tool installations through SOURCE_ROOT, WASM_BINDGEN and PLAYWRIGHT_MODULE. Keep new temporary files/builds on O: under current team resource limits.

```sh
node scripts/base64-encode-oracle.mjs --decode
node scripts/base64-encode-oracle.mjs
WASM_TEST=wasm_base64_decode node scripts/browser-host-check.mjs
WASM_TEST=wasm_base64_encode node scripts/browser-host-check.mjs
WASM_TEST=wasm_clipboard_values node scripts/browser-host-check.mjs
WASM_TEST=wasm_utility_bindings node scripts/browser-host-check.mjs
cargo test -j2 -p xterm-addons --test utility_clipboard --test utility_oracle --test utility_live_terminal
cargo clippy -j2 -p xterm-web --target wasm32-unknown-unknown --test wasm_base64_decode --test wasm_base64_encode --test wasm_clipboard_values --test wasm_utility_bindings -- -D warnings
cargo fmt --all -- --check
```

Use `--generate` only after reviewing a real trace/source change. Never alter the original source or assertions to obtain a pass.
