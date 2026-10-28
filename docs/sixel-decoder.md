# Native SIXEL decoder work

Task125 implements a safe Rust decoder and source-handler policy. Full image/terminal/browser equivalence remains NOT PROVEN. Seven initial directed Rust tests passed. The expanded decoder differential and storage-controller tests are written but need a guarded native run. Do not infer a pass from fixture generation.

## Reference and implementation

The behavioral oracle is the installed `source_repo/node_modules/sixel` package, version0.16.0. Its MIT license by Joerg Breitbart is retained in `licenses/sixel-MIT.txt`. The package's Decoder.js drives a static C/WASM band decoder. The matching repository tag0.16.0 was observed to dereference commit `35f542f8b1b9a8867fab133aea5d43cb3da6108a`. Read-only C++/build references were fetched from that commit into `tmp/sixel-reference/`; production Rust never calls that C++ or WASM decoder.

No Rust dependency was added. `crates/xterm-addons/src/image/sixel/` separates public contracts, parser, canvas assembly, color math, background extraction, protocol lifecycle and storage policy. It uses checked Vec allocation, explicit integer wrapping and explicit RGBA serialization. There is no unsafe code or process-global mutable decoder state.

| Reference | Rust |
| --- | --- |
| sixel0.16.0 `wasm/decoder.cpp` | `parser.rs`, `colors.rs`, decoder state in `mod.rs` |
| sixel0.16.0 `lib/Decoder.js` | `canvas.rs`, `types.rs`, decoder methods in `mod.rs` |
| image `SixelHandler.ts` | `handler.rs`, `background.rs` |
| image `SixelImageStorage.ts` | `storage.rs` functions over actual shared ImageStorage and fallible linefeed effects |
| original palettes | checked native constant construction in `colors.rs` |

M0/M1/M2 decision, complete-current-chunk raster replay, eight signed wrapping parameters, stale parameter slots, seven-bit byte masking, repeat overflow, clipping, partial bands and raster truncation follow the actual implementation rather than a newer SIXEL specification. Color conversion retains C f32 HLS arithmetic and its original ternary-precedence/range quirks. The decoded raster properties expose the unsigned state view used by JavaScript, including signed-C values reinterpreted through Uint32Array.

Memory and palette limits retain f64 boundary comparisons and ToUint32 behavior for negative, fractional, NaN and infinite inputs. Canvas growth follows the source's65536-pixel blocks and its unchecked initial legacy buffer policy. Reported decoder memory is the original compatibility metric, including seven fixed WASM pages, not a measurement of native process RSS. It is used to preserve the source's4MiB retained-memory release decision. Actual performance/memory comparisons must measure real native allocations/process memory separately.

## Handler and storage integration

`SixelHandler::pending` models input discarded before decoder readiness; completion installs the actual native decoder without replaying skipped hook/input. Native ready construction is also available. Hook reads `Params::stored_value(1)`, not the active parameter slice, because the original source reads the backing P2 slot even after Params reset. Task182 supplies that accessor and criterion419 tracks the consumer regression.

Warnings are two-phase. `put` returns a typed warning before setting abort or releasing, then the host logs it and calls `finish_abort`. This preserves source ordering when a warning callback throws or reenters. End planning, pixel capture and post-copy release are separate so browser code can create a real canvas, skip capture on a missing context, release handler borrows across callbacks, and insert into real storage only afterward. The native `end` convenience method performs actual decoding and returns owned RGBA pixels or the source height-only cursor effect.

`active_background` preserves missing-theme transparent fill and default/indexed/RGB/inverse branches. `add_sixel_image` supplies scrolling/top/z0/VT340 placement to the real shared storage. `advance_sixel_cursor` preserves lazy metrics access, exact -1 fallback to7x14, floating-point ceil and fallible linefeed order. The actual browser renderer normalizes falsy cell metrics before that wrapper, as the source does.

Task38 still owns final parser registration, async host initialization, real image storage/renderer routing, graphics queries and actual native/browser addon loading. The decoder module does not claim those paths are already integrated.

## Verification

Use the controlled shared queues in `docs/build-slots.md`; do not bypass the1GiB reserve or create alternate Cargo targets.

```sh
node scripts/sixel-oracle.mjs
node scripts/build-slot.mjs --owner Reviewer-125 --lane native -- cargo test -p xterm-addons --lib image::sixel
```

`--generate` is required to regenerate the source fixture deliberately. Normal oracle verification fails on fixture drift. The script executes the unmodified installed decoder and retains package/WASM/source/fixture hashes under `results/sixel/oracle.json`. Its seed is `0x516e1`. Current corpus has676 cases and7015 observed snapshots, including the full palette.sixel picture at bytewise/17/16384-unit chunking. All4096 palette slots are reconstructed from a source initial basis plus source deltas, and every decoded pixel is compared as run-length encoded ABGR words. Before/after properties retain side effects of observing source pixel buffers. Original error phase/name/message are retained; the Rust test currently checks exact records and must retain any mismatch rather than weaken expectations.

The fixture is generated source-derived data, intentionally longer than350 lines. It is not a Rust implementation table. The first directed run passed7 tests in receipt `results/build-slots/1793150018287-11592-fffff29b-df56-4b5c-98ba-5a1865a192e5/`. New storage and differential cases remain unrun until a successful guarded receipt records them. Full release, Clippy, platform and real renderer/terminal/browser evidence remain separate outstanding checks.
