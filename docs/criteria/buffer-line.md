# BufferLine acceptance criteria

Source contracts are `source_repo/src/common/buffer/BufferLine.ts` and its complete `BufferLine.test.ts` suite. These criteria cover BufferLine only. They do not establish terminal, renderer, addon or platform equivalence.

## Packed storage and cell contracts

- Construct zero-column, ordinary and wrapped rows. Defaults are empty content, width one, foreground/background zero. Custom fills preserve raw signed-shift width behavior, including the upstream width 456 assertion.
- Preserve content/foreground/background u32 words, every width/content/combined/protection mask, and codepoint access. Packed values are data, not Boolean-normalized flags.
- `get`, `get_cell`, `load_cell`, `get_width`, `has_width`, `get_fg`, `get_bg`, `has_content`, `get_code_point`, `is_combined`, `is_protected`, `get_string` and `get_extended` agree on every retained cell after each mutation.
- `load_cell` replaces a reused cell's combined and extended data. Reading an ordinary cell after an extended cell must not reset the line's earlier extended entry.
- Legacy `set` preserves background and treats all multi-unit strings as combined, including a single astral scalar. `set_cell` follows CellData's scalar-aware packing instead.
- A combined cell's legacy code is its last UTF-16 unit, not its last Unicode scalar. Preserve empty strings, lone surrogates, surrogate pairs, BMP text, combining strings and supplementary-plus-combining strings without lossy Rust String conversion.
- `set_cell_from_codepoint` preserves raw attribute words and extended identity. `add_codepoint_to_cell` creates width one on empty content, creates or extends combined strings, preserves attributes, and changes width only for a nonzero supplied width.
- All extended assignments share the same mutable object and payload identity. Explicit ExtendedAttrs cloning remains distinct and retains its upstream payload-dropping contract. Source cell changes through a shared extended handle must remain visible through fill, load, clone, whole-row copy and range copy.
- Rust boundary differences are explicit. Missing cells return Option, no terminal input can panic through unchecked row indexing, and zero-length edits terminate.

Evidence: constructor, combining and legacy unit assertions in `tests/buffer_line_upstream.rs`; shared-identity and reuse assertions in `tests/buffer_line_extended.rs` and `tests/buffer_line_differential.rs`; full snapshots in the seeded differential fixture.

## Editing and protection

- Insert shifts right in reverse order, fills the requested range, clamps counts to the row end and applies source modulo positioning on nonempty rows.
- Delete shifts left in forward order and fills the vacated suffix. Zero, oversized and end-position counts are included in differential cases.
- Insertion clears the left half of a wide character when insertion starts on its second cell, and clears an orphan wide first half at the row end.
- Deletion repairs both a preceding wide first half and a blank zero-width result at the deletion start.
- Replacement repairs split wide characters at both ends before filling the bounded range. Port every upstream start-zero and start-one expected string without changing assertions.
- `respect_protect` preserves flagged cells during fill and replacement. Boundary repair checks the same protection cell as the source. Test both protected and unprotected wide-character boundaries.
- Range copies preserve raw words, combined strings and extended identity. Both traversal directions matter for overlapping self-copies. Rust exposes `copy_cells_within` for aliasing self-copies and `copy_cells_from` for distinct rows.

Evidence: every upstream insertion, deletion, replacement and fullwidth assertion in `tests/buffer_line_upstream.rs`, `tests/buffer_line_translation.rs`, and `tests/buffer_line_extended.rs`; randomized protected edits and both range directions in `tests/buffer_line_differential.rs`.

## Resizing, allocation and whole-row copying

- Resize grows using the supplied fill and preserves all old active cells. Shrink removes cut-off combined and extended entries. Shrink then regrow must never resurrect stale strings or extended flags.
- Preserve wrapper state across resize. Cover no-change size, zero columns, retained backing allocation, growth beyond allocation and repeated shrink/grow traces.
- Resize returns whether backing storage exceeds twice the active row size, with a strict greater-than threshold. `cleanup_memory` returns whether cleanup happened, does not change active cells and is idempotent.
- `copy_from` replaces length, contents, wrapped state and active sparse entries, using existing backing storage when active lengths agree. Clone creates independent packed storage but shared extended object handles.
- The source's `blank` optimization is documented as valid only for genuinely blank rows. It clears sparse maps without scanning or silently synthesizing combined data.

Evidence: five upstream resize assertions, clone/copy cases and combining-map cardinality checks; differential snapshots and return values for resize, cleanup, clone, whole-row copy and range copy.

## Translation, UTF-16 columns and cache

- Translate empty, ASCII, astral, combining, fullwidth and explicit trailing-space rows. Port all original expected strings and all original column arrays.
- A column array has one entry for each UTF-16 unit plus a terminal column. Repeated units of one cell map to the same starting column. Fullwidth cells advance by two and can pass the requested end column.
- Zero-width or missing cells advance by at least one to guarantee progress. `end_col=0` yields an empty result and the starting terminal column.
- Right trimming uses content/width, not whitespace trimming, when actually translating a row. `get_no_bg_trimmed_length` also considers the background color mode.
- Only full-row requests with default or zero start, no explicit end and no column array use the canonical cache. Partial requests never consume or overwrite it.
- A trimmed-only cache is reused only for trimmed requests. An untrimmed canonical request refreshes it. A cached untrimmed value supplies trimmed requests through JavaScript trimEnd behavior, including U+FEFF and excluding U+0085.
- Preserve the source's observable difference between an uncached content trim and trimEnd of a cached untrimmed row. Do not normalize this away.
- Every mutation invalidates canonical text, including set, set_cell, codepoint set/add, insert, delete, replace, resize even at the same size, fill, copy_from and range copy. Cleanup alone does not invalidate text.

Evidence: 13 original translation/fullwidth cases in `tests/buffer_line_translation.rs`; both original cache tests plus explicit JavaScript whitespace checks inside `buffer/line_text.rs`.

## Faithful verification and independent review

- Discover and execute all 51 tests in the original BufferLine test file unchanged. Record the 44 BufferLine cases separately from its six AttributeData cases and one CellData case.
- Map each of the 44 BufferLine cases to Rust assertions. Do not count the neighboring attribute/cell suites twice as new BufferLine coverage. Retain exact duplicate original resize assertions even where names differ but setup does not.
- Use a seeded oracle with recorded Node/toolchain versions and source hashes. Retain operation inputs and expected full packed cells, every primitive getter, combined strings, extended words/URL, trimmed lengths, wrapped state, translated UTF-16, column arrays and method return values.
- Execute at least 8192 seeded operation steps spanning every mutation family, varying row lengths through zero, shrinking/regrowth, protected cells, malformed UTF-16, fullwidth pairs and overlap traversal.
- Run targeted Rust unit and differential tests, rustfmt and warning-free Clippy. Keep raw results in `results/buffer-line/`, not private swarm runtime data.
- A reviewer other than the implementation owner inspects implementation and source, regenerates the oracle and executes Rust checks. Record failures and their fixes. Native Windows results do not establish Linux or macOS runtime evidence.

Observed Windows evidence and the exact 44-case mapping are in docs/buffer-line.md and results/buffer-line/. All 51 original baseline tests and 8192 seeded operation steps passed. Overall project equivalence remains NOT PROVEN.
