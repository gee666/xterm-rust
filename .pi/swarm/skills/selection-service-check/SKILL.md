---
name: selection-service-check
description: Use when checking selection words, signed ranges, text extraction, pointer policies or the SelectionDom browser component against xterm.js.
---

# Selection service check

From the repository root:

```text
node scripts/selection-service-oracle.mjs
cargo test -p xterm-web --test selection_model --test selection_service --test selection_service_differential
cargo clippy -p xterm-web --test selection_model --test selection_service --test selection_service_differential --example selection_oracle -- -D warnings
```

Read docs/selection-service.md for exact accounting. The oracle loads actual unchanged TypeScript classes and all 26 source unit assertions, then checks the retained TSV of source operations and seeded traces. The source subclass disables DOM interaction. A native replay is not a browser pass.

For browser component tests, reuse the browser-host-check skill:

```text
WASM_TEST=wasm_selection node scripts/browser-host-check.mjs
```

Do not run builds during a coordinated disk pause. Keep the shared target directory and repository profiles. The harness checks disk headroom, bounds builds, serves only Rust test bindings, and closes its browsers/server.

After a reviewed source change, `node scripts/selection-service-oracle.mjs --generate` regenerates fixtures. Do not change expected values merely to hide a Rust failure. Preserve counterexamples under results/selection-service/counterexamples/. Distinguish actual source unit cases, recorded operations, seeded traces, native functions and browser assertions in reports.
