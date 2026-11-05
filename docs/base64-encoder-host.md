# Base64 encoder host calls

Overall equivalence is NOT PROVEN. The full user scope remains in `docs/intent.md` and `PORT_REQUIREMENTS.md`. Task418 closes the TextEncoder and fallback host-call gap documented by task394. Full utility/status closure323, original suites, demo interactions and platform quality gates remain open.

## Behavior and implementation

The browser Base64 export had replaced `new TextEncoder().encode(data)` with Rust UTF-16 replacement and UTF-8 conversion. Normal strings matched, but this bypassed constructor/method getters, receiver identity, exceptions and raw input/result values. The fallback also bypassed mutable `String.fromCharCode`, length/index getters and their coercions.

The repair keeps those standard browser calls at the FFI boundary. Rust still chooses modern versus fallback encoding, controls the fallback loop, parses OSC52, formats replies and resumes the Rust parser. The FFI functions each perform a host operation, not terminal-engine logic. Native encoding is unchanged. No dependency or unsafe code was added.

The browser method preserves a replaced encoder's raw result instead of downcasting it to Uint8Array. Capability lookup still happens twice when present. The fallback reuses the decoder's raw `index < value.length` helper and performs the source byte lookup, String.fromCharCode call and string concatenation in their original evaluation order. Exceptions propagate unchanged. A malicious nonterminating custom length can still reproduce the original loop's nontermination; no such input is run by the bounded regression corpus.

## Evidence

`crates/xterm-web/tests/fixtures/base64-encode-host.js` supplies20 deterministic traces shared by unchanged source and Rust. They cover constructor/encode lookup and receivers, original thrown objects, nonconstructible/noncallable hosts, unconverted Symbol and undefined inputs, null/numeric/custom byte results, changing capability getters, length/index ordering and coercion, BigInt length, mutable String.fromCharCode, return coercion and empty fallback. Host overrides are restored in finally.

`scripts/base64-encode-oracle.mjs --encode-host` reuses the existing source runner. `results/base64-encode-host/source.json` records20 discovered/executed/passed, zero failed/skipped/unrun, all776 reference entries unchanged before and after, and source/fixture/trace SHA-256 hashes. Existing source traces also pass16 encoding,25 decoding and25 fallback cases. These86 cases are source-method checks, not the complete original addon suite.

The new `wasm_base64_encoder_host` suite has two test functions. One compares all20 shared traces. The other loads ClipboardAddon into a real Rust Terminal, parses a queued OSC52 query and verifies that the custom encoder receives the exact raw provider object, with ordered host calls and the exact outgoing reply.

Native Windows evidence at base main85e0fd43 uses Node24.15.0, Rust/cargo1.85.1 and wasm-bindgen0.2.100:

- `before-final.json` retains both final regression functions failing in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. The old live binding replies with base64 of `[object Object]` and never constructs the custom TextEncoder. The first trace test records all actual/expected mismatches.
- `before.json` is an earlier baseline with the same20 trace assertions. Its initial live test also replaced String.fromCharCode globally, affecting unrelated terminal UTF-16 conversion. The final live test avoids that interference by replacing TextEncoder only; no source trace assertion changed.
- `after.json` passes both new functions in each engine, including60 source-matched traces and3 actual Rust OSC52 executions.
- Existing `wasm_base64_encode`, `wasm_base64_decode`, `wasm_base64_fallback`, `wasm_clipboard_values` and `wasm_utility_bindings` pass2,2,2,5,3 functions per engine. Together with the new suite,48 browser test executions pass, zero fail/ignore.
- `native.log` records10 passing utility_clipboard/utility_live_terminal/utility_oracle tests. `clippy-wasm.log` records warning-denied Clippy passing for all six browser suites.
- Exact `cargo fmt --all -- --check` fails with Windows OS206 in `fmt.log`. All five packages pass separate checks in `fmt-packages.log`. This is not an exact full-command pass.
- `mistyped-suite.json` retains one failed build invocation using nonexistent `wasm_base64_decode_fallback`. The correct unchanged target is `wasm_base64_fallback`; its subsequent three-browser pass is retained separately. No test was disabled to fix the command.

Reports contain historical paths and digests for disposable compiler/browser artifacts, not committed distributions. Full workspace all-feature quality, native Linux/macOS runtime, unchanged original addon suites and browser/demo closure are not established by these scoped results.

## Reproduce

Use the existing source-only esbuild/Playwright dependencies and wasm-bindgen0.2.100 via SOURCE_ROOT, PLAYWRIGHT_MODULE and WASM_BINDGEN. Set CARGO_BUILD_JOBS=2 and keep all scratch/private targets on O: under the team heavy-job limit.

```sh
node scripts/base64-encode-oracle.mjs --encode-host
node scripts/base64-encode-oracle.mjs
node scripts/base64-encode-oracle.mjs --decode
node scripts/base64-encode-oracle.mjs --fallback
WASM_TEST=wasm_base64_encoder_host node scripts/browser-host-check.mjs
WASM_TEST=wasm_base64_encode node scripts/browser-host-check.mjs
WASM_TEST=wasm_base64_decode node scripts/browser-host-check.mjs
WASM_TEST=wasm_base64_fallback node scripts/browser-host-check.mjs
WASM_TEST=wasm_clipboard_values node scripts/browser-host-check.mjs
WASM_TEST=wasm_utility_bindings node scripts/browser-host-check.mjs
cargo test -j2 -p xterm-addons --test utility_clipboard --test utility_oracle --test utility_live_terminal
cargo clippy -j2 -p xterm-web --target wasm32-unknown-unknown --test wasm_base64_encoder_host --test wasm_base64_encode --test wasm_base64_decode --test wasm_base64_fallback --test wasm_clipboard_values --test wasm_utility_bindings -- -D warnings
cargo fmt --all -- --check
```

The oracle removes its scratch bundle in finally and the browser runner closes browsers/server in finally. Task418 owns only `O:/xterm-rust-swarm-16/tmp/R418`, including its regular private target, scratch and browser artifacts. Preserve committed regressions and concise reports before safe owned cleanup. Canonical integration is coordinated by309, not implied by a producer pass.
