# Renderer theme CSS

`crates/xterm-web/src/renderer/theme_css.rs` ports the text generation in `DomRenderer._injectCss` from the supplied source. It generates the complete source rule order, owner-scoped selectors, text styles, cursor animations, selection overlays, ANSI colors and inverted-default colors. Renderer task #32 owns actual style element creation, updates and disposal.

The helper uses the existing Rust Color helpers for source `multiplyOpacity` and `opaque` calculations. Direct `.css` values are retained rather than reconstructed from RGBA. Tests deliberately make the CSS spelling and packed RGBA disagree, because source direct rules use `.css` while opacity/opaque rules compute from `.rgba`.

Numeric font size and cursor width use the existing locked ryu-js 1.0.3 formatter. Rust's default float display is not ECMAScript formatting for every exponent, non-finite value and signed zero. This is a direct xterm-web dependency on an already used, portable formatting package, not a new rendering framework.

## Reproduction

```text
node scripts/check-theme-css.mjs
cargo test -p xterm-web --test theme_css
cargo clippy -p xterm-web --lib --test theme_css -- -D warnings
rustfmt --check crates/xterm-web/src/renderer/theme_css.rs crates/xterm-web/tests/theme_css.rs
```

The checker bundles the unchanged source DomRenderer with the reference's esbuild dependency. It calls its actual `_injectCss` method with a preexisting in-memory style object and the actual source terminal-selector getter. The test does not call a copied JavaScript CSS algorithm. DOM allocation, computed styles, pixels and renderer lifecycle are not claimed by this adapter.

The source check produced 73 cases with seed 3311141121. Cases include two terminal ids, the largest safe integer id, well-formed non-ASCII fonts, retained CSS spellings, contradictory css/RGBA values, transparent colors, all font/width numeric boundaries including NaN and infinities, and ANSI palette lengths 0, 1, 2, 16, 256 and 300. Every output string and source hash is retained in `tests/fixtures/theme-css.json` and `results/theme-css/oracle.json`. Normal checks compare the committed fixture; `--write-fixtures` regenerates it for deliberate source changes.

The native test compares complete strings and prints a bounded first-difference excerpt on failure. It does not compare only selected tokens. A second test checks scope, empty palette, inverted default and negative-zero formatting. The large JSON fixture is generated source oracle data, not an oversized handwritten module.

## Current evidence and gaps

The source fixture generation ran successfully on native Windows with Node v24.15.0 and esbuild 0.28.1 before the coordinated low-disk build pause. The latest Rust generator and tests were formatted but remain uncompiled and unrun while P: is below the shared reserve. No native pass is inferred from those source results. Independent review and the native commands above remain required.

`Font` stores Rust strings and the return type is String, so this helper represents well-formed Unicode. Lone UTF-16 surrogate font strings require the renderer's separately owned raw-UTF-16 companion. This limitation does not waive full browser compatibility. Task #32 and its companion must preserve exact code units and actual style text in the browser. Browser integration also must verify style node reuse/removal, multiple terminal isolation, option/theme changes and computed styling.

Detailed cases are in `docs/acceptance/theme-css.md`, criterion #397. This helper contributes to #196 and #197 but is not full renderer completion. Full terminal/browser equivalence is NOT PROVEN.
