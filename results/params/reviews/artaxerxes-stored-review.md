# Independent Params stored-slot review

Task 182 adds the read-only `Params::stored_value` accessor used by SIXEL's existing source rule. I inspected the accessor, all three new Rust tests, source Params construction/clone/reset/resetZdm, and `source_repo/addons/addon-image/src/SixelHandler.ts:58`. The image consumer in `crates/xterm-addons/src/image/sixel/handler.rs` uses the same strict second-slot equality.

The source's normal parameter documentation warns that values past logical length are stale, but the actual SIXEL handler intentionally reads `params.params[1]` without a length check. The Rust accessor correctly uses the complete allocated backing vector, returns `Option<i32>` for bounds, exposes no mutation, and does not change the ordinary logical `values()` API. Rust clone retains backing slots, matching the source's full typed-array copy.

I independently executed the existing native Windows `params_stored-36c44ab06cfd7daf.exe` test artifact. All three tests passed, zero ignored. I did not rebuild it under the resource pause. Its exact executable hash and the current accessor/test hashes are retained in `artaxerxes-stored-source.json`.

I also independently executed the unchanged source Params implementation using Node's built-in TypeScript type erasure in an isolated VM. Fifteen stored-slot checks passed, covering initial/zero capacity, out-of-capacity reads, ZDM stale slot 1, overwrite, clone independence, empty logical reset, negative sentinel preservation and first-slot reset. No bundler, compiler or source mutation ran for this review. Node reported its experimental type-erasure warning; that warning was not suppressed.

Owner receipts separately retain broader Params source/differential and warning-free scoped Clippy results. This acceptance covers the small accessor and its source contract, not complete image decoding, actual parser/addon integration, browser behavior or a new release/native-platform matrix. Those remain separate tasks. Overall terminal equivalence remains NOT PROVEN.
