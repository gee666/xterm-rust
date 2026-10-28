# Native QOI decoder

`image/qoi/` ports the installed xterm-wasm-parts0.4.1 decoder in safe Rust, without another decoder dependency. It handles INDEX/DIFF/LUMA/RUN/RGB/RGBA, signed header dimensions, exact page allocation, encoded/output overlap and retained destination memory. Release preserves the source's page threshold and dimensions. Native callers receive owned pixel snapshots; the IIP integration must still create actual ImageData/bitmaps.

The reference is the installed WASM codec. The upstream C source was also read for implementation, but no public revision is claimed for the supplied package. The oracle found and fixed the zero-page import failure category: it is LinkError, not RuntimeError. Negative initial pages remain TypeError. Errors are recoverable Rust values with source categories.

Checks:

```sh
cargo test --locked -p xterm-addons --test qoi
node scripts/qoi-oracle.mjs
cargo clippy --locked -p xterm-addons --test qoi --example qoi_trace -- -D warnings
```

Four Rust tests and1,325 original-WASM observations pass. The corpus includes every opcode,500 seeded malformed inputs, fresh/reused memory, truncated data, overlap/traps, release boundaries, dice.qoi and the original640x80 palette. Native palette output equals all204,800 supplied RGBA bytes. Browser ImageData, resizing and IIP fast-path wiring remain tasks38/127; this codec is not a live-browser graphics pass.
