---
name: utility-addons-check
description: Use when verifying attach, clipboard, fit and web-links Rust addon behavior against unchanged source methods and real browser components.
---

Run `node scripts/utility-addons-oracle.mjs` with Node24 to check the fixed seed1112888393 corpus. The script transforms TypeScript syntax, removes type imports and supplies the referenced scrollbar constant without changing source method bodies. Use `--generate` only after reviewing source changes. Source/corpus hashes and exact counts are in results/utility-addons/oracle.json.

Run `cargo test -p xterm-addons --test utility_attach --test utility_clipboard --test utility_oracle --test utility_live_terminal` for portable and real Rust parser integration. Run `node scripts/utility-addon-browser-check.mjs` with cargo on PATH for wasm_utility_addons via browser-host-check. It starts an ephemeral WebSocket peer and closes it in finally. Browser and WASM prerequisites and target-specific logs are described by browser-host-check skill. Do not run the socket test without the wrapper, which supplies its peer URL at compile time.

Respect .cargo/config.toml shared dev/test artifact settings and inspect free disk before large builds. Do not create separate target directories. Record failed builds during shared API edits as failures, not passing runtime checks.

Read docs/utility-addons-acceptance.md. Current component assertions are not the unchanged full browser suites or proof of complete exported Terminal integration. Keep these missing checks explicit.
