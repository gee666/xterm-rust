# Live browser mouse modes

Task375 exposes `_core.mouseStateService` on the real Rust browser Terminal. The stable wrapper holds the same `Rc<MouseStateService>` used by native escape-sequence handling and browser DOM input. It does not copy mouse modes into another authority or load the JavaScript engine.

The binding exposes `activeProtocol`, `activeEncoding`, `areMouseEventsActive`, `isDefaultEncoding`, `isPixelEncoding`, `onProtocolChange` and `reset`. Built-in string setters use native validation. Invalid built-in names leave the previous mode unchanged and throw the source error text. Repeated protocol assignments still emit events. Nested assignments and thrown listener values use the existing callback-safe event adapter. Retaining the wrapper across a terminal reset observes the same native service.

This is a bounded internal binding, not completion of task305. JavaScript protocol/encoding registration, prototype property names, nonstring setter inputs and raw callback/receiver semantics remain unfinished there. The string setters must not be cited as evidence for those contracts. Public marker/decoration331, other internal services and overall equivalence also remain open.

## Evidence on Windows

Base main `ff0bec8`, Rust/Cargo1.85.1, Node24.15.0. Engines were Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4.

Five shared JavaScript cases run against original source Terminal and actual Rust Terminal. Source captures agree in all three engines. The Rust WASM integration target passes all five tests in each engine, with no ignored or filtered tests. Cases cover initial service identity before open, built-in setters and errors, repeated protocol events, subscription disposal, nested mode changes, thrown-value identity, parsed DEC mouse modes, retained reset identity and reads after terminal disposal. The live DOM case activates VT200/SGR through the service setters, dispatches mouse press/release and receives exactly `ESC[<0;1;1M` and `ESC[<0;1;1m` on `onData`, with no binary output. Reset suppresses further reports.

The unchanged upstream Playwright selection `DECSET|DECRST` discovers462 cases across the original three projects. It executes and passes437, with25 original skips, zero failures and zero unrun cases. No original assertion or source file was modified. This targeted selection repairs the access failures reported by the broader task327 checkpoint. It is not a full core/addon/demo pass.

The source oracle bundles read-only `source_repo/src/browser/public/Terminal.ts` and its source imports into temporary test artifacts. Normal invocation compares captures against the retained fixture; only `--generate` updates expected captures. The shared cases and key source files have SHA256 receipts. Rust tests compare complete captured JSON objects against those source captures. The original-suite runner retains its own source/build/runtime identities.

Full workspace formatting, focused WASM Clippy with warnings denied and WASM library checks pass. These scoped checks are not the required final all-feature workspace/native-platform matrix. There is no new Linux or macOS runtime evidence.

Raw source captures, browser results/logs, original Playwright JSON and summary, build metadata and scoped quality logs are retained in `results/mouse-modes/`. Reports contain historical owned temporary paths and artifact hashes. Generated binaries and browser packages are disposable, not checked-in evidence. Rebuild them to rerun.

## Independent review

The isolated task375 reviewer accepted source commit `884e080` with no findings or file edits. It reran workspace formatting, warning-denied focused WASM Clippy and regression-target compilation. It verified the three retained source captures agree, the freshly compiled WASM hash matches the recorded five-test browser artifact, and the raw original Playwright report has462 discovered,437 passed,25 skipped and no failures or unrun cases. It did not independently rerun browsers because the resource grant was pending.

After the review grant arrived, the owner reran the tightened source script in default equality-check mode and all five Rust WASM tests in all three browsers. Both passed. The retained `source-report.json` now includes public Terminal, MouseStateService, InputHandler and shared-case hashes. The `final-*` reports/logs preserve this rerun. This post-review execution did not change production source.

## Reproduction

Use installed reference Node dependencies, Rust's `wasm32-unknown-unknown` target and wasm-bindgen CLI0.2.100. See the existing browser-host-check skill for dependency overrides. Use a private Cargo target and temporary paths as required by the current swarm work agreement.

```sh
node scripts/mouse-mode-source.mjs
WASM_TEST=wasm_terminal_mouse_modes node scripts/browser-host-check.mjs
node scripts/build-demo.mjs --debug
node scripts/e2e-upstream.mjs --suite core --no-build --grep 'DECSET|DECRST' --timeout-ms 240000
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --test wasm_terminal_mouse_modes --lib -- -D warnings
cargo fmt --all -- --check
```

`XT_REFERENCE_ROOT` can select another read-only source tree for the source capture script. It must contain the same source and compatible installed dependencies. Inspect source changes and retained failures before intentionally using `node scripts/mouse-mode-source.mjs --generate`.
