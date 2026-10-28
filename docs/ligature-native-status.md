# Native ligature work status

Full equivalence is NOT PROVEN. Task119 is in progress. The native implementation and tests are written but have not compiled or run yet. The first strict isolated compiler probe was refused by the build-slot resource preflight before rustc started, with 1,025,216,512 bytes free and a required1GiB reserve. Receipt is `results/build-slots/1793151140555-49008-816c3a59-5933-43f1-a6cb-1730e0159b68/receipt.json`.

## Retained implementation

`crates/xterm-addons/src/ligatures/font.rs` owns a complete private module tree for the proposed native font API. It is intentionally not exported from the production ligatures root until a strict pre-integration compile succeeds. This is unfinished integration, not an advertised supported API or a successful no-op.

The files implement bounded SFNT/WOFF table reads, cmap0/4/12/13 and variation-table validation, GSUB table decoding, all source table6 formats1/2/3 and table8 format1 builders, source class/substitution quirks, graph merging, cycle-safe iterative cloning/flattening, stack-safe lookup traversal, range accumulation and shared mutable cached-result identity. An iterative retention pass releases unreachable intermediate graph nodes. The font cache weights keys by UTF16 length and preserves the upstream empty-key TypeError and oversized-key refusal.

Source malformed-table behavior is not yet fully validated. In particular this focused reader does not parse all unrelated outline, hinting, naming and optional metadata structures that opentype.js examines. Error behavior for malformed unused tables may differ and remains work, not a claim of general font-parser equivalence. WOFF2 is unsupported upstream and is rejected; WOFF1 uses bounded pure-Rust miniz_oxide decoding. Every offset and count used by the implemented reader is checked.

## Source evidence actually executed

`node scripts/ligature-font-oracle.mjs --generate` established new fixtures once. A subsequent ordinary `node scripts/ligature-font-oracle.mjs` verified them byte-for-byte without rewriting expected values.

The script executes unchanged assertions from all three original suites through actual Mocha and Chai. Only temporary import/type-only import paths and the test dirname change; source files and assertions remain untouched. Runtime wrappers observe font method outputs and selected merge-test calls without changing returned source objects.

Observed original results are 449 discovered/executed/passed test functions, no failures/skips/unrun, and669 executed/passed assertion calls. The retained traces contain2,540 font operations, including2,048 seeded operations across all four source font files, plus five original merge graph operations, eight original range operations and217 additional source-directed overlap/flatten-cycle/walk-priority graphs. Seed is `0x6C1747E5`. These are JavaScript expected values, not Rust pass counts.

`results/font-addons/ligature-oracle.json` records per-test outcomes, exact counts, source hashes, fixture hashes and Node version. `ligature-oracle-check.log` retains the verifying run. All four font binaries are unchanged tracked test fixtures, with original embedded notices and full licence bodies in `licenses/ligature-font-fixtures.md`, `OFL-1.1.txt` and `Ubuntu-font-1.0.txt`. Production Rust never loads an oracle-generated lookup tree.

The new native tests replay glyph inputs/outputs, exact ranges, cache object/array identity, original merge graph mutations/aliasing and every original range assertion. Additional tests cover mutable result sharing, no-group array aliases, malformed containers and the217 directed tree cases. The directed fixture addition was checked not to change any of the three existing fixture hashes. Their current executed count is zero. Do not infer a pass from test existence.

## Next checks

1. Restore approved build headroom and obtain a normal FIFO native lease. Do not lower the reserve or create an alternate Cargo target to manufacture a pass.
2. Compile the actual module tree through the small pre-integration wrapper `tmp/font-probe.rs` with warnings denied and the existing miniz_oxide rlib. This checks production source before exposing it to every swarm build. It is not whole-workspace integration evidence.
3. Run the same actual modules' native tests with existing serde_json/miniz dependencies, fix every mismatch and retain counterexamples. Then export the production module atomically and run the package/workspace gates through approved slots.
4. Extend synthetic table/graph cases and malformed-font comparison for uncovered branches, including the focused-reader error gap above. Obtain independent code and evidence review.
5. Connect the real font API to task40's browser font resolver/joiner lifecycle and run actual Rust terminal/browser integration. Metadata task158 and the earlier helper112 remain separate scopes.
