# WASM fixture type contracts

Task578 repairs stale Rust test bindings before the next native quality checkpoint. It does not implement new terminal functionality or establish full equivalence.

## Changes and unchanged assertions

The real Terminal custom-wheel API accepts `JsValue` and returns `Result<(), JsValue>`. Five retained callsites still passed `js_sys::Function`. On main14139449, the exact focused WASM check failed with five E0308 diagnostics. The raw failed command remains `results/quality/wheel578/old-wheel-check.log` and JSON receipt.

Each callsite now converts its existing cloned Function handle with `into()` and asserts successful registration with `unwrap()`. This conversion preserves the JavaScript function identity and does not coerce the callback. The existing five test functions, loops, all22 assertion macros and every string literal remain unchanged. The new successful-registration checks do not replace any existing assertion.

Three ignored `AttachAddon.dispose()` Results in `wasm_utility_bindings.rs` and `wasm_socket_view.rs` now assert success. The clipboard addon's unit-returning disposal calls are untouched. These changes expose failures rather than suppress warnings or discard errors. No production source, API, source_repo assertion, feature or target was removed.

`results/quality/wheel578/audit-fixture-assertions.mjs` compares the complete retained before-fixtures against current source. It permits only those five argument conversions/eight successful-Result checks and formatting, and separately compares every string literal. Run it without `--write` to verify the sealed JSON receipt.

## Separate accepted prerequisite

The broader WASM check initially exposed an existing E0425 in the CFF2 native predecessor wrapper. That fix already belongs to accepted559, not578. With coordinator approval, the validation branch privately picked559 originals `aa887ab6` and `5b74089c`, producing private copies `7779796c` and `94880dfd`. They are excluded from this producer's pickup chain. No duplicate font fix was written or credited here.

The all-target passing checks therefore describe main141 plus the578 fixture change and already-accepted559 union. Bare main141 still needs559. The next integration already owns the originals. The only578 Rust source delta is the three named WASM test files, committed in `8283a272`.

## Actual verification

Native Windows11 x64, Node24.15.0 and Rust/Cargo1.85.1 executed the following with Cargo-j2, unchanged profiles and the existing locked dependencies:

- Focused three-file rustfmt passed.
- Focused wheel WASM check passed after the exact five-error baseline.
- Workspace all-target/all-feature WASM check passed.
- Warning-denied workspace all-target/all-feature WASM Clippy passed.
- Exact `cargo fmt --all -- --check` failed with Windows OS206 at the long local path. It remains a failure, not a per-file-formatting waiver. Task584 owns the measured hosted-root repair.

Existing real Rust/WASM tests passed in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4:

| Target | Functions per engine | Failed/page errors/unexpected console |
| --- | ---: | --- |
| wasm_terminal_wheel | 5 | 0 |
| wasm_utility_bindings | 3 | 0 |
| wasm_socket_view | 1 | 0 |

These are nine retained Rust test functions per engine, not nine unchanged upstream suites or a new API-coverage total. The socket test used the existing ephemeral real WebSocket peer. Original loops, cancellation checks, raw return classes, thrown identities, recursive dispatch and queued socket-view assertions all ran. No tests were ignored or filtered. Source snapshots stayed identical before/after each target; previous latest-result pointers were restored exactly.

Compiler WASM, served WASM, bindings and snippets remain pinned in `runtime-pins.json`. Concise raw reports/logs are copied losslessly under `results/quality/wheel578/raw/`. `verification.json` names the exact source/prerequisite union and three own source hashes. Generated binaries remain temporary and are not distributed as source artifacts.

The browser driver used the already-reviewed562 Windows Job adapter only for process containment, copied from its preserved archive into owned tmp. That adapter is not a Rust source prerequisite or terminal engine. Its exact bytes/origin are recorded in each browser summary. No new dependency installation occurred. The one sequential578 allocation was explicitly released after all three targets.

## Reproduce after the accepted union is integrated

With the repository's documented Rust/WASM, wasm-bindgen0.2.100 and matching Playwright prerequisites:

```sh
cargo check --workspace --locked --all-targets --all-features --target wasm32-unknown-unknown -j2
cargo clippy --workspace --locked --all-targets --all-features --target wasm32-unknown-unknown -j2 -- -D warnings
CARGO_FEATURES=acceptance-hooks WASM_TEST=wasm_terminal_wheel node scripts/browser-host-check.mjs
CARGO_FEATURES=acceptance-hooks WASM_TEST=wasm_utility_bindings node scripts/utility-addon-browser-check.mjs
CARGO_FEATURES=acceptance-hooks WASM_TEST=wasm_socket_view node scripts/utility-addon-browser-check.mjs
node results/quality/wheel578/audit-fixture-assertions.mjs
```

Use owned O: targets/scratch, bounded process containment and a current resource grant. Supply `WASM_BINDGEN`, `PLAYWRIGHT_MODULE` and `NODE_PATH` when reusing an external installed tool/dependency tree. Do not turn the old failed compiler receipt into a passing baseline or rerun old cleanup paths.

## Retained development failures and remaining work

An initial integrity-harness pattern omitted Rust trailing commas and failed before running compiler gates. A later inline version had a regex SyntaxError before evaluation; the shell continued to the planned source commit/private prerequisite picks. Those actual side effects were inspected rather than replayed. The persistent audit now passes all three files. The original failed all-target CFF2 check, three unused-result warnings, exact five wheel errors and full-format OS206 remain retained.

Read-only preflight at main141 is retained separately under `results/quality/damaris563-preflight/`. It records no CI execution, push or dispatch. It is historical preparation, not a completed563 checkpoint or remote authorization.

Independent producer review accepted with no findings. It reran the token/literal audit and three-file Rust1.85.1 formatting check, checked all59 durable and255 runtime hashes/sizes, compared own fixtures to8283a272 and before-fixtures to14139449, and checked browser raw-report/source consistency. No compiler or browser rerun was authorized or performed. Final review receipt and actual cleanup are recorded separately, not inferred from acceptance. Full native OS/MSRV/stable, feature/release/doc/safety matrices, final source mapping and complete original/browser/demo/addon equivalence remain563/304 and full-family obligations. Overall equivalence remains NOT PROVEN.
