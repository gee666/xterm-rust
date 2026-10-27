---
name: link-check
description: Use when checking OSC 8 range discovery, provider reply ordering, hover decisions, render debouncing, or browser URL/activation behavior against read-only xterm.js.
---

# Link verification

Run from the repository root:

```sh
node scripts/link-oracle.mjs
cargo test -p xterm-web --test links --test link_state --test osc_links_differential --test debounce
WASM_TEST=wasm_links node scripts/browser-host-check.mjs
```

The first command transpiles original TypeScript modules in memory, runs all seven original OscLinkProvider/Linkifier assertions unchanged, and writes source hashes/outcomes to `results/links/oracle.json`. It creates 4,097 deterministic link-range cases under `crates/xterm-web/tests/fixtures/osc-links.tsv`, using seed `0x01a8c055`.

The WASM runner owns a temporary HTTP server and closes it plus each browser in finally blocks. It uses the shared Cargo target directory and checks free space before builds. It records failed builds as failures, not browser passes. Do not create duplicate Cargo build directories or override the repository's low-disk dev/test profile settings.

Preserve source quirks: a new single-cell run at trimmed EOL does not flush, and wrapped-range probing mutates the same work cell held by the outer scan. See `results/links/counterexample-shared-work-cell.json`.

Read `docs/links.md` for exact current gaps and `docs/link-acceptance.md` for full acceptance. Native helper tests are not proof of full DOM/terminal integration. Never edit expected fixtures to conceal a mismatch.
