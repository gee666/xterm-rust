# Native QOI decoder

`image/qoi/` ports the installed xterm-wasm-parts0.4.1 decoder in safe Rust, without another decoder dependency. It handles INDEX/DIFF/LUMA/RUN/RGB/RGBA, signed header dimensions, exact page allocation, encoded/output overlap and retained destination memory. Release preserves the source's page threshold and dimensions. Native callers receive owned pixel snapshots. The browser IIP decoder now creates actual ImageData, canvases and bitmaps from these pixels, as described in `docs/iip-browser-decode.md`. Full ImageAddon activation remains open.

The reference is the installed WASM codec. The upstream C source was also read for implementation, but no public revision is claimed for the supplied package. The oracle found and fixed the zero-page import failure category: it is LinkError, not RuntimeError. Negative initial pages remain TypeError. Review also found that oversized initial/growth page arguments fail unsigned-long conversion with TypeError before the separate WASM page-limit RangeError; regressions cover both decoder states. Errors are recoverable Rust values with source categories.

Checks:

```sh
cargo test --locked -p xterm-addons --test qoi
node scripts/qoi-oracle.mjs
cargo clippy --locked -p xterm-addons --test qoi --example qoi_trace -- -D warnings
```

Six Rust tests and 1,329 original-WASM observations pass. The added in-place host-view test verifies retained pixel mutations without an extra owned-image allocation. The corpus includes every opcode,500 seeded malformed inputs, fresh/reused memory, truncated data, overlap/traps, release boundaries, dice.qoi and the original640x80 palette. Native palette output equals all 204,800 supplied RGBA bytes. The recovered codec passes the same checks on current main's producer branch. Browser ImageData, resizing and the IIP fast path now have separate real-browser component evidence in `docs/iip-browser-decode.md`. This codec replay alone is not a live-browser graphics pass.
