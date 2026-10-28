# Charulata handoff

Worktree: `C:/Users/user/.xterm-rust-target/7144d30a1fc3-worktrees/Charulata`, branch `swarm/Charulata`. Rebase and use the shared worktree merge script only after changed crates pass. No AGENTS or source_repo edits.

Completed and accepted: task 214, ordered JavaScript identifier reads and raw registration routes; task 254, real buffer line snapshot/toJSON. Canonical DOM-free WASM bodies are in `crates/xterm-wasm-support/src/public_api`; old web paths reexport them. Original actual Terminal DOM assertion stays in `crates/xterm-web/tests/public_adapters_wasm.rs`.

Observed: source identifier oracle 195 cases; all 16 component tests pass in Chromium, Firefox and WebKit. Native snapshot test and focused native/WASM Clippy pass. Earlier native adapter suite passed 27 cases. Use `WASM_PACKAGE=xterm-wasm-support WASM_TEST=public_adapters_wasm node scripts/browser-host-check.mjs`; set WASM_BINDGEN to the existing 0.2.100 tool if absent from PATH.

Remaining task 66 waits for actual Root task 31: unchanged Parser.test.ts and Terminal.test.ts loadAddon assertions through the facade, real GC ownership checks, then swarm_review. `scripts/public-parser-browser.mjs` is the prepared original-assertion runner. Do not call component results full Root parity.

Latest restart update: Salahuddin reports actual Root numeric seven tests now pass in all three browsers on `swarm/Salahuddin` commit `09e9ab3`, after merging Xavier and the canonical adapters. This is a useful tested candidate for task 66's remaining Root suite, not a full Root acceptance. See post 184.

The branch is not merged. Last changed-crate run found main failures: missing web exports for character_joiner/scroll/webgl, web test URL u32 versus u64, and core_services_integration.rs188 logger cursor expectation. Chloe, Xavier and Willie were notified. Do not weaken assertions or copy foreign WIP to hide these. There are untracked generated results in the private worktree; stage only intended files.

GitHub Actions was triggered by pushes. Observed run URLs include https://github.com/gee666/xterm-rust/actions/runs/40659680361 and the earlier failed 40659197212. Refresh actual conclusions before citing a platform pass.
