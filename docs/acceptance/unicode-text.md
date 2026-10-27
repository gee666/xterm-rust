# Text decoding and Unicode acceptance contracts

These contracts refine the overall port requirements. A passing provider test does not satisfy browser integration. Each section has a separate swarm criterion and names exact observable behavior. Source files and tests stay read-only. No finite test set proves arbitrary mathematical equivalence.

## UTF-16 streaming

Reference `src/common/input/TextDecoder.ts`, class `StringToUtf32`.

- `decode` returns the number of new UTF-32 values and preserves existing Rust vector contents. Input is UTF-16 code units, so lone surrogates are representable.
- Every BMP unit outside surrogate and BOM ranges produces the same numeric output. Every valid supplementary scalar combines the high/low pair. Verify the full range through U+10FFFF, not one example per interval.
- A final high surrogate is retained. The next chunk's leading low surrogate combines with it. A leading non-low surrogate emits the pending high and that unit independently, without interpreting the second unit again as a new high surrogate.
- A high surrogate followed by a non-low unit in one chunk follows the same independent emission rule. Include high/high/low, high/high/high/low, lone low, minimum/maximum pairs and U+FEFF after a high surrogate.
- U+FEFF normally disappears at any position. Preserve the special high-surrogate-followed-by-BOM behavior, which emits both units.
- Empty chunks return zero without flushing state. `clear` discards pending state. Repeated clear and decode after clear must behave like a new decoder.
- Run every original multilingual sample unchanged and the one-unit surrogate streaming assertion. Compare per-chunk output against unchanged JavaScript for persisted deterministic malformed traces.

## UTF-8 streaming and malformed input

Reference `TextDecoder.ts`, class `Utf8ToUtf32`.

- ASCII includes NUL, DEL and control bytes. Valid two/three/four-byte sequences emit their exact codepoint, including U+0080, U+0800, U+10000 and U+10FFFF boundaries. BOM is discarded wherever encountered.
- Run the complete supported codepoint range, excluding source-discarded BOM and surrogate encodings. Keep all nine multilingual sample assertions.
- One, two or three pending bytes survive chunk boundaries. Test every split of representative two/three/four-byte sequences, one-byte progression, steps two and three, split after three bytes in issue #2495, and zero-valued continuation payloads from the original regression cases.
- Invalid continuation discards the consumed prefix but retries the offending byte as a new starter. Include an ASCII offender and another multibyte starter at each continuation position.
- Discard isolated continuation bytes, invalid starters, overlong sequences of each length, UTF-8 encoded surrogates, and values above U+10FFFF. Preserve valid bytes immediately before/after faults. Do not substitute U+FFFD.
- Empty input retains interim bytes. `clear` resets all three public interim bytes. Compare both output and interim state after every differential chunk, including empty chunks.
- Output append/count behavior matches the documented Rust vector mapping. Malformed input must not panic or use unsafe operations.

## UTF-32 to JavaScript string conversion

Reference `stringFromCodePoint` and `utf32ToString`.

- BMP values map directly to u16 units, including NUL, BOM and lone surrogates. Supplementary values map to exact UTF-16 pairs.
- Slicing represents upstream start/end selection; include empty input and partial ranges. The conversion must not silently replace surrogate values with U+FFFD or drop BOM.
- For the u32 input domain, preserve upstream signed 32-bit shift and `fromCharCode` truncation beyond the Unicode range. Include 0x110000, 0x80000000, 0x80010000 and u32::MAX.
- Hash and retain the original conversion assertions and add direct JS comparison for these boundary values. Document that negative, fractional and nonfinite JavaScript numbers require a binding conversion outside this typed API.

## Unicode 11 width and joining

Reference `addon-unicode11/src/UnicodeV11.ts`.

- Version is exactly `11`. Default widths match JavaScript for all 1,114,112 codepoints, including controls, lone surrogates and unassigned values. Out-of-range u32 values retain the source width of one.
- Preserve all BMP/HIGH combining and wide intervals, including endpoint inclusivity. BMP combining fills happen before wide fills, unlike Unicode 6. Generated tables must be reproducible from unchanged source.
- `char_properties` matches raw packed JavaScript results for every codepoint at preceding values 0, 2, 4, 6, 0xffffffff and 0x80000000. This covers empty state, widths zero/one/two/reserved-three and signed state extraction.
- Zero-width values join only if a nonzero preceding property has nonzero width. The joined property inherits the old width. Width and state masking must retain reserved input values.
- Shared helper extraction must leave accepted Unicode 6 full-range and property tests passing. Native Rust tests execute Rust providers, not JavaScript.

## Unicode 15 data and property helpers

Reference `addon-unicode-graphemes/src/third-party/UnicodeProperties.ts` and its bundled trie/inflater.

- For every codepoint, native `get_info` equals the value from the bundled JavaScript trie, including separate lead-surrogate codepoint lookup. Values above U+10FFFF return the bundled error value.
- The port may use generated sorted change points instead of a runtime decompressor. Retain source hashes, copyright and license. Verify the complete generated lookup against the original decompression, not an independent Unicode library/version.
- `info_to_width_info` masks only width bits. `info_to_width` returns one or two with both ambiguous-width settings and all possible property-byte values.
- Preserve forward and backward grapheme-state results, not merely join booleans. Check every before state -32..63 and after property 0..63, including regional-pair state 32.
- UTF-16 `str_width` and `column_to_index_in_context` do not perform grapheme joining. Cover empty input, ASCII, wide characters, ambiguous characters in both modes, valid surrogate pairs, lone high/low surrogates, starts inside/beyond a pair, exact column boundaries, negative columns, NaN and infinity.

## Unicode 15 provider state machine

Reference `UnicodeGraphemeProvider.ts`.

- Version is `15-graphemes` when enabled and `15` when disabled. Default constructor enables graphemes. Ambiguous width defaults false and can change after registration.
- Verify standalone width and raw `char_properties` across all codepoints for all four grapheme/ambiguous-mode combinations and the six preceding values above.
- Preserve the ASCII shortcut only when the preceding signed char kind is zero. Standalone `wcwidth` reports zero for Extend/Prepend, while `charProperties` initially assigns at least one and uses joining. Do not make these methods artificially agree.
- Cover Hangul L/V/T/LV/LVT transitions, Extend, Prepend, SpacingMark, ZWJ to Extended_Pictographic, regional-indicator pairing and the pair-width override. Non-grapheme mode deliberately resets following state to zero as source does.
- Persist seeded multi-codepoint traces so failures identify variant, case and step. Retain all 13 original browser width expected values, including text/emoji selectors, modifiers, family/student ZWJ chains, separated/paired regional indicators, combining accents, Korean Jamo and the unusual U+F3CB input in the weight-lifter assertion.

## Addon registration and disposal

References `Unicode11Addon.ts` and `UnicodeGraphemesAddon.ts`.

- Unicode11 activation registers a new provider under `11` without changing an already active version. Source disposal has no side effects. Test registration order, replacement of an existing version and repeated activation.
- Grapheme activation caches two providers, registers plain `15` before `15-graphemes`, stores the service and the old active version, then activates `15-graphemes` and emits the corresponding change event.
- Dispose restores the remembered old version without unregistering either provider. Before activation, dispose has no effect. Repeat disposal, activate-dispose-activate, activation while already active, and activating on a second service follow source assignment order and event behavior.
- Propagate service errors and event-dispatch errors through the Rust Result mapping. No reference cycles, leaked providers or RefCell borrow held across callbacks. Event callbacks may query and change the active version synchronously.
- Tests must use the actual Rust UnicodeService registration/event contracts, not a mock registry. Keep terminal public API and proposed-API gating integration visible in compatibility records.

## Actual browser terminal integration

The original Playwright addon suites must execute unchanged against the original baseline and equivalent Rust-backed browser bindings in Chromium, Firefox and WebKit. Retain every registration, version and width assertion. Confirm terminal addon loading and proposed Unicode API gating, not just direct provider calls.

Record per-suite discovered/mapped/executed/passed/failed/skipped/unrun counts and actual engine versions. Explicitly mark any engine or Rust browser integration not executed. Provider-only tests cannot satisfy this section. Overall terminal equivalence remains NOT PROVEN until the rest of the terminal and platform contracts also pass.

## Reproducibility and evidence

Every command and result is independently reviewable. Use explicit seeds, source and fixture hashes, debug/release Rust tests, targeted fmt/Clippy plus eventual workspace checks. Check fixtures rather than silently overwriting expected results during ordinary verification. A mismatch retains its full operation trace. No ignored tests, warnings hidden by blanket suppression, browser delegation to the original engine, or fabricated native Linux/macOS results.
