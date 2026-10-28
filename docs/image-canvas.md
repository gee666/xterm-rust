# Browser image canvas component

Task272 implements the canvas and resource operations from `source_repo/addons/addon-image/src/ImageRenderer.ts`. It is not the complete image addon. Task127 retains terminal open, font option, renderer replacement and disposal subscription wiring. Task38 retains protocol/parser activation and exported addon integration. Full equivalence is NOT PROVEN.

## Implementation and ownership

`crates/xterm-web/src/image/` is WASM-only. It has no new dependency and no unsafe code. Viesturs's uncompiled primitive helpers were reused and corrected, rather than discarded.

- `primitives.rs` creates actual browser canvases, ImageData and ImageBitmap promises. It preserves signed32 canvas argument conversion and the missing-global API fallbacks.
- `layers.rs` owns top and bottom DOM layers. It uses isolation without replacing the integrator's z-index. Failed context creation removes a newly inserted canvas. Clearing, resizing and removal remain explicit, fallible operations.
- `drawing.rs` updates each real storage `ImageSpec`, scales from the original resource, restores original identity when cell metrics return, clips edge tiles, and applies the source's distinct draw/extract rounding.
- `placeholder.rs` creates the screen-aligned checkerboard and optional bitmap. The small JS continuation only assigns the resolved bitmap, keeping an unsettled callback in the garbage-collected JS heap. It deliberately preserves the source's late bitmap assignment after placeholder removal. It does not invent cancellation or claim a pending bitmap was closed.
- `resource.rs` wraps real canvas/bitmap surfaces and exposes the original bitmap's close operation to storage. Native decoders call `upload_pixels` once before adding the canvas resource to browser storage. Passing an unuploaded native pixel surface to drawing returns an error instead of copying it on every tile draw.
- `storage.rs` implements every `StorageRenderer` operation over these actual browser operations. It preserves thrown JavaScript values through `StorageError`.

`ImageCanvasHost` supplies the current document, screen element, CSS cell/canvas metrics and screen width. Metrics are typed binary64 snapshots, not a JS property object. A host must release terminal state borrows before returning. No terminal subscriptions or strong terminal reference are hidden inside this component. Its lifecycle owner must call `rescale_canvas`, remove layers on renderer replacement, call `dispose`, and request row refreshes after font/placeholder changes. Those operations are pending task127, not simulated here. Resource and context calls hold no Rust state borrow across host code. Layer resizing traverses a live JS Map iterator.

## Run

Requires Rust1.85.1 or the repository-supported toolchain, the wasm32-unknown-unknown target, wasm-bindgen CLI0.2.100, and the reference's installed esbuild, sixel and Playwright dependencies/browser binaries.

```sh
node scripts/image-canvas-check.mjs
```

The default compiles the real xterm-web test target. It fails while the shared browser crate has unresolved producers. Do not describe the isolated command below as a workspace pass.

```sh
node scripts/image-canvas-check.mjs --isolated-component
CARGO_TARGET_DIR="$PWD/target" cargo clippy --locked \
  --manifest-path tmp/image-canvas/component/Cargo.toml \
  --target wasm32-unknown-unknown --all-targets -- -D warnings
rustfmt --edition 2021 --check crates/xterm-web/src/image/*.rs \
  crates/xterm-web/tests/image_canvas_browser.rs \
  crates/xterm-web/tests/image_canvas_support/*.rs
```

The explicit isolated mode creates a temporary standalone manifest under `tmp/image-canvas/`. Its library and tests point directly at the production Rust files. It uses the real core/addon/storage crates, the repository lockfile as resolution input, the same small dev/test settings, and the worktree's existing Cargo target. It neither edits nor substitutes the failing browser library. The generated manifest is temporary, not an additional shipped package.

Set `SOURCE_REPO` to another exact reference tree, `REFERENCE_NODE_MODULES` to its matching installed dependency directory, `WASM_BINDGEN` to the matching CLI, and `PLAYWRIGHT_MODULE` to the matching Playwright `index.mjs` when they are not in the documented default locations. `BROWSERS=chromium` is diagnostic only; it is not a three-browser result.

## Observed verification

Windows run with rustc/cargo1.85.1 and Node24.15.0:

- 14 Rust/WASM tests passed in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. No ignored tests.
- Seed0x272389 performs128 draw/extract pairs per browser. Each draw compares every pixel in the80x60 output canvas with the unchanged source class, then compares extracted tile dimensions and all nonempty tile pixels. Cases vary fractional cell size, rescaling, edge clipping, counts and positions. This is384 pairs across three browsers, not an exhaustive input proof.
- Other checks exercise real native engine/storage tiles, original resource identity, ImageData bytes/local document creation, top/bottom ordering, region clearing, no-op versus resetting canvas size, real bitmap conversion/close, missing APIs, missing2D contexts, unchanged spec state after failed rescaling, thrown-value identity, storage deletion partial state, and late placeholder bitmap close ordering.
- Scoped WASM Clippy passes with `-D warnings`. Direct rustfmt checks pass.

Raw browser commands, counts and logs are in `results/browser-host/image_canvas_browser/` and its latest JSON report. `results/image-canvas/verification.json` links exact Rust/source hashes and marks the isolated scope. The source bundle is generated in `tmp/` without editing source code or assertions. The original renderer is only the comparator; the Rust side draws through Rust and the browser canvas API.

Independent review found that bitmap retrieval must still return a new canvas when its2D context is absent. This was fixed without weakening existing tests. A new real-bitmap regression goes through shared storage retrieval and checks successful canvas dimensions, transparency and global document ownership. Exceptions from context creation/drawing still propagate.

Required workspace commands were attempted and failed. Logs are retained under `results/image-canvas/`:

- `cargo fmt --all -- --check`: Windows error206, command line too long.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: existing `event_context.rs` type-complexity lint.
- `cargo test --workspace --all-targets --all-features`: unresolved `xterm_web::scroll` and `search_host` imports among the existing browser integration tests.
- `cargo check --locked -p xterm-web --lib --target wasm32-unknown-unknown`: missing browser root modules/APIs such as terminal, js_error, composition callbacks and task queue producers.

The gated merge was attempted after review and rebased the accepted code to `09db704` over main `1e62bc5`. It stopped at the native `search_addon` test's missing `xterm_web::search_host` import. `results/image-canvas/merge-gate.log` retains the failure. Task277 owns the eventual gated merge of this branch and accepted QOI branch `swarm/Marie`; do not bypass the checks.

These failures are integration work, not external blockers and not waived by the component pass. Original full ImageAddon/Kitty browser suites, live protocol ingestion, terminal lifecycle subscriptions, native Linux/macOS execution and full addon equivalence remain unproven.
