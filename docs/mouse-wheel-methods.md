# Live mouse wheel methods

Task510 adds the missing `setCustomWheelEventHandler` and `allowCustomWheelEvent` methods to the actual `_core.mouseStateService` object. The source reference is `source_repo/src/common/services/MouseStateService.ts`, with public forwarding in `src/browser/CoreBrowserTerminal.ts` and DOM calls in `src/browser/services/MouseService.ts`.

## One handler authority

The raw `_customWheelEventHandler` property on the actual JS receiver is the only browser wheel-handler slot. The previous `Rc<RefCell<Option<Function>>>` fields in `Shared` and `BrowserServices` are removed. Public `attachCustomWheelEventHandler` invokes the real service setter, and DOM wheel paths invoke the real service allow method. Replacements, accessor failures and receiver identity are therefore visible through the same object. The public Rust binding now accepts `JsValue` and returns `Result<(), JsValue>` so undefined, other raw values and synchronous exceptions are not erased at its boundary. The JS success return remains undefined.

`terminal/mouse_wheel.rs` implements the truthy guard and strict-false veto. `mouse_wheel.js` only installs prototype methods, forwards dynamic `this`, and performs ordinary property read/write/member-call host operations. Those operations preserve primitive-receiver boxing, strict assignment failures, getter exceptions and engine member-call errors. The handler property is read once for the truthy guard and again for the call. No borrow spans user callbacks, so callbacks can replace themselves and recursively invoke the same methods.

The JS-owned Rust closures are stateless and capture neither terminal nor receiver. A weak set prevents each new terminal from replacing the prototype methods, preserving shared method identity and user prototype overrides. This follows existing JS-owned closure patterns rather than introducing a second mode service. No dependency, unsafe code or terminal-engine delegation is added.

The source's uninitialized TypeScript handler declaration produces no own property. Rust likewise leaves it absent until assigned. Reset and disposal retain the handler on a retained mouse receiver. Disposal still disposes the native service emitter through existing lifecycle code. This task does not establish all cross-language garbage-collection behaviour.

## Reproduction

Use an isolated worktree and the resource owner's grant. Reuse existing reference dependencies and the matching wasm-bindgen0.2.100 CLI. Set `CARGO_BUILD_JOBS=2`, `RUSTUP_TOOLCHAIN=1.85.1`, `CARGO_TARGET_DIR` to the owned regular target directory, and TEMP/TMP to its tmp directory. `XT_REFERENCE_ROOT`, `PLAYWRIGHT_MODULE` and `WASM_BINDGEN` select existing tools without dependency installation.

```sh
MOUSE_CASE_FAMILY=wheel-methods node scripts/mouse-mode-source.mjs
WASM_TEST=wasm_terminal_mouse_wheel_methods node scripts/browser-host-check.mjs
```

Initial fixture creation used `--generate` against the unchanged source. A subsequent source run without that flag passed. The seven shared JS functions execute unchanged against original and Rust Terminals. Their complete JSON outputs are compared; fixture assertions were not weakened after a Rust failure.

## Observed results

Source base was main `e64f4870`. Durable results are in `results/mouse-wheel-methods/`.

| Check | Observed result |
| --- | --- |
| Original source capture and unchanged-fixture replay | Seven shared functions per Chromium, Firefox and WebKit, passed |
| First Rust browser run | Six passed, one failed in each browser |
| Corrected and final restored Rust runs | Seven passed in each browser, no failed/ignored/filtered cases |
| Exact old production negative | Zero passed, seven failed in Chromium |
| Existing wheel receiver suite | Three passed per browser |
| Existing mouse modes suite | Five passed per browser |
| Existing built-in policy suite | Four passed per browser |
| Existing disposal suite | Eight passed per browser |
| Existing UTF-16 mouse names suite | Five passed per browser |
| Native xterm-web all-target/all-feature tests | 406 passed across169 test summaries on Windows, no failed or ignored tests |
| Rust1.85.1 native workspace all-target/all-feature warning-denied Clippy | Passed |
| Rust1.85.1 WASM workspace all-feature library warning-denied Clippy | Passed |
| Rust1.85.1 new WASM test all-feature warning-denied Clippy | Passed |
| Each of the five package rustfmt checks | Passed |
| Exact workspace `cargo fmt --all -- --check` | Failed with Windows OS206 command-length error, retained |
| Reference integrity | 776 original files matched manifest before browser execution |

The first browser failure exposed an unintended own undefined handler property. Removing that initialization fixed the mismatch while retaining the original fixture and assertion. The initial failing report is retained alongside the corrected runs.

The negative driver replaces exactly five existing production files with their bytes from `e64f4870`, retaining the new test and fixtures. That restores the old active production module graph, without the new wheel module. It runs Chromium, expects the seven tests to fail, then restores all five fixed files in `finally` and verifies each SHA-256. The final full three-browser run follows restoration. See `negative-driver.mjs.txt`, `restoration.json`, and the retained per-run reports. The old and restored WASM outputs have distinct hashes.

Browser versions were Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. Node was24.15.0. Successful Clippy and browser builds used Rust/Cargo1.85.1 and wasm-bindgen0.2.100. Native tests also passed on Rust1.99.1, but strict workspace Clippy on1.99.1 and1.97.1 failed on an existing `font_cff2_dict.rs:179` double-ended-iterator `.last()` warning. Those logs remain failures, not waived passes. The1.97.1 WASM check also failed because that toolchain lacks the WASM target; no target was installed to disguise that failed attempt. An initial `+stable` invocation auto-installed1.99.1 into shared Rust tooling; no target was installed for it. No source change or lint suppression was made to hide the warning.

`browser-runs.json` distinguishes all nine runs and their exact counts. `source-pins.json`, `native-pins.json` and `browser-artifact-pins.json` retain source, native executable and generated browser artifact hashes. Per-run reports/build logs are copied under `browser/`; generated WASM and binding bytes remain available through independent review, then are disposable owned build outputs. Hashes are retained after cleanup, not a promise that deleted paths remain available.

## Remaining work

Raw protocol/encoding registration, inherited registry names, `__proto__`, non-string active keys and raw registry callbacks remain task305. This task changes no native mode authority and does not close the full browser ABI, all mouse behavior, original browser/demo matrix, native Linux/macOS runtime evidence, workspace format failure or full equivalence. Overall equivalence remains NOT PROVEN.
