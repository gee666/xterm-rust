# X11 color parser port

`source_repo/src/common/input/XParseColor.ts` maps to `crates/xterm-core/src/input/xparse_color.rs`.

| Upstream | Rust | Contract |
| --- | --- | --- |
| `parseColor(string)` | `parse_color(&str) -> Option<[u8; 3]>` | `None` replaces `undefined`. Input is UTF-8 text. Accepted specs are ASCII, so UTF-16 indexing does not change results. |
| `toRgbString(color, bits = 16)` | `to_rgb_string([u8; 3], Option<u32>) -> String` | `None` selects 16 bits. Four, eight and twelve select those depths. Other representable depths select 16 bits. |

Rust requires valid 8-bit RGB channels through the `u8` type. JavaScript callers can pass other numbers to the formatter despite its intended RGB contract. Such out-of-domain calls are not represented by this typed API. Negative, fractional and non-finite JavaScript depths all select the 16-bit fallback and can map to `None` at a future binding boundary. Browser bindings and terminal OSC dispatch are not implemented by this module.

The parser distinguishes X11 hash truncation from `rgb:` scaling. For example, `#fff` gives `[240, 240, 240]`, while `rgb:f/f/f` gives `[255, 255, 255]`. Four-bit formatting preserves upstream's first-unpadded-digit behavior, so channel 15 formats as `f`, not `0`.

## Verification

Commands, run on native Windows x64 with Node v24.15.0 and rustc/cargo 1.85.1:

```sh
cargo test -p xterm-core --test xparse_color
node scripts/check-xparse-color.mjs
cargo clippy -p xterm-core --lib --test xparse_color --example xparse_color_oracle -- -D warnings
rustfmt --edition 2021 --check crates/xterm-core/src/input/xparse_color.rs crates/xterm-core/tests/xparse_color.rs crates/xterm-core/examples/xparse_color_oracle.rs
```

- Nine Rust tests passed. Seven faithfully port the original cases and 49 assertions. Two add malformed-input and low-channel formatting checks.
- All seven original TypeScript cases and 49 assertions passed through the documented Node assertion adapter.
- 153,872 Rust/JavaScript operation comparisons passed, including every possible channel value at each width, 2,048 formatter operations, and 12,000 randomized malformed strings. Seed is 1480806220.
- Focused Clippy passed with warnings denied. There is no unsafe code in this module or its adapter.
- Raw counts and source/workload/output hashes are in `results/xparse-color/summary.json`.

The script does not weaken the original assertions or edit the original source. It strips TypeScript syntax in memory and supplies synchronous `describe`/`it` callbacks and Node strict assertions. See `.pi/swarm/skills/xparse-color-check/SKILL.md` for the exact adapter scope.

Linux/macOS runtime checks and terminal/browser integration remain unrun. Full terminal equivalence is NOT PROVEN.
