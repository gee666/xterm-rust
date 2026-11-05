# Live internal render dimensions

Task414 adds `_core._renderService.dimensions` over the actual Rust `RenderService` and its active renderer. It removes six failures in the unchanged original Kitty placement assertions. It is a read-only inspection binding, not the complete writable source-private service ABI.

The core render-service property is undefined before open. Browser service creation installs one stable view before renderer installation. Its dimensions getter checks the live renderer and caches only the JavaScript view identity for that renderer. Every enumerable scalar getter reads the original Rust renderer's `RenderDimensions`; there is no mirrored geometry, theme, font measurement, or terminal engine in JavaScript. Active nested views retain their identity across resize, font changes and reset. Replacing the renderer produces a different dimensions view. A retained old view continues to read its original renderer. Disposal still disposes the actual service/renderer and host jobs. Reading service.dimensions after disposal throws TypeError; previously retained dimensions remain readable.

The shape adapter is `crates/xterm-web/src/terminal/internal_render.js`. Its closure owns the Rust renderer while a retained dimensions view remains reachable. Thus a retained view can also retain the disposed renderer's Rust allocation and references until JavaScript collection. It does not defer explicit disposal. Full source object reachability/GC equivalence, raw writes/deletion/replacement, service methods, descriptors, cross-service aliases and manually reinstalling already-disposed backends remain under315/324. The diagnostic deliberately rejects mutation rather than accepting writes that cannot control Rust.

## Observed checks

Windows, Rust/Cargo1.85.1, wasm-bindgen0.2.100, Node24.15.0:

- Directed image coordinator tests pass26 in each of Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. Two new source/Rust tests compare before-open absence, all twelve dimensions, active/nested identities, resize, font updates, reset, actual renderer replacement, disposal and retained old values. Rejected diagnostic writes are separately asserted.
- Unchanged original Kitty `only [cr] specified computes` assertions discover6, execute6 and pass6, with0 failed/skipped/unrun. The tests now read real Rust private render dimensions.
- The first complete-family attempt discovers639 but executes452, passes444, fails8, skips3 and leaves184 unrun. Two Firefox beforeAll hooks time out at their original10-second bound. Remaining observed assertion failures are the missing `_core._inputHandler` and165-versus160 IIP geometry differences. This incomplete attempt is not a full-family pass and does not supersede the preceding complete639-case report.
- Focused WASM Clippy passes with `-D warnings`. Exact workspace `cargo fmt --all -- --check` fails with Windows OS206 at both long and short worktree paths and through a temporary drive alias. That alias was removed in finally. The five individual package formatting checks pass and cover the same workspace members; no full-command pass is claimed.

Concise raw reports, source identities, exact commands and logs are retained in `results/render-dimensions/`. Historical paths in the reports identify the build at the time. The new owned worktree moved from O:tmp/G414 to O:tmp/H via Git. Original assertions/reference files were not edited. An owned dependency junction was removed in finally and original P: dependencies were checked afterward. Full image-family parity, private writable ABI, img01..48 and complete native/browser/platform equivalence remain NOT PROVEN.

## Reproduce

Use the prerequisites and explicit dependency/tool environment paths from `docs/image-canvas.md`.

```sh
WASM_TEST=image_addon_browser CARGO_BUILD_JOBS=2 node scripts/image-canvas-check.mjs
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test image_addon_browser -- -D warnings
node scripts/e2e-upstream.mjs --suite addon-image --grep 'only [cr] specified computes' --timeout-ms 180000
node scripts/e2e-upstream.mjs --suite addon-image --no-build --timeout-ms 180000
```
