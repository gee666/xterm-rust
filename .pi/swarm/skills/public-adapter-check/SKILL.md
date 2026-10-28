---
name: public-adapter-check
description: Use when checking common public buffer/parser/addon adapters against original addon assertions and focused Rust integration tests, retaining failure logs and exact case counts.
---

# Public adapter checks

Run `node scripts/check-public-adapters.mjs` from the repository root with a tool timeout above 120 seconds. The script bounds each child at 120 seconds, finds Cargo through CARGO, CARGO_HOME or the user's cargo bin, and writes non-secret logs plus accounting to `results/public-adapters/verification.json`.

It first runs `scripts/public-addon-oracle.mjs`, which bundles unchanged reference AddonManager source and its original two-case suite with the existing esbuild/Mocha dependencies under source_repo/node_modules. Generated output goes only under root tmp/. The script records source hashes, original case titles and directed assertion checks, including reentrant index failures. No original assertion is removed.

Then it runs actual xterm-core integration tests `public_addon_manager`, `public_addon_reentrancy`, `public_buffer_api` and `public_parser_api`. Compile errors are command failures with zero executed cases and all discovered cases unrun, never passes. Native buffer tests use the real BufferSet and parser tests the real EscapeSequenceParser.

See `docs/acceptance/public-adapters.md` for completion gates. This command does not claim original headless/browser groups, WASM bindings, all Unicode/number boundaries or Linux/macOS execution passed. Retain those as explicit gaps until separate observed evidence exists. Preserve `.cargo/config.toml` and monitor free disk before building.
