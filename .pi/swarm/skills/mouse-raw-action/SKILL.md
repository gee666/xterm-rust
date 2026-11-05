---
name: mouse-raw-action
description: Use when validating native numeric mouse action representation, strict comparisons, bitwise encodings and normalized report routing against unchanged source.
---

# Numeric mouse action checks

Run `node scripts/mouse-raw-action-oracle.mjs` from the repository root. It reuses `scripts/mouse-reference-module.mjs` for original MouseStateService behavior and executes unchanged MouseService normalized-report method bodies. Existing esbuild dependencies must resolve from the retained reference; a temporary physical reference copy can use `NODE_PATH` pointing at that existing node_modules. Do not install or link duplicate dependencies merely to run this check.

The core fixture contains 19,328 rows across all builtin protocols/encodings, 36 action boundaries and 2,048 seeded actions at `0x524ac710`. The report fixture has 2,160 rows. `--generate` explicitly regenerates expectations; inspect original source hashes and changes first. Normal replay refuses fixture drift. Source reports are retained under `results/mouse-raw-actions/`.

With an explicit coordinated heavy-job grant and private O: target/TEMP/TMP, run serially at Cargo `-j2`:

```sh
cargo test -p xterm-core --test mouse_action_numeric --test mouse_differential
cargo test -p xterm-web --test mouse_action_numeric --test mouse_policy --test mouse_routing --test mouse_failures
```

Run affected crates/all-target/all-feature tests, warning-denied native and WASM library Clippy and required formatting gates before review. Record exact outcomes and failures rather than calling this a full platform/browser pass. Native tests do not launch browsers and do not prove ordinary-object registries or real DOM routing. See `docs/mouse-action-numbers.md` for contracts and limits.
