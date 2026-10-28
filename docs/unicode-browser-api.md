# Public Unicode browser API

Task 144 adds the missing `Terminal.unicode` binding to the real Rust service. The work starts from main b6ed3ff, which includes the coherent browser producers. It does not complete task 31 or task 46, and full equivalence remains NOT PROVEN.

## Contract and implementation

- Each getter checks the current proposed-API option and returns a fresh view. Retained views continue to use the same service after options change or terminal disposal, as upstream.
- `register`, `versions` and `activeVersion` are implemented by `crates/xterm-wasm-support/src/public_api/unicode_api.rs`. The browser getter only obtains the native service and constructs the view.
- JavaScript provider methods are read live, receive their original receiver, and can throw or reenter. The portable core has a fallible host-provider interface alongside the existing infallible native interface. No service borrow crosses a host callback. Failures travel through the existing callback error chain and recover the original thrown JS value.
- Replacing a registered provider does not change the active provider snapshot until reselection. Initial activation deliberately reads the provider version twice. Failure on the second read leaves the first insertion visible.
- Rust's native service still uses String keys. At the WASM boundary, ill-formed UTF-16 names and names beginning with the reserved U+F0000 prefix are encoded as that prefix followed by four hex digits per UTF-16 unit. All other names stay unchanged, preserving numeric key ordering. Public getters and unknown-version errors decode the representation. Native code inspecting the service sees encoded keys for those names. Builtin addon names are unchanged.
- Host `wcwidth` returns the declared width values 0, 1 or 2. Other return types are rejected at the Rust type boundary. Packed `charProperties` retains JavaScript's 32-bit bitwise conversion.

## Reproduction

Use the existing `browser-host-check` skill. Git, Cargo, wasm32-unknown-unknown, wasm-bindgen 0.2.100 and the reference Playwright dependencies are required. Set `WASM_BINDGEN` and `PLAYWRIGHT_MODULE` when they are not installed in the documented default locations.

```sh
node scripts/unicode-browser-oracle.mjs
WASM_TEST=unicode_public_api node scripts/browser-host-check.mjs
cargo test -p xterm-core --test unicode_service --test unicode_service_differential --test unicode_addons --test unicode_addon_manager --test unicode_addon_drop
cargo clippy -p xterm-wasm-support --target wasm32-unknown-unknown --lib -- -D warnings
node scripts/build-demo.mjs --debug
node scripts/e2e-upstream.mjs --suite addon-fit --no-build
node scripts/e2e-upstream.mjs --suite addon-unicode11 --no-build
node scripts/e2e-upstream.mjs --suite addon-unicode-graphemes --no-build
```

The oracle executes unchanged upstream Lifecycle, Event, UnicodeService and UnicodeV6 code. The shared test operation trace compares registry snapshots and errors, including high and low lone surrogates, mixed strings, a valid surrogate pair, empty/numeric names, prototype-like keys and encoding-prefix collisions. It never executes the source engine in the Rust terminal.

## Earlier task 144 Windows evidence

- Five WASM tests passed in each of Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4. Tests include source-derived registry snapshots, live provider callbacks, receiver identity, reentrant change listeners, committed state on callback errors, failed version getters, proposed gating, fresh views and actual terminal cell advancement from a JS provider.
- Five native Unicode integration targets passed 31 tests total. Their retained log is `results/unicode-browser/native.log`.
- WASM library check and warning-denied wasm-support Clippy passed. Full web Clippy fails at two existing type-complexity diagnostics in `links/animation_queue.rs` and `selection/dom.rs`. No suppression was added.
- `cargo fmt --all -- --check` cannot launch on this Windows path layout, OS error 206. A direct rustfmt check of the changed Rust modules with `skip_children=true` passes. Logs retain both outcomes.
- The unchanged Fit suite now executes all 27 cases, with 17 passed and 10 failed, no skips or unrun cases. WebKit passes all nine; Chromium and Firefox fail default/width geometry and repeat-fit stability. This is progress beyond the previous missing-public-getter setup failure, not Fit equivalence.
- Original Unicode11 and graphemes browser suites each execute three cases and fail all three after their public Unicode assertions. They access `term._core.unicodeService.getStringCellWidth`, which has no Rust internal-test adapter yet. No source assertion was changed or removed.

Fit raw report: `results/e2e/upstream/2026-10-28T17-03-31-150Z-dc35bd61-3433-477f-8344-bded838c3ee6/`.
Unicode11 raw report: `results/e2e/upstream/2026-10-28T17-06-08-703Z-d29abbd7-85fe-4574-8c02-49f60400e595/`.
Graphemes raw report: `results/e2e/upstream/2026-10-28T17-06-16-079Z-0846f3f2-e21f-4de8-a37b-c7c08f7d74a9/`.

The browser-host result JSON and per-engine logs identify that component run. Generated WASM and large traces stay local. Native Linux/macOS, complete original browser suites, other missing addon bindings, fit geometry and the full E01-E05 matrix remain open.

## Task 284 internal width binding

The real browser Terminal now retains a stable `_core` view with a stable
`unicodeService` width view. Its `getStringCellWidth` method calls the same Rust
UnicodeService used by public registration, addon selection and printing.
`JsString` becomes UTF-16 units without a lossy Rust String conversion.
Provider callbacks keep their receiver and original thrown value. No source
engine runs behind this binding, and no bootstrap JS changes were needed.

This is a narrow internal entry point, not a complete CoreTerminal ABI.
Other internal services and methods are absent rather than nominal placeholders.
The internal width view is not proposed-API gated. Retained views continue to
use the same service after terminal disposal, matching the underlying service
lifetime; obtaining the public Unicode view remains gated.

The existing source oracle now also executes `unicodeWidthTrace` against the
unchanged upstream service and verifies `tests/fixtures/unicode-width-api.json`.
The WASM test runs that identical trace against the real binding. It checks
malformed and trailing surrogates, second-unit width callback order, live
provider changes, selected-provider replacement snapshots, callback receiver
and error identity, recovery and switching providers during a width callback.
Initial draft tests had two incorrect handwritten expectations, V6 emoji width
and version-getter count. Those were replaced with source-derived expectations;
the failed draft receipts remain available.

Commands, in addition to the prerequisites above:

```sh
node scripts/unicode-browser-oracle.mjs
WASM_TEST=unicode_internal_service node scripts/browser-host-check.mjs
WASM_TEST=unicode_public_api node scripts/browser-host-check.mjs
node scripts/build-demo.mjs --debug
node scripts/e2e-upstream.mjs --suite addon-unicode11 --no-build
node scripts/e2e-upstream.mjs --suite addon-unicode-graphemes --no-build
```

The unchanged original baseline passed one case per addon in each engine, as
retained in `docs/upstream-browser-baseline.md` and its complete matrix receipt.
The new Rust execution passes the same two suites: three cases per suite,
three executed and passed, zero failed/skipped/unrun, covering all original
Chromium, FirefoxStable and WebKit projects. The original assertions and source
reference remain unchanged. Fresh reports are under `results/e2e/upstream/`:

- Unicode11: `2026-10-28T17-31-18-866Z-4bbcdb09-6af3-4632-892b-b237ac07ba5d/`.
- Graphemes: `2026-10-28T17-31-23-700Z-46c6b782-9487-42bd-89a5-fa203c0d1261/`.

Two internal-service WASM tests and five existing public-API WASM tests pass in
each of Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4. Five focused native
targets pass 31 tests. Warning-denied WASM library Clippy passes for both
xterm-wasm-support and xterm-web on this main checkpoint. Targeted rustfmt
passes; workspace-wide cargo fmt still cannot launch due to Windows OS206.
Logs are retained under `results/unicode-browser-internal/`, with component
receipts under `results/browser-host/`.

The wider WASM all-targets/all-features Clippy attempt fails outside this
change: stale wasm_input host trait implementations, unsupported Function.call4
in wasm_search_addon, and an excessive-precision literal in width-cache tests.
The complete diagnostics are retained in `all-wasm-clippy.log`; task47 owns
these workspace quality gaps. No warnings or assertions were suppressed.

These results close the missing internal-width setup failure from task144.
They do not close task31, task46, Fit geometry, complete internal API coverage,
the full E01-E05 matrix or native Linux/macOS verification. Full equivalence
remains NOT PROVEN.
