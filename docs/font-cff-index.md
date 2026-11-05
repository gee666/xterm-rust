# Initial CFF INDEX constructor reads

Task472 extends the accepted457 header phase. The Rust terminal and addon do not run an OpenType JavaScript parser. The supplied opentype.js is a read-only test oracle, SHA256 `737212d8aae53d1996ef10b315e4369a1542ebb9b94c13c5f99793914745d484`. The actual integration baseline is main3a4f5e8f, including Harry463's FontError split and accepted442/447/453/457 union. Private prerequisite history is not a new source contribution to replay.

## Implemented contract

After existing directory/name/outline selection and CFF header validation, the constructor reads CFF1 Name, TopDict, String and GlobalSubr INDEX structures. CFF1 starts at the fixed fourth byte regardless of header.size. CFF2 reads its GlobalSubr INDEX at header.size + topDictLength before interpreting the dictionary. Counts are16-bit for CFF1 and32-bit for CFF2.

All INDEX offset bytes are bounds checked before object reads. JavaScript signed shift-and-add semantics apply even to offSize values outside1..4. Width0 produces empty objects without validating their location. Forward object ranges must fit the whole DataView; empty and reversed ranges perform no reads. SFNT and uncompressed WOFF retain the full font view, so signed offsets can address bytes preceding the selected table. Earlier table selection, duplicate precedence, callback-refreshed bytes and glyf/loca precedence remain in the integrated loader.

Compressed WOFF validates the entire deflate stream, zero-fills short output and discards overflow as the source does. The existing bounded prefix decoder now shares decoding with a range sink. One32KiB prefix is cached. INDEX reads beyond that prefix replay the stream with a32KiB backreference window and retain at most16 offsets, at most4080 bytes. Object byte reads need only bounds checks because these four phases do not interpret their contents. No allocation uses untrusted origLength and no arbitrary font acceptance limit was added. Replays trade CPU for bounded storage only for unusually distant offset tables. The history size is a DEFLATE protocol constant.

The range sink preserves overlapping copies, wrapped history, zero-filled short output and ignored overflow. Its native unit test compares stored, fixed and dynamic streams with70,000 expected patterned bytes over seven declared output limits and five captured ranges each. These compressed fixtures are generated reproducibly by the oracle script, not copied from an external font.

## Tests and commands

Use a private O: worktree, Cargo-j2 and a current resource grant. With the reference Node dependencies installed, run:

```sh
node scripts/font-cff-index-oracle.mjs --source=/path/to/source_repo
cargo test --locked -j2 -p xterm-addons --test font_cff_index
cargo test --locked -j2 -p xterm-addons --lib font_deflate::tests
cargo clippy --locked -j2 -p xterm-addons --lib --test font_cff_index -- -D warnings
cargo clippy --locked -j2 -p xterm-font-addons --target wasm32-unknown-unknown --test font_cff_index_browser -- -D warnings
SOURCE_PACKAGE_JSON=/path/to/source_repo/package.json node scripts/font-cff-index-browser-source.mjs
WASM_PACKAGE=xterm-font-addons WASM_TEST=font_cff_index_browser node scripts/browser-host-check.mjs
```

The source constructor corpus has476 recipes,56 accepted and420 RangeError bounds failures, seed0x472cff01. It covers all four initial indexes, truncation through count/width/offset/object bytes, signed and reversed ranges, widths0 and5..255, both directory tags, SFNT/WOFF, and eight CFF2 inputs whose count/width reads cross the cached prefix. Recipes preserve input SHA256 and FNV32 and expand independently in Rust and JavaScript. Original font bytes are not duplicated per case.

Browser tests replay the native corpus and520 accepted457 header cases, compare all476 independent byte reconstructions, and load four actual LigaturesAddon fonts asynchronously. They assert rejection, failed-font caching, fallback ranges, recovery, read/refresh counts, and the prior RangeError's name. Raw source and Rust debug error receipts are retained separately. Results under `results/font-cff-index/` distinguish source execution, native Rust and actual WASM execution. Initial logs before the bounded-storage revision are historical, not final-source receipts.

## Explicit remaining gaps

This is not complete CFF parsing. CFF dictionaries, private dictionaries, FDArray/FDSelect, charstrings, charset, encoding, variation stores, glyph behavior and later constructor effects remain task342. Initial INDEX object bytes are checked but not interpreted here. The source's enormous count/width0 allocations are not recreated simply to allocate empty arrays; eventual dictionary count semantics remain342. Inputs known to make the original inflater loop forever retain the established safe termination divergence.

The browser adapter still maps FontError::Bounds to RangeError with the Rust message `Font table offset is outside the input`. Original engines use their own DataView wording. The new addon test deliberately asserts this existing discrepancy and retains raw messages; it does not claim message equality or silently normalize a complete-equivalence assertion. Task342 owns exact error-message parity. No original upstream assertion was changed.

Full workspace formatting on this Windows path may still hit OS206. Member formatting is separate evidence, not a claimed full-workspace pass. This task does not establish native Linux/macOS results, complete unchanged upstream suites, all font functionality, or overall port equivalence. Overall equivalence remains NOT PROVEN.
