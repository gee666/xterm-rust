# Native Kitty graphics

Task124 implements control parsing, streaming base64/chunk state, actions/replies, source-byte storage, raw RGB/RGBA preparation and placement arithmetic. Browser decoding/drawing, live terminal integration and user img-33 through img-48 remain tasks38/127 and are not claimed here.

## Code

- `image/kitty/command.rs`: all control fields, UTF-16 strings, absent/empty/NaN/signed-zero distinctions.
- `handler.rs`, `transmission.rs`, `finalize.rs`, `commands.rs`: real native protocol controller and fallible APC adapter. The required host starts actual asynchronous display work; the controller does not provide a fake rendering success.
- `storage.rs` and shared `image/number_map.rs`: ordered SameValueZero maps, live read-only views/iterators, source record aliasing, deletion callback ownership and reentrancy.
- `image/base64.rs`: strict deferred native decoder matching the original installed WASM dependency.
- `geometry.rs` and `pixels.rs`: source arithmetic stages and raw pixel conversion. Hosts execute actual bitmap/canvas operations between stages, using returned dimensions and fresh command reads.
- `image/encoded.rs`: immutable native memory or platform Blob-backed source handles. Factory capture precedes decoder release; asynchronous reads retain owned data.

## Tests run

```sh
cargo test -p xterm-addons --test kitty_protocol --test kitty_geometry --test kitty_storage --test kitty_pixels
cargo test -p xterm-addons --lib image::base64
cargo test -p xterm-addons --lib image::kitty::command
cargo clippy -p xterm-addons --lib --test kitty_protocol --test kitty_geometry --test kitty_storage --test kitty_pixels -- -D warnings
```

All16 native test functions and scoped Clippy passed on Windows. Comparisons cover23 original command cases and1411 complete-field inputs,270 decoder traces/7955 operations,235 protocol traces/7129 operations,4148 geometry cases,182 original Chromium ImageData cases and seven storage lifecycle tests. Three source mutants for quiet filtering, first-chunk metadata and the512-unit boundary were rejected.

Regenerate or check fixtures with `scripts/kitty-command-oracle.mjs`, `image-base64-oracle.mjs`, `kitty-protocol-oracle.mjs`, `kitty-geometry-oracle.mjs` and `kitty-pixels-oracle.mjs`. Normal execution checks committed expectations; `--generate` is only for reviewed source changes. Pixel regeneration launches real Chromium and closes its context/profile in `finally`.

The protocol comparison deliberately records a deferred display boundary. It verifies reply/state/future ordering, including reset/dispose completion and throwing listeners; it is not a substitute for original browser image assertions. Geometry tests execute original arithmetic blocks without pretending to implement platform WebIDL conversion. Pixel tests capture real Chromium ImageData before bitmap premultiplication, not final rendered canvas pixels. Raw invalid JS numeric decoder option timing remains task205. Full image equivalence is not proven.
