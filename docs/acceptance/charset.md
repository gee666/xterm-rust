# Charset acceptance contract

This is the component checklist for `common/data/Charsets.ts`, `common/services/CharsetService.ts` and their use by `InputHandler.ts` and `CoreTerminal.ts`. It replaces reliance on the broad charset summary as a quality gate. Component unit-test success does not satisfy terminal integration checks.

## Reference and execution

The oracle reads the supplied, unchanged xterm.js 6.0.0 reference. `results/charset/oracle.json` must record SHA-256 hashes for both source files and `CharsetService.test.ts`. No fabricated upstream revision is allowed. `scripts/charset-oracle.mjs` may change only interface imports in temporary source copies and test imports. It must preserve every implementation statement and original assertion body. Temporary files belong under `tmp/charset-oracle/`.

Required commands, run at the repository root:

```sh
node scripts/charset-oracle.mjs
node scripts/check-charset-oracle.mjs
cargo test -p xterm-core --test charset --test charset_edges
cargo test -p xterm-core --release --test charset --test charset_edges
cargo clippy -p xterm-core --lib --test charset --test charset_edges -- -D warnings
rustfmt --edition 2021 --check crates/xterm-core/src/charset.rs crates/xterm-core/src/charset_tables.rs crates/xterm-core/tests/charset.rs crates/xterm-core/tests/charset_edges.rs
```

Record actual tool versions, platform, exit codes and logs. The checked-in evidence observed Windows x64, Node v24.15.0 and Rust/Cargo 1.85.1. Linux, macOS, WASM and browser results are not implied by those runs. Workspace quality gates remain separate.

## CSET-MAP: every lookup and alias

Required source paths: the full `Charsets.ts` table and `DEFAULT_CHARSET` initialization.

- The exact exported designator set is `0 4 5 6 7 = A B C E H K Q R Y Z`. `B` is present but undefined. There are 15 defined designators and 12 distinct maps.
- Compare all 128 ASCII lookup inputs for each designator and unknown `X`, exactly 2,176 fixture rows. Check absence as well as presence. An extra mapping is a failure.
- `0` selects DEC special graphics, including control pictures, box drawing, scan lines, mathematical symbols, and UK pound. Do not replace these with a different standard's table.
- National maps are British, Dutch, Finnish, French, French Canadian, German, Italian, Norwegian/Danish, Spanish, Swedish and Swiss. Every source entry must be present with exact UTF-16 values.
- `C` and `5`, `E` and `6`, and `H` and `7` share map identity, not just equal contents. Repeated lookup of any defined designator retains identity. Distinct map definitions do not share identity accidentally.
- `DEFAULT_CHARSET`, `B`, unknown ASCII designators, and unknown supplementary designators return no map. Add tests for unknown non-ASCII BMP and surrogate-invalid binding inputs where bindings accept raw strings.
- Map keys are ASCII. An unexpected non-ASCII key cannot escape coverage by falling outside the fixture's 128 inputs.
- Dutch `[` maps to the complete two-unit string `ij`, not `i`. Stored table values must not incorporate InputHandler's later printing truncation.

Existing evidence: `every_ascii_mapping_matches_javascript`, `aliases_share_identity_and_undefined_sets_have_no_map`, `charset-maps.tsv` and `charset_edges.rs::builtin_identity_is_stable_and_distinct_except_declared_aliases`. The latter checks repeated identity, every designator pair and unknown NUL/BMP/supplementary designators. Raw surrogate-invalid binding inputs apply only if a binding accepts raw designator strings.

## CSET-STATE: designation, invocation and direct access

Required source paths: every statement in `CharsetService.ts`.

- A new service has `glevel = 0`, no active charset, and no designated slots.
- Designating an inactive level preserves the active level and active charset.
- Invoking a designated level selects that exact map identity. Invoking an unassigned or out-of-array level selects no map and does not grow the slot array.
- Designating the currently active level immediately refreshes the active charset, including replacement with no map.
- G0, G1, G2 and G3 must all work. Directed tests also cover sparse level 7 and level 16. Earlier unassigned slots remain absent values and observable array length reaches the designated index plus one.
- Overwriting a lower level must not truncate higher designations. Repeated designation of the same identity must not create a new map.
- The getter exposes the current designated array. Direct slot assignment, truncation, clearing, append and out-of-range growth must not refresh the separate active snapshot. A later invocation must observe the changed designated array.
- Direct writes to the public active charset and level remain independent until another service operation updates them, matching the source's public fields.
- Reset selects level zero, clears all designated slots and active charset, and remains safe when repeated. A service can be reused after reset without retaining stale designations.
- Preserve all assertions in all four `CharsetService.test.ts` cases. Rust test aggregation must not hide a missing original case.

Existing evidence: the four named upstream service tests, `direct_array_edits_do_not_refresh_the_active_charset`, and `seeded_designation_trace_matches_javascript`. The trace has 11 directed plus 4,096 xorshift32 operations, seed `0x0c4a25e7`, exactly 4,107 rows. Compare active level, active identity, complete slot values, and array length after every operation.

## CSET-UTF16: custom maps and ownership representation

Required source paths: `Types.ts::ICharset`, `CharsetService.ts`, and `InputHandler.ts::print` charset lookup.

- Custom maps are accepted, not only the predefined maps.
- Replacement values retain empty strings, embedded NUL, BMP units, supplementary pairs, and isolated high or low surrogate units without replacement-character conversion.
- Keys are single UTF-16 units in Rust. A custom key containing a surrogate unit must remain representable. Arbitrary multi-unit JavaScript object keys have no single-unit terminal lookup and must be identified as a representation difference, not silently claimed as supported.
- Designation, active selection and cloning a service retain shared map identity. Resetting one cloned service must not destroy the other service's selected map.
- The `Charset` handle must share live mutable object identity, not only immutable table data. A per-print-run captured handle observes property edits on that object, while replacing the active handle does not replace the captured object. `get_first`, `get`, `set`, `remove` and `snapshot` must release every map borrow before Unicode providers or user callbacks execute.
- Preserve explicit undefined properties separately from absent properties and empty strings in snapshots. `Charset::clone` shares identity; returned lookup values and snapshots are owned copies. A source callback's in-place `q` edit must change the next mapped character even in one whole print run.
- Built-in aliases and repeated lookup share maps within a thread-local realm. The non-Send core must not add global locking or leaked maps. Native threads have separate registries; browser worker proof is separate.
- Do not convert map strings through Unicode scalar-only `String` on the raw API path. The later terminal-printing contract is separate.

Existing evidence: `custom_maps_preserve_utf16_empty_values_and_identity` and `charset_edges.rs::custom_maps_preserve_all_raw_utf16_value_boundaries` cover empty and embedded-NUL replacements, valid supplementary pairs, lone high/low surrogates, surrogate keys, identity, clone retention and reset. `charset_mutation.rs` adds shared selected/saved/cloned edits, undefined/empty/absent distinction, owned read results, live built-in aliases and native thread realms. `charset_terminal.rs` compares both handle replacement and in-place edits against actual source callback traces.

## CSET-ERROR: growth and failure atomicity

Required source path: `CharsetService::set_g_charset` allocation boundary.

- Rust levels are nonnegative `usize` indices. Negative, fractional, nonfinite, and non-array JavaScript property keys are not accepted Rust indices. Bindings must reject unsupported inputs explicitly rather than coerce them into a valid level.
- Designating `usize::MAX` returns `LevelOverflow`; it must not wrap the required length to zero.
- A capacity-overflow request such as `usize::MAX - 1` returns an allocation error without attempting a huge resize after failure.
- Either error leaves active level, active identity, slot length and every existing slot unchanged.
- After either error, ordinary designation, invocation and reset still work. Errors must not poison the service.
- Invoking an out-of-range level does not allocate, panic or modify slots. It records the requested level and selects no map.
- The `Result` representation and its difference from JavaScript allocation exceptions must be documented. Never add a successful no-op to hide an invalid index or failed allocation.

Existing evidence: `failed_growth_leaves_the_service_unchanged` covers both errors, level/length/identity, and maximum-level invocation. `charset_edges.rs::both_growth_failures_preserve_every_slot_and_allow_recovery` checks four existing slot identities, then successful designation, invocation, reset and reuse.

## CSET-ORACLE: reproducible, independent evidence

- Discover, execute and pass exactly four original suite cases with zero failed, skipped or unrun cases. Name every original test in the report.
- Fixture generation is deterministic. Repeated generation with unchanged reference and seed produces byte-identical map and state TSV files.
- Record seed 206185959, 2,176 mapping rows, 4,107 state rows, canonical identity encoding and reference hashes. The canonical aliases are Finnish `5`, Norwegian/Danish `6` and Swedish `7`.
- Tests must fail for a changed map entry, alias identity, active-level result, slot length, or slot value. Test counts must be asserted so empty or truncated fixtures cannot pass.
- An independent reviewer reads the source and Rust implementation, checks the original assertions, executes the commands, and retains their own findings. Teammate claims and configured CI are not runtime evidence.
- Retain failing input rows as counterexamples; do not edit expected fixtures to make Rust pass. Report unrun integration/platform checks explicitly.

Existing evidence: `results/charset/oracle.json`, `results/charset/independent-review.json`, reviewer logs and the separate acceptance reviews for tasks 20 and 21. `scripts/check-charset-oracle.mjs` confirms byte-identical regeneration and runs all twenty-one integration tests against temporary source mutants for a map value, alias identity, active level, slot length and slot value. All five must fail through assertions, not compilation or process errors. Results are in `results/charset/oracle-controls.json`. Temporary executables are deleted after checking to conserve disk.

## CSET-INTEGRATION: terminal sequences and printed results

These are not satisfied by the standalone charset service. They belong to the real parser, buffer, InputHandler and CoreTerminal integration.

- For every recognized designator, `ESC (` designates G0, `ESC )` and `ESC -` designate G1, `ESC *` and `ESC .` designate G2, and `ESC +` designates G3. `ESC /` is registered but intentionally performs no designation in the source; preserve this observable result and return handling.
- SO byte `0x0e` invokes G1. SI byte `0x0f` invokes G0. Locking shifts `ESC n` and `ESC }` select G2, `ESC o` and `ESC |` select G3, and `ESC ~` selects G1. Preserve the source's limited GL/GR behavior, not a different terminal standard's ideal implementation.
- `ESC % @` and `ESC % G` invoke G0 and designate the default map only in G0. Other designated levels must survive. Direct `selectCharset` calls with length other than two take this same default path.
- For every map, printing tests compare resulting codepoints, widths, cells, cursor and accessibility events. Mapping is applied only to input codepoints strictly below 127. Check 126, 127, 128 and a supplementary codepoint explicitly.
- Lookup uses a one-unit key. Only a truthy replacement changes printing; empty replacement leaves the original codepoint. A nonempty replacement uses its first UTF-16 unit only, so Dutch `[` prints `i`, not `ij`. Embedded NUL as the first replacement unit is still a nonempty replacement.
- A `print` call snapshots the active map at its start. Tests involving reentrant events must not refresh the map halfway through a run unless the source does.
- Soft hyphen `U+00AD` is ignored before mapping. Charset changes cannot turn it into a printed cell.
- DECSET private mode 2 resets designated G0 through G3 to the default map. Verify active-level effects, not just the slot array.
- DECSC saves active charset, a shallow copy of all designated slots, and active G level. DECRC restores saved slots and invokes the saved level. Test missing saved maps, aliases, later designation changes, alternate buffers and saved-array lengths. The source restores saved slots but does not explicitly truncate extra current slots; preserve that behavior.
- Soft reset, full reset and public CoreTerminal reset must match their source charset state and saved-state behavior. Test reset while an alternate map is active and reuse afterwards.
- Feed designation/shift/print/reset sequences as one string, one byte/unit at a time, and at every internal boundary. UTF-8 byte and UTF-16 string input must produce the same supported results.
- Run the same integration cases through headless and actual Rust-backed WASM/browser entry points. Compare against the unchanged JavaScript terminal. Native service tests are not browser evidence.

Required evidence: a named Rust/JS terminal differential suite with all operations above, full cell/state/event comparisons, chunk-boundary cases, logs and exact pass/fail/skipped/unrun counts. At this writing, this integration evidence is missing. Keep its criterion incomplete until it exists.
