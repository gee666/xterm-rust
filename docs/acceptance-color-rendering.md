# Color and browser rendering acceptance

These are required outcomes, not a list of implemented behavior. Task 4 owns the color library and task 32 owns browser rendering. Every item needs executable evidence and an independent source review. A native helper test does not satisfy a DOM, font, pixel, animation, or browser-integration requirement. Run applicable browser cases on Chromium, Firefox and WebKit. Preserve upstream assertions, including awkward behavior, unless the user explicitly approves a difference.

## C1. CSS parsing and packed channels

Sources are `common/Color.ts` and `common/Color.test.ts`.

- Convert all three and four-channel inputs to CSS and big-endian RGBA. Optional alpha defaults to 255; explicit zero stays zero. Keep at least two hexadecimal digits, but do not truncate larger or negative values. Numeric packed operations use JavaScript 32-bit wrapping, not saturation.
- Parse `#rgb`, `#rgba`, `#rrggbb`, and `#rrggbbaa`; expand short forms, retain long-form spelling/case, and match all 350 original assertions without omissions. Verify mixed case and alpha endpoints.
- Match upstream's unanchored hash regex and total UTF-16 length dispatch. Invalid-looking strings can be accepted. For long hex, `parseInt` trims ECMAScript whitespace, accepts signs and optional `0x`, consumes a valid prefix and coerces NaN to zero at the bitwise boundary. Required independent regressions are `x#abc`, `a-1#abc`, `a+1#abc`, `a f#abc`, `a 1#abcde`, and `a0x1#abcd`.
- Preserve the exact lowercase `rgb`/`rgba` matching grammar, arbitrary matching prefix/suffix, one-to-three digit channels, comma whitespace including ECMAScript Unicode whitespace, optional alpha under either function name, fractional alpha spelling, channel overflow and invalid component counts. Reject unsupported formats in headless mode with the source error message. Check alpha 0, .5, 1, 1.5, 9.99, leading decimal point, malformed trailing point, four-digit channels and unsupported integer alpha 2.
- Preserve `transparent` spelling and packed zero. Do not treat uppercase `TRANSPARENT` as the special case.
- Browser fallback validates canvas fillStyle assignment using a non-string sentinel, draws and samples one pixel, retains the original CSS spelling, and rejects nonopaque fallback pixels. Test named colors, hsl and browser-supported CSS colors, invalid assignment, unavailable context, alpha loss and valid fast-path bypass of the callback. The real canvas adapter must execute in Rust/WASM, not an xterm.js engine.

## C2. Blend, luminance and contrast

- Unpack every RGBA byte in correct order. Foreground alpha 255 returns its original value/CSS; alpha zero returns the background RGB with opaque alpha, regardless of background alpha. Check intermediate alphas and both positive and negative channel deltas with JavaScript Math.round tie behavior.
- Apply the WCAG piecewise transfer threshold 0.03928, denominator 12.92, exponent 2.4 and weights 0.2126/0.7152/0.0722. Check black, white, all upstream grayscale values and each separate channel. Ratios must be symmetric and match black/black 1, black/mid-luminance 11 and black/white 21.
- Return no adjustment when the existing contrast meets the ratio, including equality and NaN ratio. Otherwise choose the source's initial direction based on foreground/background luminance, moving ten percent towards an endpoint with ceiling per channel. Check termination at black/white, ratios below one and impossible ratios above 21, reversed direction fallback and equal-result tie choice.
- Compare packed adjusted colors exactly, not only whether a target ratio was reached. Preserve all black-background and white-background expected values in the original suite, plus mixed colored backgrounds and alpha-bearing inputs. Plain and dim contrast caches remain separate.

## C3. Color object operations and numeric boundaries

- `opaque` replaces only alpha and formats six-digit CSS. `opacity` rounds alpha from the supplied JavaScript number, does not clamp, bitwise-wraps packed output, and preserves full hexadecimal spelling. `multiplyOpacity` derives the initial alpha from the packed color before multiplying. Verify .25/.5/.75, negative, >1, NaN, both infinities, signed zero, values beyond i64 and integer precision boundaries.
- `isOpaque`, RGB projection and null-color construction match upstream. Object blend retains foreground CSS for full opacity. Color ensure-contrast converts returned RGB to six-digit CSS and returns absence only for an already satisfactory ratio.
- Differential evidence records seed, operation count, exact expected/actual comparison and retained counterexamples. Generate random RGBA pairs and contrast/opacity factors. Do not use a green deterministic suite as a substitute for the required edge cases.

## R1. Palette, cache and color-object identity

Sources are `browser/Types.ts`, `ColorContrastCache.ts` and their applicable tests.

- Generate the exact Tango first 16 colors, all 216 cube colors with levels 0/95/135/175/215/255 and all 24 grayscale colors starting at 8 with step 10. Confirm all 256 values and CSS strings; retain all 240 explicit ThemeService palette expectations.
- Cache by ordered background/foreground pair. Distinguish missing, explicit null and stored value independently for CSS and color caches. Replacing one key does not change another; clear removes both caches. Cached object values and theme restore slots retain upstream shared-object identity.

## R2. Theme application, selection and restoration

Source is `browser/services/ThemeService.ts` and its complete test suite.

- Initialize every color field, exact default selection CSS/alpha, two contrast caches, 256 ANSI entries and absent selection foreground. A partial replacement theme reverts unspecified values to defaults, not previous theme values. Parse failures use the specified default. Browser-only CSS values must use the real canvas adapter.
- Blend cursor and cursor accent over the current background. Compute selection opaque colors from the original parsed selection before applying the source's 0.3 opacity adjustment to opaque transparent-selection fields. Inactive selection falls back to active selection. Empty or invalid selection foreground is absent; valid transparent black remains present.
- Default scrollbar alphas are .2/.4/.5 of the current foreground; explicit slider colors override them. Overview ruler border defaults to white. Parse all 16 named ANSI overrides and at most 240 extended overrides. Empty, shorter, longer, malformed and absent extended arrays preserve source fallback/reset behavior.
- A theme change clears both contrast caches, captures restore values and fires one color-change event after state is committed. Minimum-contrast option change clears only the full-contrast cache, as the source does. Test event ordering and reentrant listeners without holding state borrows across callbacks.
- Restoring no slot restores ANSI only. Restoring foreground/background/cursor or an ANSI slot uses the theme restore snapshot and still emits an event even when unchanged. `modifyColors` receives the live color set and fires after its callback. Theme replacement updates restore snapshots; temporary modifications do not. Verify object mutation versus property replacement and disposal of option/event subscriptions.

## R3. Text width and character-size measurement

Sources are `renderer/dom/WidthCache.ts` and `services/CharSizeService.ts`.

- WidthCache constructs four independently configured regular/bold/italic/bold-italic measurement canvases. Prefer OffscreenCanvas when supported and otherwise a DOM canvas. Fail on missing context rather than claiming a measured width.
- Regular single UTF-16 units below 256 use the Float32 cache, including its rounding on subsequent retrieval. Other strings/variants use the source's string-key cache, including collisions from appended B/I markers. Cache only positive widths; zero/negative widths are measured again. Empty strings, combining sequences, non-BMP strings, bold/italic combinations and cache clear/dispose are covered.
- Font configuration equality avoids needless invalidation. Any font/family/size/weight/weightBold change configures all variants and clears values. Include font loading updates and device-pixel-ratio changes through actual browser measurement.
- Character-size measurement first requires OffscreenCanvas width plus font bounding-box ascent/descent, falling back only when those APIs are unavailable. The DOM fallback measures 32 W characters, uses offsetWidth/32 and offsetHeight, sets aria-hidden, whitespace and font-kerning styles, and follows current font options. Nonpositive/undefined measurement retains the last valid size. Emit only when width or height actually changes. Dispose measurement nodes and subscriptions.

## R4. Joiners and UTF-16-to-cell ranges

Source is `services/CharacterJoinerService.ts`.

- Registration IDs increase, deregistration returns true exactly once, and an empty registry/missing/empty row returns no ranges. Split runs on fg/bg changes, skip width-zero cells, retain empty-cell spaces, trim only according to BufferLine and invoke handlers only for runs spanning more than one column.
- Handler ranges use UTF-16 indices, not UTF-8 bytes or Unicode scalar counts. Map them across supplementary and combined strings, wide cells, adjacent ranges, range starts inside multi-unit strings, and trailing ranges ending at the trimmed line. Handler failure is reported while subsequent joiners still execute, matching source ordering.
- Exercise every one of the nine source range-merge branches: preceding, contained, front overlap, end overlap, gaps after a merge, overlap into next, cover several, extend beyond last and append after last. Adjacent ranges remain distinct where source comparisons require it.
- JoinedCellData copies fg/bg, returns combined marker, supplied text/width, content zero and fake codepoint 0x1fffff. Its char-data getter preserves those values; unsupported setter returns the source error rather than a successful no-op.

## R5. DOM row content, merging and glyph attributes

Sources are `renderer/dom/DomRendererRowFactory.ts`, its complete tests, and shared renderer utilities.

- Trim only background-free trailing cells, extend through cursor, skip width-zero placeholders, use spaces for empty/invisible cells and NBSP for underline/overline spaces. Joined ranges render as one span only when all selection states agree and the cursor is outside. Splitting a rejected range must not accidentally rejoin later cells.
- Merge spans only when all source predicates agree: selection-aware fg/bg equality, extended attributes, link hover, spacing, no cursor/join/decoration and valid join boundary. Check each predicate independently and together. Preserve final merged text and width-based letter spacing.
- Match CSS classes for bold/dim/italic, all underline styles/colors, overline, strikethrough and blink-hidden. Apply hover underline last. Preserve bold-bright palette promotion and default/inverse/RGB/palette color modes.
- Resolve decorations after inverse; top decorations win over bottom and selection. Selection applies focused/inactive opaque background and optional foreground unless a top decoration overrides it. Minimum contrast honors overrides, skips background glyphs and ratio one, halves the target for dim cells, distinguishes cached null from missing and emits the exact adjusted CSS.
- Cursor requires initialized/visible state, correct row/column and focused active style or configured inactive style. Cover block/bar/underline/outline, disabled inactive style, blink settings and cells beyond trimmed text. Assert DOM attributes/styles/text and browser pixels, not merely generated strings.

## R6. Renderer geometry, selection, links and blink lifecycle

Source is `renderer/dom/DomRenderer.ts`, shared render models and browser styles.

- Create and dispose unique terminal-scoped CSS and row/selection containers. Add/remove rows on resize and clear removed blinking-row accounting. Calculate CSS/device character/cell/canvas dimensions using the source rounding and spacing rules for integer/fractional DPR, font metrics, line height and letter spacing. Set default spacing from W measurement and invalidate width cache on DPR/character-size/font changes.
- Selection overlays cover first/middle/final rows or reversed-column rectangles, clip to viewport and canvas width, preserve scrollback offsets and redraw the union of old/new bands plus cursor as the source requires. Removing selection clears overlays with source callback ordering.
- Render live buffer rows at ydisp, clear missing lines, use ybase+y cursor row and clamp cursor X. DEC cursor mode overrides options. Maintain per-row blink flags and aggregate count, including clear, resize, missing rows and link-only redraws.
- Link hover/leave clamps offscreen and resize-racing coordinates, redraws the correct inclusive row/exclusive column range and restores original decoration afterward. Focus/blur and visibility control cursor/text blink; cursor movement restarts the source idle timeout. Disposal cancels all timers and removes all created DOM/CSS resources.

## R7. Render scheduling and synchronized output

Source is `services/RenderService.ts` plus render debouncer integration.

- Coalesce asynchronous row refreshes and support immediate synchronous renders. Clamp queued row ranges after intervening resize. Distinguish redraw-only from content changes: render always emits, rendered-viewport-change emits only when not redraw-only. Selection refresh occurs after row rendering and before events.
- Hidden terminals pause rendering, remember full refresh and the latest deferred resize, measure size on becoming visible if invalid, flush deferred resize, then refresh. Handle IntersectionObserver isIntersecting and intersectionRatio fallback, owner-window replacement and observer disconnect/disposal.
- DEC 2026 buffers the union of requested rows both when requests arrive and when previously queued renders execute. A single 1000ms safety timer disables synchronized output and requests full refresh. Explicit flush cancels timer and merges buffered/current ranges. Disposal cancels without spurious rendering. Test exact boundary timing with controlled timers, not sleeps.
- Replacing a renderer disposes its predecessor, wires redraw callbacks, marks selection dirty and fully refreshes. Options, buffers, decorations, themes, font metrics, DPR and cursor changes trigger exactly the source's clear/resize/full-row/cursor-row actions. Clear, texture-atlas clear and focus/blur/cursor/selection forwarding must reach the active renderer and tolerate absent renderers as the source does.

## R8. Decoration and overview rendering

Sources include `browser/decorations/ColorZoneStore.ts`, `BufferDecorationRenderer.ts`, `OverviewRulerRenderer.ts` and original tests.

- Ignore decorations without overview settings. Merge only identical color/position zones at intersecting or padded-adjacent lines; test all full/left/center/right and default-position padding, insertion order, clear/reuse and pool trimming. Do not silently merge two existing zones beyond source behavior.
- Render decoration anchors, x/width/height/layer and top/bottom positioning against live viewport/scrollback; update on marker movement, resize, registration/removal and disposal. Deliver onRender and onDispose in source order with actual HTMLElement ownership.
- Overview canvas dimensions, border, lanes, DPR scaling, minimum zone height, clear/redraw scheduling and colors match the source. Exercise multiple overlapping zones, scrolling, buffer activation, marker disposal and option/theme changes with pixel assertions in the Rust-backed browser.

## R9. Raw UTF-16 CSS boundary

Task 170 must retain original JavaScript UTF-16 units for font family/weight, theme CSS and generated stylesheet text. Lone high/low surrogates and split surrogate sequences must not pass through `from_utf16_lossy` or wasm-bindgen String conversion. Native well-formed String helpers are only one path, not a waiver for the browser path. Compare complete CSS code-unit arrays with the source, including malformed but fast-path accepted colors, numeric CSS formatting, object getter failures and callback reentrancy. Test actual canvas/style setters and retained stylesheet textContent in all three browsers. Live theme getter failures must retain the source's already-committed partial color state, not convert the operation into a transaction.

## R10. Shipped terminal stylesheet

Preserve every rule in `source_repo/css/xterm.css` and its complete MIT attribution. The Rust-backed distribution must load that stylesheet or prove equivalent rules. Check root focus/outline/user-select, hidden textarea placement and dimensions, IME active layering, opaque-versus-transparent viewport background, relative screen/canvas layout, mouse/link/column-selection cursors, accessibility overlays/tree/live-region clipping, dim opacity, every underline/overline combination, strikethrough, decoration layer order and overview ruler placement. Verify scrollable arrow masks, visibility/fade timing, pointer-event suppression and each scrollbar shadow. Retain an asset hash comparison and real-browser computed-style/pixel assertions, including focus, transparency, composition, scrolling and disposal. A demo-only CSS file that is omitted from the supported web distribution is not sufficient.

## Evidence gate

For each section record discovered/mapped/executed/pass/fail/skip/unrun assertions, exact commands and environment, source hashes, random seeds/iterations, retained failures and independent findings. Existing Color evidence is in `results/color`; all remaining rendering evidence is required, not implied. Full rendering equivalence remains NOT PROVEN until every path above and the complete applicable upstream suites execute through Rust.
