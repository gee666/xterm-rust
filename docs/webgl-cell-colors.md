# WebGL cell colors

`webgl::cell_colors::CellColorResolver` ports the packed-color algorithm in
`source_repo/addons/addon-webgl/src/CellColorResolver.ts`. It is a typed Rust
component, not a public WebglAddon or a completed WebglRenderer.

## Contract

The caller passes the existing buffer cell's `AttributeData`, its codepoint,
absolute buffer coordinates, device cell dimensions, font size/DPR, focus and
selection state, the existing theme color set, and decorations at that cell.
Selection must come from the terminal's shared selection model. Decorations use
the existing `RowDecoration` representation and preserve service iteration order.
The resolver visits bottom decorations, selection, then top decorations. It does
not own another terminal, selection model, theme service, or Unicode authority.

The result contains unsigned Rust words for JavaScript's signed packed `bg`,
`fg`, and `ext`. Bit patterns are unchanged. Selection retains non-default cell
backgrounds by blending at source alpha 0x80. Decorative glyph foregrounds use
their own corresponding blend. Only selected background overrides remove DIM.
Partial inverse overrides resolve the other color, including the different
meanings of default foreground and default background. Attribute flags otherwise
survive. Transparent black decorations are present colors, not missing values.

Dotted underline variants use font/DPR line width and the horizontal device
position. Pattern phases use the existing custom-glyph definition membership,
even when custom glyph drawing is disabled. The source's zero-offset fallback,
JavaScript remainder, truncating shift conversion, hyperlink underline override,
and nonfinite arithmetic are preserved.

Missing palette entries return the existing `AtlasColorError` with the requested
index. The result then retains the original base words, including unmodified
extended variant bits, just as the source's result does before its exception.
The next call resets all work state. Live theme color objects remain shared and
are read on each call.

This API accepts typed values with no JavaScript getters or decoration callbacks.
It does not establish raw getter ordering or module-global scratch reentry parity.
The browser integration must keep those host phases at its actual boundary.
DOM row colors cannot replace this algorithm: DOM emits CSS classes/styles,
whereas WebGL preserves packed flags and produces atlas cache keys. AtlasColors
is a later phase that resolves those keys to raster paint and contrast caches.
No new dependency or terminal engine delegation is used.

## Reproduction

Use the unchanged reference with its installed esbuild dependency. From the
worktree root, set `SOURCE_ROOT` to that reference directory, then run:

```sh
node scripts/webgl-cell-color-oracle.mjs
cargo test --locked -p xterm-web --test webgl_cell_colors --all-features -j2
cargo test --locked -p xterm-web --test webgl_cell_colors --release --all-features -j2
cargo clippy --locked -p xterm-web --lib --test webgl_cell_colors --all-features -- -D warnings
cargo clippy --locked -p xterm-web --lib --test webgl_cell_colors --all-features --target wasm32-unknown-unknown -- -D warnings
```

The oracle bundles unchanged source in memory and appends only exports for its
real `AttributeData` and color helpers. Typed leaf services supply colors,
selection and decorations. No source file or assertion is edited. The fixture
pins every imported source file, the generator and bundle. To check reproducible
output without replacing the fixture, set `CELL_COLOR_ORACLE_OUTPUT` to an owned
`O:/xterm-rust-swarm-16/tmp/...` file, regenerate and compare bytes.

The fixture has 2,956 cases: 2,304 directed color/layer combinations, 105 phase
combinations, 16 short-palette cases, one hyperlink case, 18 nonfinite cases, and
512 seeded cases at `0x7a12c431`. The Rust test reuses one resolver and runs each
case twice, comparing every packed word and error presence. A separate test
checks exact missing-palette index, partial result state, recovery and live theme
mutation. These are two Rust test functions, not 2,956 browser executions.

## Verification at task 441

Receipts are in `results/webgl-cell-colors/`. Windows Rust/Cargo 1.85.1 and Node
24.15.0 were used. Six focused native WebGL targets passed 60 tests with all
features. The default-feature run also passed 60. The focused release target
passed two. Native and WASM library/test warning-denied Clippy passed. WASM was
compiled, not executed in a browser.

The retained selected-DIM negative control changed only the DIM clearing
expression. The real differential test failed at `matrix-36`, with one test
passed and one failed. The fixed file was restored in `finally` and its exact
SHA-256 checked before fresh all-feature and release passes. The negative source,
failed log and restoration hashes are retained separately from passing receipts.

Exact `cargo fmt --all -- --check` failed locally with Windows OS206. The
xterm-web member format check passed. This is not a substitute full-workspace
pass. No full-workspace Clippy/test, native Linux/macOS run or browser matrix was
claimed for this component.

Task 306 still requires actual WebglRenderer model/orchestration/layers,
WebglAddon exposure, Root and RenderService switching/restoration, shared atlas
runtime installation, context loss/restoration and original full WebGL/demo
suites. Task 441 does not close those obligations. Overall equivalence remains
NOT PROVEN.
