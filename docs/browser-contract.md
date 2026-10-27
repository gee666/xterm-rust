# Browser integration contract

Xavier owns `crates/xterm-web/Cargo.toml`, `src/lib.rs`, `src/dom.rs`, `src/api.rs`, WASM exports and public terminal integration under task 31. Cornelia owns `src/renderer/` and rendering modules under task 32. Other browser task owners must send export/dependency requests before editing the crate manifest or root module.

## Dependencies and targets

The core remains platform independent. The web crate uses wasm-bindgen for the browser ABI, js-sys for JavaScript callbacks/errors and web-sys for DOM operations. These packages are MIT OR Apache-2.0 and are maintained together. The browser needs these dependencies to create real DOM nodes and register browser event callbacks without executing the upstream terminal engine. Versions are locked in Cargo.lock. DOM implementation is compiled on wasm32; API validation and render contracts also compile and test natively. No successful placeholder Terminal.write or no-op API is permitted.

## DOM ownership

`dom::BrowserDom::new(parent: &web_sys::HtmlElement, options: &DomOptions) -> Result<BrowserDom, JsValue>` creates the upstream structure in the parent's ownerDocument or explicit document override:

- `element`: div.terminal.xterm, dir=ltr, optional allow-transparency class
- `viewport`: div.xterm-viewport under element
- `screen`: div.xterm-screen under element
- `helpers`: div.xterm-helpers under screen
- `textarea`: textarea.xterm-helper-textarea under helpers
- `composition`: div.composition-view under helpers

All six nodes are public fields for browser components. Renderer owns its rows container under screen, not the helpers. DOM listeners belong to the registering component and must be removed on disposal. BrowserDom removes its root on disposal and on Drop. Dispose is idempotent. BrowserDom never owns core terminal state. Repeated public open must not recreate DOM or move an already-open root.

`DomOptions` specifies `allow_transparency`, `disable_stdin`, `is_chrome_os`, `prompt_label`, and optional `document_override`. DOM defaults match upstream. Selection/accessibility owner owns localized string updates; prompt label is provided at DOM construction as upstream reads its current global value only then.

## Rendering and input

Rendering reads Rust core buffer/cell snapshots, never the source_repo engine. `screen` is the renderer attachment point. Renderer reports its dimensions through native-testable types in `api`; callbacks are registered only after the renderer has valid dimensions. Each task owner should keep component APIs in its module and request root exports.

Input binds DOM handlers to textarea/root with explicit RAII listener ownership. Input data must feed the Rust core data event path, not be written to the terminal display as an echo shortcut. Parser writes, generated replies, user input and render notifications remain distinct operations.

## Public API boundaries

API validation preserves upstream JavaScript Number behavior before conversion to Rust indices. `verify_integers` rejects NaN, infinities and fractional numbers. `verify_positive_integers` deliberately accepts NaN as upstream's truthiness guard does, and accepts zero and negative zero. Constructor-only option writes reject cols/rows. Proposed APIs use the upstream allowProposedApi error.

A complete exported Terminal is pending the task 28 Rust engine API and integration with tasks 32-35. BrowserDom is a host component, not a full xterm-compatible terminal. Missing terminal methods must not appear as successful no-ops. Overall browser equivalence is NOT PROVEN.
