# Raw CSS task 170 handoff

Cornelia is handing task 170 to Octavia so task 32 can finish the actual RenderService and browser integration. This is implementation work, not a waiver of UTF-16 behavior. Criteria 193, 194, 196, 197 and 397 remain incomplete until the raw path is integrated and verified. Section R9 of `docs/acceptance-color-rendering.md` is the contract.

## Current files and status

- `crates/xterm-core/src/color/css.rs` adds `Utf16Color` and `to_color_utf16`. The existing string parser is unchanged. The new function uses a private classification-only view for unpaired surrogates, replacing them with one nonmatching UTF-16 unit. This preserves the fast grammar's ASCII/whitespace classes and UTF-16 length. Original units are returned whenever the source retains the original CSS. This view is never emitted to a browser. Add independent differential cases before accepting it.
- `theme/color_set.rs` adds raw code-unit storage to shared ColorRef objects. `css_units()` returns actual data. `set_css_units()` retains invalid units and uses an escaped native inspection view for `borrow().css`. Mutating packed RGBA preserves original CSS; changing the native CSS projection causes `css_units()` to use that new well-formed value. Explicit `set_css_units` is the lossless mutation API.
- `theme/css_text.rs` introduces `CssText(Vec<u16>)`, with units, strict conversion and escaped Display for inspection. It is already used for all Theme CSS properties. String literals using `.into()` still work. Display is not a browser conversion.
- `theme/mod.rs` stores `Utf16ColorParser`, exposes `new_utf16` and `parse_css_units`, and retains the old well-formed ColorParser constructor as a convenience path. `resolve.rs` parses units directly and preserves foreground CSS units when an opaque blend copies it.
- `canvas_css.rs` now exposes `to_color_utf16` and `utf16_theme_parser`, using the real owner-document canvas and raw JsString fillStyle assignment. The legacy to_color and theme_parser APIs remain. Root browser construction should use `ThemeService::new_utf16` with `CanvasColorParser::utf16_theme_parser`.
- `theme/live.rs` adds `set_theme_value`, preserving source field commits before later property getters, double selectionForeground reads, repeated extendedAnsi reads, cache clearing, restore snapshot and event order. `option.rs` retains actual OptionError/EmitError sources. Add directed tests for partial state and error identity. InvalidAnsiState is an explicit error when a native callback shortens the typed ANSI vector during getter traversal; do not mistake this for proof of JS sparse-array mutation behavior.
- `tests/theme_upstream.rs` has the one conversion needed for its reverse-palette Theme values. Existing expected values are unchanged.

These new raw-path changes were formatted but have NOT been compiled or run yet. Earlier native theme/cache tests, width tests and browser canvas tests passed before this raw-path patch. Do not describe those historical results as current raw-path evidence.

## Ownership

Octavia may edit the raw Theme/CssText/ColorRef/parser files above, add tests and oracle scripts, and add the raw CSS companion. Coordinate any change to core Color behavior with Cornelia; preserve all 350 original assertions and the retained counterexamples.

Font fields are still String. Keep that source frozen until task 138's 73-case native CSS baseline runs in a controlled build slot. Then Octavia may change Font family/weight/weight_bold to CssText and update `renderer/width_cache.rs`, its tests, `renderer/options.rs`, `renderer/canvas_width.rs` and `char_size_browser.rs`. Existing `.into()` fixtures should still work. Native font equality must compare actual code units, not a replacement-character projection.

Cornelia still owns `renderer/dom/**` and `renderer/service*.rs`. Provide a stable raw CSS companion API and coordinate its integration into DomRenderer rather than editing those files concurrently. Xavier owns the browser crate root and manifest, including public exports. Sargon owns shared RendererUtils and GPU code. Isaac owns BufferDecorationRenderer and ColorZoneStore. Sophie owns the canonical stylesheet delivery.

## Remaining implementation

1. Preserve raw font units in canvas font assignment, DOM font properties and cache invalidation. Do not use from_utf16_lossy or Display for host setters. Keep source font trimming and JS numeric formatting.
2. Reuse the canonical native theme_css helper without copying its hundreds of selectors. A possible companion encodes every input CSS/font code unit as a fixed marker plus four hex digits before calling the helper, deep-copying colors while retaining their RGBA. Decode the resulting stream to UTF-16. Since every user code unit is encoded, user marker/NUL text cannot collide with the wire syntax. The static template must not contain the marker; reject unexpected malformed wire data, and do not use unbounded marker-search retries. Test literal marker-like strings, NUL and lone high/low surrogates. An equivalent direct raw-fragment builder is acceptable if it still shares one canonical rule generator.
3. DomRenderer must assign the final stylesheet with raw JsString textContent. Cornelia will coordinate the small DOM integration change. Constructor, theme-event and option-event failures must propagate without holding RefCell guards across callbacks.
4. Verify actual canvas fallback with CSS comments containing lone surrogates, such as `rgb(/*<lone surrogate>*/255,0,0)`. Such input can miss the fast parser yet remain valid to the browser CSS parser. Retain the original units in ColorRef and generated CSS.
5. Add independent source-oracle comparisons for malformed UTF-16 prefixes admitted by the unanchored hex regex, RGB prefixes/suffixes, normalized short colors, preserved long colors, blending, font strings and full stylesheet output. Compare code-unit arrays and packed values, not lossy display strings.
6. Run all original Color assertions, theme/cache tests, the 73 full native CSS fixtures and raw companion cases, plus actual Chromium/Firefox/WebKit tests. Keep counts, seeds, logs, snapshots and counterexamples under results. No full equivalence claim while these checks or actual browser integration remain missing.

Use `scripts/build-slot.mjs` and `docs/build-slots.md` for scoped native/WASM commands. The shared target and profile policy remain in force; do not create another target directory or override release settings. Tiny source-only fixture checks under 20 MiB need no lease, but compilers, bundlers and browsers do.
