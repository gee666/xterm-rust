---
name: public-adapter-check
description: Use when checking common public buffer, parser and addon adapters against source assertions and real Rust consumers.
---

# Public adapter checks

Run focused commands in your worktree. See the shared worktree skill for setup and merge-back.

- `node scripts/check-public-adapters.mjs` runs the two source oracles and seven explicit native test targets. `node --test scripts/check-public-adapters.test.mjs` checks its startup failure handling without a compiler.
- `node scripts/public-parser-identifiers-oracle.mjs --reuse-built` verifies the 195 getter/proxy observations. It verifies embedded source-map contents before reusing the original generated JS. Omit `--reuse-built` to build the original reference afresh.
- `WASM_TEST=public_adapters_wasm node scripts/browser-host-check.mjs` runs the grouped Rust/WASM cases in Chromium, Firefox and WebKit. Actual Terminal DOM assertions stay in xterm-web.
- `node scripts/public-parser-browser.mjs` executes unchanged source Parser.test.ts and Terminal.test.ts loadAddon assertions against the Rust facade. Its transport fixture and omitted unrelated Unicode-graphemes setup are documented in the report.

Regenerate golden data only with the appropriate oracle's `--generate` flag. Do not edit expected values to make a test pass. The source oracle is not Rust coverage, and component tests do not prove the whole terminal. Report the command, result and any concrete remaining failure briefly. See `docs/public-adapters.md` and `docs/acceptance/public-adapters.md`.
