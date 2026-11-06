# Eager CFF2 font dictionaries

Task546 follows accepted533. The source calls gatherCFF2FontDicts after validating the whole FDArray INDEX. Each font DICT is parsed in order, followed immediately by its private DICT and local Subrs INDEX. A failure in the first font's private data precedes errors in later font dictionaries. FDSelect, charstrings, CFF1 interpretation and other342 metadata remain outside this task. Full equivalence is NOT PROVEN.

## Implementation

The lexical parser is shared with the existing CFF2 top dictionary. Byte operand, real-prefix, blend and deferred duplicate rules stay in one implementation. Private DICT size controls the parser loop, not the underlying whole-font DataView boundary. An operand can consume bytes past its declared size. Each address retains floating arithmetic until DataView ToIndex truncation, including NaN and negative fractions. Private zero offsets or sizes skip reads; null defaults produce no reads. Local Subrs arrays preserve string addition of relative private offset, operand and table base.

All FDArray offsets and object ranges are checked before interpreting the first font, preserving eager INDEX error ordering. Empty/reversed objects have default private[0,0] and require no dictionary work. Valid numeric contiguous objects borrow input; negative fractional starts reconstruct individual reads because both -0.5 and +0.5 truncate to byte0. Compressed reads reuse the bounded DEFLATE range implementation. No new dependency, public API, unsafe code or JavaScript font-engine delegation is introduced.

The predecessor INDEX-only phase and its original assertions remain intact. Production applies the new font stage only after that phase. New dedicated tests compare the expanded eager phase and complete source constructors rather than silently changing predecessor expectations. The existing test-only SFNT/WOFF writer was extracted unchanged into font_cff_container.rs. New browser tests reuse it without recursively rerunning every predecessor corpus; native tests retain the complete predecessor assertion chain.

## Safety limits and unproven behavior

Existing negative-blend and string-range nontermination divergences remain explicit. An infinite private size at a NaN address repeatedly reads byte0 forever in source. Rust preserves an immediate invalid operand or missing byte error, otherwise rejects that nonterminating loop explicitly. The dangerous source input is not launched. A multi-element Subrs array can make the source allocate billions of identical empty offsets and objects. Native directed tests cover bounded Rust traversal only, not source resource-error equivalence. These safety decisions are documented divergences, not approved equivalence.

## Verification plan

Source hash737212d8aae53d1996ef10b315e4369a1542ebb9b94c13c5f99793914745d484 pins unchanged opentype.js. Test-only in-memory exports expose existing parseCFFIndex, parseCFFTopDict and gatherCFF2FontDicts bodies; full constructors still use the original installed source. Each source child has128MiB heap/3000ms deadline. Unknown worker failures fail the run. Hash and retain exact malformed recipes, original outcomes, warnings and any resource failures. Seed0x546cff02 currently defines408 recipes, including4 unchanged good-font controls and404 eager-phase recipes. Directed cases include fractional/NaN private fields, order-sensitive empty-array Subrs addition, whole-INDEX error precedence and compressed private/Subrs reads beyond the retained32768-byte prefix. Explicitly hazardous infinite-size/array-allocation inputs are native safety observations only.

Source generation and immutable replay completed408 recipes.396 complete constructors and392 table-only eager phases were observed. Twelve Subrs=1 cases reached a huge count with width0 and exhausted the source child's128MiB heap before reporting either phase or constructor. Exact inputs and stderr are retained; none are dropped or called equivalent. All12 are excluded from source browsers. Rust executes them as bounded observations only.

Final native Windows changed-addon all-target/all-feature tests passed222 functions across52 harness summaries,0 failed/ignored/measured/filtered/warnings. Strict native-addon and WASM-font all-target/all-feature lint and native/WASM dependent checks passed. Initial lint found duplicate module loading/module inception in the new shared test wiring; conditional imports fixed the causes without suppression. Exact workspace format still fails WindowsOS206; five complete member checks passed. WASM test no-run passed.

The negative bypass executed184 full constructor mismatches, including36 duplicate0 mismatches and all4 designated addon-bad cases. Exact source SHA restoration and positive replay passed. Source/Rust three-browser constructor/addon lifecycle matrices remain pending under a fresh grant. No full workspace/platform/equivalence claim.

## Ownership

Read docs/intent-nefertiti546.md for the seven private533 prerequisite exclusions. Urraca owns integration543 and resources309. The only owned temporary tree is O:/xterm-rust-swarm-16/tmp/worktrees/NefertitiCffFonts. Preserve source, regression cases and evidence before inspected cleanup. Do not remove other owners' paths, original references, credentials or AGENTS.md.
