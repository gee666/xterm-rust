---
name: utility-addons-check
description: Use when verifying attach, clipboard, fit and web-links Rust addon behavior against unchanged source methods and real browser components.
---

Run `node scripts/utility-addons-oracle.mjs` with Node24 to check the fixed seed1112888393 corpus. The script transforms TypeScript syntax, removes type imports and supplies the referenced scrollbar constant without changing source method bodies. Use `--generate` only after reviewing source changes. Source/corpus hashes and exact counts are in results/utility-addons/oracle.json.

Run `cargo test -p xterm-addons --test utility_attach --test utility_clipboard --test utility_oracle --test utility_live_terminal` for portable and real Rust parser integration. Run `node scripts/utility-addon-browser-check.mjs` with cargo on PATH for wasm_utility_addons via browser-host-check. It starts an ephemeral WebSocket peer and closes it in finally. Browser and WASM prerequisites and target-specific logs are described by browser-host-check skill. Do not run the socket test without the wrapper, which supplies its peer URL at compile time.

Use the current worktree/resource policy. For the16-agent continuation, private worktrees and regular Cargo targets live on O:, with Cargo -j2 and at most four heavy jobs/two browser matrices team-wide. Preserve dev/test profile settings. Record failed builds as failures, not passing runtime checks.

After building the actual Rust distribution, run `node scripts/fit-browser-oracle.mjs` to compare12 live geometry snapshots per engine. `--suite` additionally runs the unchanged original Fit suite against JavaScript and returns failure for any original failures. `SOURCE_ROOT` can select the existing reference with its built demo bundle/dependencies. The command removes its temporary files and closes browsers/servers. See docs/fit-browser-geometry.md for the observed Windows17/27 baseline shared by both engines. Do not force stability or change default fonts to hide source rounding behavior.

For Base64 encoding host compatibility, run `node scripts/base64-encode-oracle.mjs` and `WASM_TEST=wasm_base64_encode node scripts/browser-host-check.mjs`. The shared16-case fixture checks capability getter order, typed-array mutation, receiver, raw result and exception identity against unchanged source. A second browser assertion checks actual OSC52 reply interpolation. `SOURCE_ROOT` selects the unchanged reference with esbuild; `WASM_BINDGEN` and `PLAYWRIGHT_MODULE` may select existing tools. Keep baseline failures and scoped counts under results/base64-encode. See docs/base64-encode-host.md. Do not label this the full original clipboard suite.

Read docs/utility-addons-acceptance.md. Focused geometry/component assertions are not the full browser suites or proof of complete exported Terminal integration. Keep missing checks explicit.
