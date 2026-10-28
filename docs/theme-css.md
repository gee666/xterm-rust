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

The source fixture generation ran successfully on native Windows with Node v24.15.0 and esbuild 0.28.1. After the approved shared-target relocation, a controlled native lease ran both Rust tests successfully, including all 73 exact full-string cases against the new shared UTF-16 rule generator. The receipt is `results/theme-css/slots/native-1793171612.log`, with its linked managed command log. Earlier failed compile and disk-preflight attempts remain separate. Independent review and warning-free scoped Clippy remain required; this is not browser rendering proof.

The native `Font` convenience model stores Rust strings/f64 and the return type is String, so this path represents well-formed Unicode. Its same canonical rule generator also supports a lazy live reader under task #170, reading raw OptionValue strings and String-hint coercions at source interpolation positions. Lone UTF-16 surrogate font strings require the renderer's separately owned raw-UTF-16 companion. This limitation does not waive full browser compatibility. Task #32 and its companion must preserve exact code units and actual style text in the browser. Browser integration also must verify style node reuse/removal, multiple terminal isolation, option/theme changes and computed styling.

Detailed cases are in `docs/acceptance/theme-css.md`, criterion #397. This helper contributes to #196 and #197 but is not full renderer completion. Full terminal/browser equivalence is NOT PROVEN.
