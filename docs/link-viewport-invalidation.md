# Stationary link hover after viewport changes

The root never called `BrowserLinkifier::on_render`. Its component tests passed because they invoked that method directly. A hovered OSC8 URI therefore survived replacement of the underlying Rust cells until the mouse moved.

`terminal/link_services.rs` now subscribes the live linkifier to the shared RenderService viewport event. The callback holds a weak reference, propagates errors, and the terminal owns its disposal handle. Redraw-only frames do not cause invalidation. The callback leaves unrelated rows alone and requeries affected links using the last mouse event. Render bounds stay `f64` through the Rust-only component API so fractional values and NaN retain the source comparisons instead of being saturated into integer rows.

The regression opens the exported Rust Terminal, writes an OSC8 link, moves an actual DOM mouse event into it, renders an unrelated row, rewrites the URI without moving the mouse, verifies the replacement URI and actual DOM underline, then erases the link and checks pointer/hover removal. It also disposes the terminal. The earlier click/range-identity/disposal test retains every assertion and shares only its render-wait helper.

## Observed checks

Windows, Rust/Cargo 1.85.1, Node 24.15.0, wasm-bindgen 0.2.100. Build jobs were limited to two. Temporary builds, runner artifacts and browser profiles were under the owned O: worktree.

- Before adding the subscription, Chromium ran two Terminal tests. The existing test passed; the new regression failed with URI `first` instead of `second`. `results/links/viewport-invalidation/baseline.json` retains the exact failure.
- With the fix, `wasm_terminal_links` passed 2/2, `wasm_linkifier` passed 6/6, and `wasm_utility_links` passed 3/3 in each Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4. Each suite ran separately, not as one workspace matrix.
- Seven unchanged original OscLinkProvider/Linkifier tests passed through `scripts/link-oracle.mjs`. Its seed `0x01a8c055` regenerated the unchanged 4,097-case fixture. Source hashes and exact names remain in `results/links/oracle.json`.
- Seven native test targets passed 30 tests, including the 4,097-case OSC corpus, reply/cache differential and numeric viewport boundary checks.
- Scoped WASM library and all three changed browser test targets passed warning-denied Clippy.

Raw reports and command output are in `results/links/viewport-invalidation/`. The browser reports preserve compiler/WASM/binding hashes. Their O: artifact paths describe the run, not a promise to retain temporary WASM binaries. Concise reports and regression source are durable; disposable build outputs are removed during cleanup.

## Commands

Use the existing locked wasm-bindgen 0.2.100 executable and Playwright installation through `WASM_BINDGEN` and `PLAYWRIGHT_MODULE` if they are not local. `NODE_PATH` can point at the read-only reference dependencies for the source oracle without creating a junction.

```sh
node scripts/link-oracle.mjs
cargo test --locked -j2 -p xterm-web --test link_state --test links --test osc_links_differential --test osc_links_live --test osc_links_policy --test link_replies_differential --test link_replies_live
WASM_TEST=wasm_terminal_links node scripts/browser-host-check.mjs
WASM_TEST=wasm_linkifier node scripts/browser-host-check.mjs
WASM_TEST=wasm_utility_links node scripts/browser-host-check.mjs
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_terminal_links --test wasm_linkifier --test wasm_utility_links -- -D warnings
```

This repair does not establish full link or rendering equivalence. Mutable JavaScript link objects, provider/cache reentry, complete selection integration, full original browser suites and native Linux/macOS gates remain governed by `docs/link-acceptance.md` and the family closure tasks.
