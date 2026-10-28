# Options verification

Full-port equivalence is not proven. These checks cover the shared options component, not complete Terminal rendering or cross-GC ownership.

## Implementation

- `crates/xterm-core/src/options/` owns 48 defaults, validation, UTF-16 values, event ordering and native storage.
- Native default bags are shared and mutable. Nested writes do not emit root option-change events. `WindowsPty::into` builds a mutable property bag; explicit legacy opaque WindowsPty payloads remain readable by legacy consumers.
- `crates/xterm-wasm-support/src/options.rs` uses actual JS constructor/prototype/descriptor mechanics, with one Rust sanitization pass. Old web adapter paths reexport this implementation.
- Full dynamic events use `on_any_option_change`; the older `on_option_change` is a declared-key convenience view over the same emitter.
- Native diagnostics are returned rather than written to the host console. The browser adapter preserves source console diagnostics and original thrown values.

## Checks run

In `swarm/Athena`:

- `cargo test -p xterm-core --test options_service --test options_boundary --test options_host_store --test options_properties`: 43 passed.
- `node scripts/check-options.mjs`: original 10 cases / 22 assertions passed; 48 defaults and 31,961 differential output lines matched. Seed 1330664531.
- `WASM_PACKAGE=xterm-wasm-support WASM_TEST=wasm_options node scripts/browser-host-check.mjs`: 15 passed in each of Chromium, Firefox and WebKit.
- The same browser command with `wasm_events`: 6 passed per browser. With `wasm_properties`: 1 passed per browser.
- Scoped native and WASM Clippy passed with `-D warnings`; scoped formatting checks passed.

The browser tests execute generated, unchanged OptionsService reference code separately from Rust. They compare values, property layouts, descriptors, events, getters, prototype handling, reentrancy and failures. The public descriptor loop is transcribed for that component comparison; it is not represented as a complete original Terminal engine. Actual Terminal assertions remain in xterm-web.

The original-case mapping is one-to-one in `tests/options_service.rs`: four constructor cases, two font-weight cases, one general event case, both repeated specific-event cases, and one multiple-event case.

## Fixed findings and limits

- Fixed lone-surrogate error text loss, missing coercion hints, swallowed host errors, immutable native default bags, and payload destruction under a storage borrow.
- Fixed a test-harness global-prototype leak between the sequential reference and Rust runs. No assertion was removed.
- Historical failed attempts remain under results/options and results/browser-host. Latest component summaries are `results/browser-host/xterm-wasm-support__*-results.json`.
- Native property bags are acyclic data containers, not a general JS prototype heap. Actual host graphs use PropertyAccess and retain JS graph behavior.
- Full Terminal integration, cross-GC ownership and the complete platform/feature matrix remain separate work. A support-crate pass does not establish those results.
