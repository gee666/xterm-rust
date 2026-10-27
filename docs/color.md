# Color utilities

`crates/xterm-core/src/color/` ports `source_repo/src/common/Color.ts`. It uses safe Rust and the standard library. Packed RGBA values are `u32` in `0xrrggbbaa` order, never native-endian pixel words.

| Upstream | Rust |
| --- | --- |
| `IColor`, `IColorRGB` | `Color`, `[u8; 3]` |
| `NULL_COLOR` | `Color::null()` |
| `channels.toCss/toRgba/toColor` | `channels::to_css/to_rgba/to_color` |
| `color.blend/isOpaque/ensureContrastRatio/opaque/opacity/multiplyOpacity/toColorRGB` | same snake-case functions in `value` |
| `css.toColor` | `css::to_color` for headless formats; `css::to_color_with_canvas` for browser fallback |
| `rgb.relativeLuminance/relativeLuminance2` | `rgb::relative_luminance/relative_luminance_channels` |
| `rgba.blend/ensureContrastRatio/reduceLuminance/increaseLuminance/toChannels` | same snake-case functions in `rgba` |
| `toPaddedHex`, `contrastRatio` | `to_padded_hex`, `contrast_ratio` |

The Rust `value` namespace avoids the redundant `color::color` module name. Absent optional alpha arguments become `None`; absent adjusted colors become `Option::None`. CSS parse failures return `UnsupportedCss` with upstream's error text. Channel arguments are `i64`, not bytes, because the source accepts CSS channel values above 255 and bitwise-packs them without clamping. Packed inputs use `u32`. RGB luminance-channel arguments use bytes. These typed internal interfaces do not accept arbitrary JavaScript objects or fractional channel values.

CSS strings retain original spelling where upstream does. The parser deliberately preserves the source's unanchored regular-expression matching and overflow behavior. Alpha blending rounds towards positive infinity on ties. Opacity preserves out-of-range, nonfinite, negative and very large JavaScript-number results, including the hexadecimal CSS spelling. Contrast adjustment moves ten percent towards black or white per iteration, matching the source rather than substituting another contrast algorithm.

The browser fallback is an explicit callback that validates the supplied fillStyle and returns the resulting pixel. It rejects nonopaque fallback pixels as upstream does. This core module cannot query a DOM canvas. A working browser adapter and browser-color tests are still required by the browser tasks; this utility check is not evidence that those contracts already work.

## Reproduction

Run from the repository root with Node 24 and Rust 1.85 or later on PATH:

```text
npm install --prefix tmp/color-oracle --no-audit --no-fund --ignore-scripts chai@6.2.2 mocha@11.7.5
node tools/generate_color_tests.mjs
rustfmt --edition 2021 crates/xterm-core/tests/color_upstream.rs
cargo test -p xterm-core --test color_upstream --test color_edges
node tools/check_color.mjs
```

The generator translates all 350 original assertions into 28 Rust tests in source order, with source-line comments and unchanged expected values. It rejects unknown syntax and verifies that the assertion count matches the reference. Its generated fixture exceeds the usual 350-line limit so the full source assertion sequence remains auditable. Formatting and hexadecimal literal case are language-specific changes, not changed expectations.

The original baseline runner uses actual Mocha 11.7.5 and Chai 6.2.2. Node's built-in TypeScript transform removes types and namespace syntax. Temporary generated modules remove one residual type-only import and resolve imports to the isolated dependencies and the transformed original Color module. Assertion bodies are not rewritten or omitted. The original files stay unchanged. Installation reported a transitive deprecated glob package from Mocha; these packages are test-only, not Rust runtime dependencies.

`tools/check_color.mjs` compares seven operations per iteration for 4,096 xorshift32 iterations at seed `0xc010a123`: blend, ensure contrast, reduce luminance, increase luminance, four-decimal relative luminance, opacity and multiplied opacity. It adds 24 exceptional opacity operations and 37 valid/malformed CSS inputs. The 28,733 comparisons use the original JavaScript functions as the oracle and a Rust executable implementing the same line protocol. This is deterministic differential utility coverage, not full terminal-state coverage. Four additional Rust tests exercise the callback boundary, optional results, malformed CSS matching, and wide/nonfinite opacity.

## Observed evidence

`results/color/original.json` records source hashes, runtime and dependency versions, 28 discovered/executed/passed cases, 350 discovered assertions, zero failed and zero skipped cases. `results/color/differential.json` records 28,733 compared operations and zero current mismatches on native Windows. `results/color/rust-tests.log`, `results/color/quality.log`, and `results/color/environment.txt` retain focused command output and versions.

`results/color/counterexamples.json` retains a discovered and fixed parser discrepancy. `x#abc` returned `#00aabbcc` in the first Rust implementation; the source returns `#NaNaabbcc` while packing the same numeric RGBA. The parser and a named regression assertion now preserve that behavior. The retained historical counterexample is not a current failing result.

Linux and macOS execution, browser canvas/color integration, and integration with terminal theme/rendering consumers remain unverified here. Overall xterm equivalence is NOT PROVEN.
