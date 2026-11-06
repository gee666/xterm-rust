# ANSI raw marker allocation coercion

Task 549 repairs the allocation phase of browser ANSI `SerializeAddon.serialize`, within the still-open tasks 346 and 322. Full equivalence is NOT PROVEN.

The original `_serializeBufferByRange` reads `range.start` twice and its marker's `line`, captures columns, then reads `range.end` twice and its marker's `line`. `BaseSerializeHandler.serialize` evaluates `endRow - startRow` before calling `StringSerializeHandler._beforeSerialize`. That subtraction evaluates the end coordinate first, then the start coordinate, both with the number hint. Exceptions from either operand stop serialization before later option getters. A numeric invalid difference reaches the browser's Array constructor and throws its actual RangeError.

Current main 0442 validates only primitive-number differences, then converts the start index before the end. For raw object marker lines, this skips the source subtraction and changes conversion order, error precedence, reentrant state changes and diagnostics. The scoped repair must perform that subtraction before native index conversion. A small caught JavaScript operator is host-language glue, not a serializer or a JavaScript terminal engine. It stays beside this adapter's existing invalid-array-length bridge rather than making serialization depend on scrolling internals.

The shared `scripts/serialize-browser-cases.mjs` adds three cases with 35 subcases. They cover boxed start/end/both invalid differences, nonfinite/fractional/negative/oversized differences, Symbol.toPrimitive getter/call exceptions and exact thrown identity, Symbol/nonprimitive/mixed-BigInt results, reset/resize, nested serialization and end coercion mutating the start operand. Existing cases and source assertions remain intact. The complete runner expects 93 cases, including the unchanged 32 seeded traces with seed `0x05e21a11`.

## Scope limits

Valid arbitrary-object row traversal is still incomplete. Source comparisons, raw ring lookup, strict endpoint equality and row increment can each coerce the original coordinate again. This task does not replace those original values with a supposedly equivalent cached number. A successful subtraction does not prove valid raw coordinates are supported. Pure-BigInt difference traversal and arbitrary replaced host/public-buffer methods remain open. Equal infinite source bounds can hang and are not run or claimed compatible. No new dependency, unsafe code or source assertion changes are needed.

## Verification

Pending actual negative-control and fixed execution. No results are claimed by this file before raw receipts exist. Run the existing `scripts/build-demo.mjs --debug` and `scripts/serialize-browser-check.mjs --suite` from an owned O: worktree with explicit team resource grants, Cargo -j2 and literal O: TEMP/TMP/TMPDIR. The source bundle and original tests must retain their content hashes. The debug browser distribution is for functionality, not performance comparison.
