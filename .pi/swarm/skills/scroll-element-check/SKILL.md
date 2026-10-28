---
name: scroll-element-check
description: Use when comparing the concrete Rust scrollable Element and mouse/wheel normalization with unchanged source in Chromium, Firefox and WebKit.
---

# Scrollable element checks

Run from the isolated worktree containing task167 and its concrete Root/Viewport dependency stack. Keep the private Cargo target on C, not the nearly-full primary volume. Shared dependency downloads are sufficient. Do not copy dirty primary source files into the worktree.

Requires Cargo/rustc1.85.1, wasm32-unknown-unknown, wasm-bindgen0.2.100 and installed Playwright/esbuild with Chromium, Firefox and WebKit. Set `PLAYWRIGHT_MODULE`, `ESBUILD_MODULE` and `WASM_BINDGEN` to existing installations if dependencies are not installed in this worktree's read-only source reference copy.

```sh
node scripts/scroll-element-source.mjs
node scripts/scroll-mouse-source.mjs
WASM_TEST=wasm_scroll_element node scripts/browser-host-check.mjs
cargo test -p xterm-web --lib --test scrollable --test scrollbar_visibility --test scrollbar_state_differential
cargo clippy -p xterm-web --lib --test wasm_scroll_element -- -D warnings
rustfmt --check --edition 2021 crates/xterm-web/tests/wasm_scroll_element.rs
```

Apply normal workspace fmt/Clippy/test gates too. Do not suppress unrelated failures to call this a workspace pass. Source runners record fresh status before execution, retain source hashes, and always close their browsers and HTTP server. Cargo/build and browser limits are enforced by browser-host-check. Set a timeout for the outer shell process as well.

The browser suite has11 functions per engine, including38 Element scenarios and13 wheel/eight mouse source-constructor cases. Source reports are fixtures compiled into the Rust suite, not live JavaScript engine delegation. Preserve failing reports under immutable run directories. Regenerating source reports must precede the Rust run when the source fixtures change.

On Cargo1.85, a new private target can have only `target/wasm32-unknown-unknown/CACHEDIR.TAG`. If artifact pinning rejects a missing parent tag, first verify that the compiler artifact is in that private target, then copy that Cargo-generated tag to `target/CACHEDIR.TAG` and record the action. Do not forge a tag for arbitrary artifacts.

Evidence and explicit gaps are in `docs/scroll-element-acceptance.md`. This suite does not establish main integration, raw JavaScript model-position fidelity, native Linux/macOS execution or whole-port equivalence.
