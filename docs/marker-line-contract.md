# Planned portable marker line contract

This is the shared contract for task 221 and its Buffer, decoration, collection and WASM consumers. The scalar API is implemented by task 227; Marker wiring below remains planned. Seven Rust tests and 5,617 JS operator cases pass. This does not establish raw browser bindings. `docs/acceptance/marker-js-boundary.md` defines the validation and operator cases.

## Scalar

`buffer::marker_line::MarkerLine` is an owned Number/String sum value. It reuses the existing portable `options::OptionValue` number parsing and addition rather than copy a JavaScript coercion implementation. Construction accepts f64 and lossless UTF-16 strings only. Conversion from a general OptionValue validates that restriction. A host object is coerced at its source evaluation stage before constructing the stored line.

Implemented scalar operations:

```text
MarkerLine::number(f64)
MarkerLine::string(Vec<u16>)
MarkerLine::try_from(OptionValue) -> Result<MarkerLine, MarkerLineError>
as_value() -> &OptionValue              // always Number or String
into_value() -> OptionValue
as_number() -> Option<f64>              // raw type inspection, no coercion
as_string() -> Option<&[u16]>
to_number() -> f64                      // ECMAScript primitive Number conversion
to_utf16() -> Vec<u16>
strict_eq_number(f64) -> bool
strict_eq(&MarkerLine) -> bool
rel_lt/rel_gt/rel_le/rel_ge(&MarkerLine) -> bool
add_number(f64) -> Result<MarkerLine, MarkerLineError>
add_value(&OptionValue) -> Result<MarkerLine, MarkerLineError>
subtract_number(f64) -> MarkerLine
increment() -> MarkerLine
map_key() -> MarkerLineKey
```

Relational comparison is UTF-16 lexicographic when both values are strings and numeric otherwise. Do not implement `>=` as negated `<`, because NaN makes both false. Strict equality preserves the type distinction and makes NaN unequal to itself. Addition can concatenate; subtraction and increment always produce a Number.

MarkerLineKey provides Eq/Hash under JavaScript Map SameValueZero: all NaNs share a numeric key, positive/negative zero share a numeric key, and String keys remain distinct from Number keys. It does not imply Map iteration order. Consumers must preserve source insertion order where collisions and merging can expose it.

SortedList needs the same distinction: sort subtraction is numeric, binary-search `<`/`>` can be string-lexicographic, and duplicate checks are strict equality. A universal f64 key projection is not equivalent. Chloe owns that collection adaptation.

## Pending Marker integration

The authoritative State field becomes one MarkerLine, not an integer field plus a JavaScript shadow. The integration window replaces old ambiguous `line()` reads with these explicit APIs:

```text
Marker::new(impl Into<MarkerLine>)
line_value() -> MarkerLine
line_number() -> f64
set_line(impl Into<MarkerLine>)
add_to_line(f64) -> Result<(), MarkerLineError>
subtract_from_line(f64)
```

Native integer convenience conversions model JavaScript Number conversion; valid native buffer rows remain exactly representable. `line_number` is only a computed projection. Public JS getters use `line_value` and preserve its raw type. Clearing a numeric row and OSC link dedup use strict equality, not the projection. Native comparisons against numeric event indices can use the numeric projection. Insert mutation must use add_to_line, never project then add numerically.

Convert constructor/setter arguments before borrowing line state or taking the id lock. Release every line/state borrow before callbacks or host coercion. Disposal publishes Number(-1) before firing. Existing Rc/Weak identity and explicit fallible disposal remain unchanged.

Browser public registerMarker first runs the exact `_verifyIntegers` checks, without a proposed-API gate. Preserve accepted raw arguments and repeated Number-hint conversions. The later addition uses the original value and default-hint semantics. Capture the core addMarker receiver before evaluating arguments; a coercion callback can switch/reset buffers without redirecting that already captured call.
