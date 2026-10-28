# Ligature shaping source branch matrix

Task 119 owns GSUB/cmap/shaping/cache. Task 40 owns browser font discovery and addon lifecycle. Task 112 owns CSS parsing/fallback matching. These are acceptance obligations, not completed behavior.

All paths below start with `source_repo/addons/addon-ligatures/src/`.

## Metadata resolution and async addon state

`font.ts`: prefer legacy `navigator.fonts` to modern `window.queryLocalFonts`. Permission requests are optional. Ignore a thrown TypeError or non-Error; reject a denied status or other Error and do not cache that failure. Cache successful query metadata and empty query failures, logging only Error name/message. Concurrent first queries may both run because the source caches after enumeration, not before. Keep dynamic family-key grouping, first metadata selection, absent blob stopping, blob/arrayBuffer errors and generic family stop order. Generic names are case-sensitive. Missing APIs and unmatched families return no font. CSS parsing happens after catalog loading, preserving final-quoted-family omission and invalid escape errors. The Rust service shares cache through clones and a host-owned catalog rather than a mutable global.

`index.ts`: on first truthy family or family change, clear the font, enter loading and launch a request; all loading/error/no-font states use fallback. Completion compares the request family with the terminal's current family, not a monotonically increasing generation. Thus changing A to B to A can allow an old A completion, exactly as source. A current found font enters loaded and refreshes rows 0 through rows-1. A current no-font result enters loaded without refresh. Current failure enters failed and clears the font. Debug logging receives the previous loadError, then stores the new error. Stale success/failure has no effect. An empty family does not trigger another load and can leave a previously loaded font usable. Clone range pairs when returning shaped results from the joiner so callers cannot mutate its internal cache.

## Table and processor conversion

`fontLigatures/tables.ts`, `types.ts`: preserve each coverage/class/substitution/lookup field and distinction between an individual glyph and a half-open glyph range. Typed Rust may reject malformed table representations, but record every rejection and compare actual font parse errors rather than silently choosing no ligatures.

`processors/coverage.ts`: format1 returns the first glyph index; format2 lookup returns the stored range.index, not glyph offset. Enumeration of format2 uses the range's array position as index, turns singletons into individual glyphs and larger inclusive bounds into end+1 ranges. Do not replace this with conventional OpenType coverage behavior without differential proof.

`processors/classDef.ts`: only format2 has class membership. Other formats return null membership/empty lists. Individual lookup selects the first inclusive containing range. Class lists preserve source ordering and singleton/range representation. Range splitting uses a fixed initial class and start in the supplied code, even when later classes differ. Port that behavior or retain counterexamples explicitly; do not assume a corrected segmentation algorithm is equivalent.

`processors/substitution.ts`: absent coverage is null. Format1 uses JavaScript remainder of glyph+signed delta by 65536, including negative results. Format2 uses coverage index and null for a missing substitute. Range substitution retains the fixed initial replacement/start behavior of the reference and Map key identity, including overlapping entries. Test negative delta, first/last covered glyph, missing replacements, singleton and cross-class/range boundaries.

`processors/helper.ts`: preserve sequenceIndex filtering, lookup/subtable order, first non-null individual substitution and range branch's exact `!every(val !== null)` condition. For individual inputs insert the actual new entry in the tree. For a range broken into an individual, the source places a different empty object in individual and returns the local entry; keep this observable distinction in table-to-tree fixtures. Input positions replace forward trees; lookahead/backtrack retain existing subtrees and share one entry among alternative glyphs. Deduplicate by entry identity, not value. Preserve substitution array sharing along lookahead/backtrack paths.

`processors/6-1.ts`: enumerate first coverage in order; skip null rule sets; process input forward, then lookahead, then backtrack. Lookup metadata is input length+1, table index, rule subIndex and signed backtrack/context end.

`processors/6-2.ts`: resolve first input class; skip null membership/class sets; build each class rule separately using input/lookahead/backtrack class definitions, then merge all result trees. Preserve rule order and table/subindex priority.

`processors/6-3.ts`: use first inputCoverage to seed; process remaining coverages, lookahead and backtrack; lookup length is full inputCoverage count, subIndex zero, exact signed context range.

`processors/8-1.ts`: enumerate coverage and choose substitute by that source enumeration index; process lookahead/backtrack; lookup length one, subIndex zero and context includes backtrack plus one input and lookahead. This lookup later walks the glyph sequence in reverse.

## Merge, flatten, traversal and range accumulation

`fontLigatures/merge.ts`: cover empty and multiple trees, individual/individual, individual/range, range/individual and range/range overlaps. Keep earlier tree preference on equal index/subindex, then lowest index and lowest subindex selection. Preserve insertion/enumeration order, cloning versus sharing and visited entry-pair protection. Test adjacent/nonoverlapping/contained/containing/partial overlaps at both bounds. In particular `getIndividualOverlap` considers only `first > end` disjoint, not `first >= end`; test the exact endpoint quirk. Preserve lookup arrays/contextRange clones and identity of cyclic/shared graphs without leaking ownership cycles.

`fontLigatures/flatten.ts`: individual entries are processed before range expansion, so ranges overwrite a colliding individual. Glyph intervals are end-exclusive. Memoization preserves shared entry identity and terminates graph cycles. Lookup payload is retained rather than semantically merged at flatten time.

`fontLigatures/walk.ts`: missing start glyph returns none. Compare root, reverse and forward results using source priority. Forward and initial reverse candidates compare index then subIndex. Deeper reverse traversal compares only index, not subIndex. Test start/end of sequence, no previous glyph, empty trees, longer lower-priority matches, ties and shared subtrees.

`fontLigatures/mergeRange.ts`: port all nine branches. Empty list, before all, inside/overlap front, containing/overlap end, after current, gap after previous, extending into current, spanning multiple ranges, and extending past last. Adjacent ranges remain separate because source comparisons do not merge touching intervals. Retain mutation order and returned list identity where the language representation exposes it.

## Shaping, glyphs and cache

`fontLigatures/index.ts`: select only calt features, preserving allLookups order rather than feature-index list order. Duplicated selected indices do not duplicate groups. Unsupported types/formats still contribute source-exact empty group behavior. Type6 walks forward; type8 backwards. `_getNextLookup` starts at the requested group and retains the source first/last calculations, including an earlier-index group discovered later in the sequence. Apply only non-null substitutions, merge signed context-relative ranges, extend forward scan end after replacement and skip the matched input length in each direction.

Glyph input uses JS string iteration, which combines valid surrogate pairs but retains lone surrogates. Shaped context ranges are glyph-sequence positions, not remapped UTF-16 offsets. Fallback matcher positions are UTF-16 offsets. Test supplementary text before and inside a potential ligature rather than silently converting one representation into the other.

A zero/absent cacheSize disables cache. A positive size measures input key UTF-16 length, not bytes, glyph count or entry count. Test empty keys, keys exactly at/above capacity, LRU get promotion, eviction, replacement and calls alternating findLigatures/findLigatureRanges. A cached full result can serve ranges; cached ranges alone cannot serve full glyph data. A font with no groups returns empty ranges without glyph lookup or cache, and findLigatures returns unmodified glyphs. Source full-result getters return cached objects directly; the addon joiner clones range pairs separately. Rust ownership decisions must be documented and compared through all observable APIs.

## Required evidence

Port unchanged assertions from `index.test.ts`, `merge.test.ts` and `mergeRange.test.ts`. Account for exact assertion calls separately from test bodies. Compare all four supplied fonts, directed synthetic table/graph cases above, and seeded malformed/boundary/random traces against source. Keep source and fixture hashes, seeds, raw glyph arrays and ranges, failures and saved counterexamples. Validate font parsing from bytes rather than consuming an oracle-generated lookup tree as the production engine. Native/WASM/browser and independent review gates apply. Current implementation and evidence gaps remain NOT PROVEN.
