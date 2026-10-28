# Shared browser assets

The canonical authored terminal stylesheet is `crates/xterm-web/assets/xterm.css`, copied with its complete attribution from `source_repo/css/xterm.css`. Sophie owns its delivery under task 45. The build copies this asset to the distribution path `browser/xterm.css`. The demo loads `../xterm.css`; it does not own a separate authored terminal stylesheet.

Browser component tests, demos, Rust/WASM bindings and addon tests must use this same asset. From a Rust file directly in `crates/xterm-web/src` or `crates/xterm-web/tests`, `include_str!("../assets/xterm.css")` references it. A JavaScript harness may serve the authored asset or its identical distribution copy. Do not create another authored copy under `web/assets` or a test-only stylesheet to make layout pass.

Xavier owns the browser crate's public exports and can expose an `XTERM_CSS` constant for consumers. The supported web distribution must include the stylesheet as a mandatory asset even when a binding also embeds it for component tests. Source hashes and browser computed-style/pixel evidence are governed by criterion 412 and `docs/acceptance-color-rendering.md` section R10.

This asset contract does not establish renderer, selection, accessibility or addon equivalence. Those checks still run through the Rust-backed browser terminal.
