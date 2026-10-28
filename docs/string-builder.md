# String builders

`crates/xterm-core/src/string_builder.rs` ports both classes in
`source_repo/src/common/StringBuilder.ts`. Copyright and MIT attribution are
retained. No dependency or platform adapter is needed.

## Binding contract

| Upstream | Rust |
| --- | --- |
| `new StringBuilder()` | `StringBuilder::new()` or `default()` |
| `new LimitedStringBuilder(limit)` | `new(limit: usize)` for native bounds; `with_js_limit(limit: f64)` for all JS Number values |
| `length` | `len()` counts UTF-16 code units |
| `limit` | `limit()` returns `PayloadLimit::Units(usize)` or `PayloadLimit::JavaScript(f64)` without losing metadata |
| `append(string)` | `append(&str)` for valid Unicode, `append_utf16(&[u16])` for arbitrary JS strings |
| `reset()` | `reset()` |
| `toString()` | `as_utf16()` borrows exact JS code units; `to_utf8()` returns `Result<String, FromUtf16Error>` |

`PayloadLimit` distinguishes exact native unit bounds from JavaScript Number
bounds. `new(usize)` retains integer overflow-safe comparison even at usize::MAX.
`with_js_limit(f64)` preserves fractions, negative values, NaN, infinities and
negative zero. It applies the source `length > limit` comparison. Thus a negative
limit rejects even an empty append, NaN never rejects, and equality succeeds.
The enum returned by `limit()` makes that representation explicit to callers.

Lone surrogates survive append, observation and reset. A surrogate pair may be
split across chunks. UTF-8 conversion returns an error for an unpaired surrogate
rather than replacing it. Callers needing JavaScript string semantics must use
the UTF-16 API, not a lossy conversion.

Contiguous UTF-16 storage grows amortized linearly instead of retaining JS
string chunks. This avoids quadratic concatenation while exposing borrowed
payloads to parser callbacks. Limited appends preflight the UTF-16 unit count,
then clear on strict overflow without allocating a rejected chunk. The
observable result matches upstream's append-then-clear operation. A limit breach
does not poison the builder. An empty append at the limit succeeds.

## Verification

Run from the repository root with Node 24 and Rust 1.85 or newer:

```sh
node --experimental-transform-types scripts/string-builder-oracle.mjs
cargo test -p xterm-core --test string_builder_upstream --test string_builder_differential --test string_builder_edges
rustfmt --edition 2021 --check crates/xterm-core/src/string_builder.rs crates/xterm-core/tests/string_builder_upstream.rs crates/xterm-core/tests/string_builder_differential.rs
cargo clippy -p xterm-core --lib --test string_builder_upstream --test string_builder_differential --all-features -- -D warnings
```

The Node harness imports the actual reference implementation and runs a temporary
copy of the upstream test file under `tmp/`. Only module and assertion-library
imports change. Test bodies and assertions remain unchanged. Native Node
TypeScript transformation is required because the limited builder constructor
uses a parameter property.

All 17 upstream cases map one-to-one to tests in
`tests/string_builder_upstream.rs`, in source order. The seven unbounded cases
cover empty state, a single chunk, ordered joins, empty chunks, reset, reuse and
10,000 small chunks. The ten limited cases cover the configured limit, empty
state, cumulative equality, single-chunk equality, cumulative overflow,
single-chunk overflow, reuse after breach and reset, 100 chunks to the limit,
zero-limit overflow and zero-limit empty appends. Assertions retain the original
values and expected results.

`tests/string_builder_differential.rs` adds four tests. One checks 278 traces
containing 16,648 operations, generated with xorshift32 seed `0x51b01d`. Every
operation compares overflow status, exact UTF-16 length and all output units.
Eight directed traces cover zero/equality limits, NUL, split pairs, isolated
surrogates, empty appends and resets. Another 256 seeded traces mix reset and
arbitrary UTF-16 appends with unlimited and 0..64-unit limits. Three focused tests
cover UTF-8 helper counts, error handling and maximal representable Rust limits.
Fourteen additional directed traces cover all JavaScript numeric limit classes,
with exact configured metadata tested separately. The checked-in TSV records
inputs and full expected states, so any mismatch identifies a retained
counterexample by row number. The Node oracle also checks all 65,536 single code
units. `tests/string_builder_edges.rs` adds six tests for those units, every
partition of six surrogate/combining sequences, native boundary matrices,
repeated overflow/recovery, numeric metadata and independent owned snapshots.

`results/string-builder/oracle.json` records exact reference/fixture SHA-256
hashes, seed, platform, Node version, all upstream names and baseline counts.
Rust logs and command status are retained alongside it. Local execution uses
Windows x64, Node v24.15.0 and rustc 1.85.1. These checks prove only the stated
utility behavior for the executed cases. Native Linux/macOS, browser and complete
terminal equivalence remain unproven.
