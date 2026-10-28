# Native IIP transfer policy

Task156 implements OSC1337 header/payload state, multipart recovery, sizing and replies in `crates/xterm-addons/src/image/iip/`. Actual image codecs, terminal activation and browser rendering remain tasks38/195/127.

- `prepare_end` returns early for rejected/control-only paths or an owned geometry token. A host reads fallible geometry without holding a controller borrow, then calls `finish_geometry`. Reentrant reset and getter failure preserve the source's partial state.
- The native handler uses the strict Rust payload decoder and HeaderParser. Size metadata is informative, not an extra payload limit. Reset retains the source multipart/header quirks.
- Sizing preserves source width precedence, cell/percent/pixel units, viewport-minus-one-row auto fit, no upscale, flooring and strict pixel bounds. Shared numeric helpers preserve parseInt and exact toFixed0/3 rounding.
- `DisplayRequest` is an explicit host handoff, not a rendered image. Browser code must consume its bytes into the real Blob/codec before awaiting. QOI synchronous errors and bitmap promise rejection have different source phases; task195 supplies actual QOI pixels/ownership.
- `IipImageStorage` forwards real shared storage with scrolling, top layer, z-index0 and IIP cursor placement.

## Checks

```sh
cargo test -p xterm-addons --test image_iip
node scripts/image-iip-oracle.mjs
cargo clippy -p xterm-addons --test image_iip --example image_iip_trace -- -D warnings
```

Results:10 Rust tests pass;9,925 comparisons with unchanged source IIPHandler pass, seed322379814. Scoped Clippy passes. The oracle uses the actual source base64 decoder and recording bitmap/storage hosts, comparing state, bytes, replies, warnings, geometry, fixed-point output and reset/throw during geometry. Those hosts do not prove pixels or live browser integration. Summary is `results/image-iip/summary.json`.

The reference-only mode writes an explicitly unrun Rust baseline; it is not the current passing comparison. Raw JS decoder numeric option boundaries remain task205. User img01..48 still require actual Rust-backed browser execution.
