# Marker registered resources

Task385 implements the supplied Marker.register lifecycle through the actual
Rust-backed browser Terminal. This is a scoped addition. Full marker fields,
mutable marker arrays and public decoration bindings remain task331 work.
Overall equivalence is NOT PROVEN.

## Contract

`BrowserMarker.register(value)` stores any JavaScript value without inspecting
it and returns that exact value. During marker disposal the adapter reads the
current `value.dispose` member and invokes it with `value` as receiver. The
small JavaScript helper performs that member call only. Rust owns retention,
traversal and invalidation; no JavaScript terminal engine runs in
the Rust tests.

The core's `MarkerResource` callback returns `Result<(), EmitError>`. Existing
native `DisposableHandle` resources and these fallible host callbacks share one
ordered resource list. Traversal releases the list borrow before callbacks, sees
resources appended by listeners or other resources, and retains duplicates.
Marker invalidation and onDispose happen before resource cleanup. A resource
error stops cleanup with its original error, retaining the resource list until
marker destruction. Later dispose calls do not retry it. Registrations after
disposal remain retained without immediate invocation, matching the source.
Successful cleanup releases the retained list. The native API has no browser or
OS dependency and adds no package dependency or unsafe code.

## Verification

The unchanged reference files are `src/common/buffer/Marker.ts` and
`src/common/Lifecycle.ts`. The shared browser fixture adds one source-derived
registered-resource case, not an unchanged upstream test. Source and Rust retain
independent expected captures in the existing oracle and Rust test respectively.
The full scoped marker suite now has four cases per engine. Its previous three
registration/handle/buffer cases remain intact.

Fresh Windows results on base1729d89:

- Native Marker resources3/3 and existing BufferRange/Marker15/15 pass.
- Source-derived browser cases4/4 in each of Chromium148.0.7778.96,
  Firefox150.0.2 and WebKit26.4 pass, totaling12 executions.
- Actual Rust/WASM marker tests4/4 in each of those engines pass, totaling12
  executions, zero failed/ignored/filtered cases.
- Exact full-workspace formatting check passes. Full native all-target/all-feature
  workspace Clippy and focused native/WASM Clippy pass with warnings denied.
- Native all-target/all-feature workspace tests pass2046 test functions across402
  summaries on this Windows checkout. This is not source assertion mapping or
  native Linux/macOS evidence.

The new assertions cover raw return identity, lazy member access, changed
methods, receiver identity, duplicate resources, additions during onDispose and
resource traversal, reentrant dispose, late registrations, thrown getter/method
object identity, null/noncallable resource TypeErrors, listener failure before
cleanup, no retry after error, and native retention/release counts.

Use the existing installed wasm-bindgen0.2.100 and reference Playwright module.
Set `REFERENCE_ROOT`, `PLAYWRIGHT_MODULE` and `WASM_BINDGEN` when they are outside
the checkout. Set TEMP/TMP to an owned O: scratch directory and
CARGO_BUILD_JOBS=2. Commands and hashes are retained in
`results/marker-resources/checks.json`, `source.json`, `rust.json` and logs.

```sh
cargo test -p xterm-core --test marker_resources --test buffer_range_marker -j2
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -j2 -- -D warnings
cargo test --workspace --all-targets --all-features -j2
cargo clippy -p xterm-core --test marker_resources --test buffer_range_marker -j2 -- -D warnings
cargo clippy -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_terminal_markers -j2 -- -D warnings
MARKER_ORACLE_REPORT=results/marker-resources/source.json node scripts/terminal-marker-oracle.mjs
WASM_TEST=wasm_terminal_markers node scripts/browser-host-check.mjs
```

Generated WASM/binding paths in the report identify the tested bytes. Those
files and the private Cargo target are disposable after review; their historical
absolute paths need not remain present. Native Linux/macOS, the complete original
browser suites and writable raw marker/decorations contracts are not established
by these scoped results.
