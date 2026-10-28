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

## Observed Windows evidence

- Five WASM tests passed in each of Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4. Tests include source-derived registry snapshots, live provider callbacks, receiver identity, reentrant change listeners, committed state on callback errors, failed version getters, proposed gating, fresh views and actual terminal cell advancement from a JS provider.
- Five native Unicode integration targets passed 31 tests total. Their retained log is `results/unicode-browser/native.log`.
- WASM library check and warning-denied wasm-support Clippy passed. Full web Clippy fails at two existing type-complexity diagnostics in `links/animation_queue.rs` and `selection/dom.rs`. No suppression was added.
- `cargo fmt --all -- --check` cannot launch on this Windows path layout, OS error 206. A direct rustfmt check of the changed Rust modules with `skip_children=true` passes. Logs retain both outcomes.
- The unchanged Fit suite now executes all 27 cases, with 17 passed and 10 failed, no skips or unrun cases. WebKit passes all nine; Chromium and Firefox fail default/width geometry and repeat-fit stability. This is progress beyond the previous missing-public-getter setup failure, not Fit equivalence.
- Original Unicode11 and graphemes browser suites each execute three cases and fail all three after their public Unicode assertions. They access `term._core.unicodeService.getStringCellWidth`, which has no Rust internal-test adapter yet. No source assertion was changed or removed.

Fit raw report: `results/e2e/upstream/2026-10-28T17-03-31-150Z-dc35bd61-3433-477f-8344-bded838c3ee6/`.
Unicode11 raw report: `results/e2e/upstream/2026-10-28T17-06-08-703Z-d29abbd7-85fe-4574-8c02-49f60400e595/`.
Graphemes raw report: `results/e2e/upstream/2026-10-28T17-06-16-079Z-0846f3f2-e21f-4de8-a37b-c7c08f7d74a9/`.

The browser-host result JSON and per-engine logs identify the final component run. Generated WASM and large traces stay local. Native Linux/macOS, complete original browser suites, other missing addon bindings, fit geometry and the full E01-E05 matrix remain open.
