# Cell and attribute acceptance matrix

Scope is the full CellData.ts and AttributeData.ts behavior, including the AttributeData and CellData describes in BufferLine.test.ts. Browser/public API exposure is a separate gate, not established by these unit tests.

## C1. UTF-16 content and legacy CharData

- Default content is zero, width is zero, chars are empty and code is zero.
- Empty legacy strings use JS NaN-to-int32 conversion and become zero content, with requested raw width retained.
- One UTF-16 unit, including NUL and lone high/low surrogate, stores that unit directly. NUL reads back as an empty string.
- Exactly two units form a single codepoint only for high-surrogate then low-surrogate. Every other two-unit arrangement is combined, including low/high, high/BMP, BMP/low and two BMP units.
- Every string longer than two units is combined, including multiple valid surrogate pairs and grapheme clusters.
- Combined getCode returns the last UTF-16 unit, not last Unicode scalar. An empty raw combined string returns absence, corresponding to JS NaN.
- CharData input code is ignored. Foreground is replaced, background cleared, while extended object identity and stale combined storage remain untouched on noncombined input.
- getAsCharData returns current foreground, reconstructed chars, arithmetic-shift width and computed code. Raw signed widths include 0,1,2,3,456,-1,i32::MIN,i32::MAX, not just valid terminal widths.
- Raw content masks are exactly upstream's values. Out-of-Unicode-range packed codepoints retain the original UTF-16 helper's truncation instead of silently replacing them.

Evidence must include named branch tests and all 1024 original-JS CharData fixtures from seed 0xce11da7a.

## C2. Packed colors and flag queries

- Each foreground flag and background flag changes only the corresponding query. Queries return original masks, except underline's extended-style path returns 1.
- All four color modes are tested separately in foreground and background. Palette modes mask to low 8 bits; RGB masks to low 24; default color returns -1 even with nonzero low data.
- Foreground/background palette, RGB and default predicates follow mode bits, not payload.
- isAttributeDefault requires both raw words to be exactly zero, irrespective of extended object contents.
- fromColorRGB masks each incoming channel to 8 bits. toColorRGB discards all upper flag/mode bits.
- Numeric Rust u32 versus JS signed high-bit representation is documented, not treated as normalized boolean equality.
- Underline color data without HAS_EXTENDED is ignored. With HAS_EXTENDED, default underline color reports default mode but getUnderlineColor falls back to foreground. Palette/RGB modes use their own payload.
- Underline style uses the foreground UNDERLINE flag, unlike isUnderline's extended-only path. Both directions and empty/nonempty extended style are tested.

Evidence must include an exact mode/flag matrix and the raw-cell oracle snapshots.

## C3. Extended object mutation, sharing and cloning

- Every style value 0 through 7, offset 0 through 7, and all four color modes preserve unrelated packed fields. Larger setter inputs mask like JS shifts.
- URL nonzero forces dashed in style and ext getters but does not destroy the stored style. Clearing URL restores it. URL replacement retains identity.
- Color/offset alone are empty; style, URL or payload independently make the object nonempty. updateExtended changes only HAS_EXTENDED and preserves every other background bit.
- share retains mutable object identity and payload Rc identity; mutation through either handle reaches the other without keeping a RefCell borrow across public calls.
- Explicit ExtendedAttrs.clone creates a different object, retaining stored raw style and URL while dropping payload. AttributeData.clone copies fg/bg verbatim, including potentially stale HAS_EXTENDED, and uses that explicit clone.
- CellData's Rust Clone is a language addition using the same AttributeData cloning rule; buffer storage must not mistake it for JS reference assignment. BufferLine uses share for load/copy/storage instead.

Evidence must include named mutation/isolation tests and JS-generated operation traces, not only random initial words.

## C4. Attribute equality decision paths

- Different characters/content/width do not affect equality.
- Foreground/background color mode and decoded color differences independently fail equality. Same mode with irrelevant high/unused color payload bits remains equal.
- Inverse, bold, numeric underline, overline, blink, invisible, italic, dim and strikethrough each independently affect equality.
- Protection, URL identity and underline offset are excluded except where URL's forced visible style changes another compared query.
- With underline present, style differs => false. If both underline colors are default, payload colors are ignored. Exactly one default => false. Two explicit colors compare both mode and decoded payload.
- Without underline, differences in extended underline color/style must follow the source's separate numeric isUnderline query, not a guessed rendering rule.
- Equality is reflexive and symmetric across all oracle pairs, including raw inconsistent fg/bg/extended combinations.

Evidence must include deterministic cases for each return branch plus 2048 seeded original-JS raw cell pairs and snapshots.

## C5. Oracle, mapping and review

- Preserve upstream source and assertions. Execute all five CellData.test.ts tests and all seven relevant BufferLine.test.ts tests without weakened assertions.
- Run each corresponding Rust test and enumerate discovered, executed, passed, failed and unrun counts. Do not call the remaining BufferLine suites passed from this subset.
- Generate source hashes, exact Node/platform/toolchain metadata, seed 0xce11da7a, fixture row counts and raw logs. Regeneration followed by tests must reproduce fixture contents.
- Run focused rustfmt, warning-free Clippy, debug and release tests. Workspace quality snapshots must label unrelated concurrent failures rather than overwrite or suppress them.
- Independent review inspects source branches, the assertion mapping and shared identity interactions, then reruns the stated commands. Keep full terminal equivalence NOT PROVEN while broader APIs, browser integration or native platform evidence is absent.
