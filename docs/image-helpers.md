# Image header and metadata helpers

Task123 implements `IIPHeaderParser.ts` and `IIPMetrics.ts` under `crates/xterm-addons/src/image/`. These utilities do not claim to decode, store or render live images. Parent38 and User criteria280-327 still require actual Rust-backed WASM/three-browser image execution.

## Contracts

Header state, partial fields, backing-buffer content and source return codes are preserved. `ParseOutcome::{NeedMore,Rejected,Payload}` represent source -2/-1/absolute offset. Rust validates slice ranges before mutation. Field keys and strings use UTF16 vectors; unknown field data remains u32 vectors. Numeric fields use binary64, including source accumulation rounding and overflow. Field ordering follows JavaScript integer keys before other insertion order. Prototype-sensitive decoder keys have explicit outcomes, including __proto__ abort/null-prototype state and inherited toString/valueOf behavior. These cases are not falsely treated as plain safe-map behavior.

`HeaderParser::new(NameDecodePolicy)` deliberately separates Node Buffer and browser atob name policies. Buffer tolerates URL alphabet/junk and truncates UCS2 units to bytes before decoding. Browser atob rejects nonalphabet nonbyte characters, while UTF8 TextDecoder consumes an initial BOM. Payload base64 is a separate strict streaming decoder owned by task124, not this filename decoder.

Metadata sniffing recognizes PNG/JPEG/GIF/QOI/WebP/AVIF, including all three WebP variants. PNG and QOI dimensions retain signed32 bitwise behavior. AVIF scanning preserves the strict upper bound and positive-dimension requirement. JPEG truncated typed-array bitwise reads become zero rather than indexing unchecked. Rust byte slices do not impose JS Uint32Array alignment; original unaligned-offset exceptions belong at the browser binding boundary. `RgbaImage` validates actual decoded raster byte length and checked multiplication before allocation ownership, and permits zero-size SIXEL cursor-only outcomes.

## Original assertions and Rust replay

`node scripts/image-helpers-oracle.mjs` compiles the unchanged source modules and the complete original test bodies in private VM realms with TypeScript5.9.3. The real Chai assertions execute. Module imports point to instrumented source classes only to record operations and post-state; test bodies and expected assertions are not rewritten.

The original 15 header tests and one metrics test execute 134 assertions. Thirty-eight original image fixtures are read intact. The original tests produce 539 operations recorded in `tests/fixtures/image-helpers-upstream.json`. The generated fixture is one cohesive machine-readable upstream replay and intentionally exceeds the ordinary line-width limit. The Rust replay test compares every complete snapshot against those recorded values and confirms all16 source case names. It is one Rust test containing539 snapshot checks, not539 migrated tests.

Six added Rust tests cover Node/browser name-policy divergence, retained failing seed, BOM/padding/empty behavior,1024/1025 fields, the non-aborting START guard, prototype keys/copy/reset, validated range atomicity, checked pixel length/overflow and signed/AVIF boundaries.

The seeded oracle compares 17,483 outputs with0 mismatches. It includes all single split points of directed headers shorter than120units, selected field-capacity splits, the original bytewise assertions,1,000 randomized malformed/streamed headers and1,000 randomized metadata inputs. Seed443424806, or0x1a6e2026. It compares state, partial fields, exact numeric bits, UTF16 keys/text, unknown arrays, source positions, stale backing-buffer contents, null-prototype state, MIME and dimensions. Four deliberately incorrect in-memory source mutants are rejected: capacity1024→1023, decimal radix10→9, END→ABORT and PNG high-byte shift24→16. Source files remain untouched.

## Bugs retained as evidence

The first harness attempt compared VM object prototypes against the transport JSON object's realm, creating false metrics mismatches. Only transport realm ownership was normalized; actual field types, byte values and source prototype behavior remain explicit in snapshots. The failed receipt is `results/image-helpers/harness-realm-counterexample.json`.

The corrected harness found a real Rust bug with Node Buffer filename decoding. Non-ASCII UTF16 units whose low bytes were valid base64 digits were initially ignored instead of truncated. Seed443424806 produced filename byte106 while Rust returned empty. `node-buffer-ucs2-counterexample.json` retains this case; the implementation was corrected and a directed Rust regression now checks Node acceptance versus browser rejection. The historical `counterexample.json` remains as a retained failure, not current status. Current `summary.json` reports0 mismatches.

## Commands and observed results

Native Windows x64, Node24.15.0, TypeScript5.9.3, rustc/cargo1.85.1. The shared disk-saving debug profile was preserved, and all checks used the existing target directory.

```text
node scripts/image-helpers-oracle.mjs
cargo test -p xterm-addons --test image_helpers_upstream --test image_helper_edges
cargo clippy -p xterm-addons --test image_helpers_upstream --test image_helper_edges --example image_helpers_trace -- -D warnings
rustfmt --edition 2021 --check crates/xterm-addons/src/image/header.rs crates/xterm-addons/src/image/header_codec.rs crates/xterm-addons/src/image/metrics.rs crates/xterm-addons/src/image/types.rs crates/xterm-addons/tests/image_helper_edges.rs crates/xterm-addons/tests/image_helpers_upstream.rs crates/xterm-addons/tests/image_helper_support/mod.rs crates/xterm-addons/examples/image_helpers_trace.rs
```

Oracle,7 Rust tests and scoped Clippy pass. Logs, full replay operations, source/fixture/implementation hashes, counts and mutant results are under `results/image-helpers/`. Earlier shared core resize/SGR compilation failures were integration churn, not bypassed by a duplicate build or disabled tests. Source fixture files under `source_repo/addons/addon-image/fixture/testimages` are required for the38 metadata replay operations.

The final scoped run left P drive near the shared1GiB reserve; further builds were paused and the quality owner notified. Native Linux/macOS, actual browser alignment/name decoding, IIP payload decoding, SIXEL/Kitty protocols, shared image storage, rendering and the exact User img01..48 acceptance commands remain separate unproved obligations. Helper parity is not complete image addon equivalence.
