# Renderer theme CSS acceptance

The pure generator ports the text construction inside `source_repo/src/browser/renderer/dom/DomRenderer.ts` `_injectCss`, lines 177-313. The renderer owner still creates/reuses/removes the style element and calls this function after options/themes change. These browser lifecycle checks are not replaced by string-only assertions.

- Preserve the complete source CSS text in source order, including spaces, selector scoping, the font family's raw spelling, font weight strings and numeric font-size/cursor-width serialization.
- Scope every rule with `.xterm-dom-renderer-owner-{id}`. Blink keyframe names independently include the same id, so different terminals cannot animate each other's cursor.
- Preserve pointer-events suppression, foreground and dim alpha, normal/bold/italic/blink-hidden text classes, three cursor animations, focus-only animation selectors and the idle `!important` override.
- Preserve block-cursor normal and nonblinking important overrides, outline offset, bar inset shadow and underline border/height.
- Preserve focused and inactive selection overlays and exact theme color CSS strings. Do not canonicalize an explicitly retained source color spelling except in source multiplyOpacity/opaque calls.
- Generate FG, dim FG and BG rules for every actual ANSI entry, not only the default 256. Include empty and shortened/extended palettes. Generate inverted-default class257 rules using opaque background and its half opacity, and direct foreground background color.
- Test independent `.css` and `.rgba` values in theme objects. Source direct CSS uses css; dim and opaque calculations use rgba. No hidden assumption that both representations agree is allowed.
- Test JS number boundaries, including -0, fractions, 1e-7, 1e21, NaN and infinities. Native u64 ids model valid integral terminal ids; arbitrary JS number ids are not claimed represented.
- Compare full strings against the actual unchanged `_injectCss` method with a test style element, not against a separately handwritten JS rendering model. Retain source and output hashes, case definitions and raw fixtures.
- The requested Font/String interface covers well-formed Unicode. It cannot represent lone UTF-16 surrogate font strings or CSS output; document this boundary and leave full browser binding conversion under renderer task32, without silently replacing those code units.
- Run native tests, source oracle checks, rustfmt/Clippy and independent review. Verify final real DOM style insertion/update/disposal and computed styles on the Rust-backed browser terminal under task32. Full rendering equivalence is not established by this helper.
