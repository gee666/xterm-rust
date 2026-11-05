# Primitive OSC link handlers

Task396 repairs `terminal/link_services.rs`, the production registry used by exported Terminal. The original accepts raw `linkHandler` values. Its policy, hover and leave use optional chaining, while activation chooses its fallback with a truthy conditional. Those are different operations.

The old binding discarded falsey handlers outright. It also read nested properties through the portable option model, which does not represent browser primitive prototypes. For opaque BigInt/Symbol values, that adapter boxed the receiver before invoking getters. A strict getter could therefore observe an object instead of its original primitive.

The binding now samples the raw handler once per existing row and uses a small JavaScript property-access binding. It performs only `value[key]`; Rust still parses OSC8, discovers ranges, applies protocol policy, manages link state and dispatches callbacks. The binding preserves inherited getters, primitive receivers and original thrown values. Null/undefined handlers retain the default behavior. Falsey non-nullish handlers can supply policy/hover/leave through their prototypes, but activation still uses the confirmation fallback. No new dependency or terminal-engine delegation was added.

## Evidence and scope

`scripts/osc-handler-trace.js` runs the same31 directed scenarios against the original production Terminal's first OSC provider and the Rust production registry over native terminal state. It covers Boolean, Number including negative zero and NaN, String, BigInt and Symbol values; prototype policy/method getters; strict receiver identity; nullish methods; noncallable methods; exact thrown-object identity; shared mutable callback range identity; and default confirmation. The original production bundle's embedded OscLinkProvider/OptionsService source was checked against its read-only TypeScript files before execution.

All31 source scenarios matched across Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4 on Windows. The source additionally confirms two circular-list missing-row setups in each browser. The Rust regression compares the entire source fixture. On unchanged production code at main e7cea61 plus test commit0c81017, Chromium failed: primitive links were omitted and BigInt/Symbol getter receivers were boxed. The patched regression passed1/1 in each engine, covering93 scenario comparisons. Baseline Firefox/WebKit were not run. These are custom differentials, not31 unchanged upstream test definitions.

A second shared program, `osc-handler-terminal-trace.js`, uses actual DOM mousemove/down/up/leave through an opened Terminal with `linkHandler: false`. It checks native OSC8 discovery, inherited hover/leave callbacks, default activation confirmation and pointer appearance/removal. Its source trace matched all three engines. The new Rust Terminal test failed on unchanged production code in Chromium, while the two existing Terminal link tests passed. With the fix, all three Terminal tests passed per engine. The baseline used the identical new Terminal test and trace; its only production difference was restoring `link_services.rs` from0c81017.

The DOM probe dispatches on the screen so source listeners receive events. It checks the pointer class on either screen or root because current Rust applies it to the root and source applies it to the screen. Exact class placement remains an explicit full-family321 gap. This test does not establish complete DOM, live-link-object mutation, public provider-registration, descriptor or GC equivalence.

Existing Linkifier tests passed6/6 per engine. Native link/debounce suites passed36 test functions across8 summaries, including their existing seeded differential loops. Native xterm-web all-target/all-feature Clippy, scoped WASM library/four-suite Clippy with warnings denied, full workspace rustfmt and script syntax checks passed. This is Windows/MSRV1.85.1 evidence, not the full workspace release/platform gate. No new unsafe code was added.

## Repair to an existing regression setup

`wasm_osc_policy` initially failed in every browser before reaching its intended protocol assertions. An unchanged-production Chromium rerun failed identically. It requested row99 from a full zero-scrollback circular list and expected a missing row, but the original circular list aliases that index to an existing backing slot. The original source trace confirms that scrollback0 returns an empty array and scrollback10 returns undefined for the same row99.

The Rust test now leaves ten scrollback slots, so its requested index is genuinely absent. No assertion was removed or relaxed. It now passes1/1 per engine, including the existing missing/empty-row checks, per-run policy getter counts, exact error identity and callback suppression. Both failed attempts and the corrected pass remain in the run index.

## Reproduction

Use Rust/Cargo1.85.1, Node24.15.0 and locked wasm-bindgen CLI0.2.100. Set `SOURCE_ROOT` to the read-only built original reference and `PLAYWRIGHT_MODULE` to its installed Playwright entry. Put Cargo targets and TEMP/TMP/TMPDIR under an owned O: directory and coordinate one compiler/browser slot. The browser runner owns its server and browser cleanup.

```sh
node scripts/osc-handler-source.mjs
# Only to regenerate independent source snapshots:
node scripts/osc-handler-source.mjs --generate
WASM_TEST=wasm_osc_handler node scripts/browser-host-check.mjs
WASM_TEST=wasm_terminal_links node scripts/browser-host-check.mjs
WASM_TEST=wasm_linkifier node scripts/browser-host-check.mjs
WASM_TEST=wasm_osc_policy node scripts/browser-host-check.mjs
cargo test --locked -j2 -p xterm-web --test links --test link_state --test osc_links_differential --test osc_links_live --test debounce --test debounce_errors --test debounce_differential --test link_replies_differential
cargo clippy --locked -j2 -p xterm-web --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_osc_handler --test wasm_osc_policy --test wasm_terminal_links --test wasm_linkifier -- -D warnings
cargo fmt --all -- --check
```

`results/osc-handler/` retains source hashes/traces, the browser run index and raw reports/logs, native/lint logs and final owned-file hashes. Reports retain removed binary paths and their hashes as provenance; disposable binaries are not checked in. The generated source fixture/report exceed350 lines because they record complete independent scenario data. The first live-source probe incorrectly dispatched on the outer root and saw no callbacks; correcting its event target to the actual screen did not alter production source or existing assertions.

Full xterm.js equivalence remains NOT PROVEN. Full315/321 work, unchanged applicable suites, workspace quality and native Linux/macOS evidence remain open.
