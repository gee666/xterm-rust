# Legacy keyboard policy

`crates/xterm-core/src/input/keyboard.rs` ports `source_repo/src/common/input/Keyboard.ts`. It evaluates an event without DOM access and returns the outgoing UTF-16 sequence, cancellation request, and send/select/page action. Native and browser adapters can share this policy. This module does not install browser event handlers or prove browser integration.

## API mapping

| Upstream | Rust |
| --- | --- |
| `evaluateKeyboardEvent` | `keyboard::evaluate_keyboard_event` |
| `IKeyboardEvent.keyCode` | `KeyboardEvent.key_code: u32` |
| `IKeyboardEvent.key`, `.code` | borrowed UTF-16 slices, including unpaired surrogates |
| `shiftKey`, `altKey`, `ctrlKey`, `metaKey` | corresponding snake-case boolean fields |
| `IKeyboardEvent.type` | omitted, upstream evaluator never reads it |
| three positional option booleans | same order, application cursor mode, Mac platform, Mac Option-as-Meta |
| result `.key` | `Option<Vec<u16>>`, preserving undefined separately from empty text |
| result `.type` | `KeyboardResultType`, original numeric discriminants 0 through 3 |
| result `.cancel` | `bool`, request for the caller to cancel the event |

Key codes follow the DOM unsigned-long range. Browser bindings must apply WebIDL conversion before calling this Rust API. The policy preserves upstream modifier priority, Mac Option handling, mobile arrow names, iOS Ctrl+C, insert copy/paste exceptions, application cursor mode, function keys, control punctuation and cancellation distinctions.

`keyboard::KeyboardEvent`, `KeyboardResult` and `KeyboardResultType` are also shared by the Win32 encoder. Result payloads stay UTF-16, rather than replacing malformed units during a String conversion.

## Lowercase data

The dead-key branch takes `code.slice(3, 4)` in JS, then lowercases that one UTF-16 unit. A scalar-based substring would incorrectly emit a complete non-BMP character. The Rust code selects one unit, preserves surrogates and supports the multi-unit lowercase mapping for U+0130.

The original JavaScript delegates lowercase data to its runtime. The checked-in table pins Unicode 17.0 from the observed Node 24.15.0 oracle, ICU 78.2. It does not inherit whichever Unicode version a Rust compiler uses. `keyboard_lowercase.rs` is generated cohesive protocol-support data and exceeds the 350-line limit to retain all mappings. The generator checks every BMP unit. Unicode data attribution is in `licenses/UNICODE-3.0.txt`, retrieved from https://www.unicode.org/license.txt. Node versions using another Unicode release are explicitly rejected by this verifier, not silently treated as equivalent. Actual browser engines and event wiring remain unverified.

The first differential run found a real mismatch. Rust 1.85.1 left U+A7CE unchanged, while the JS oracle lowercased it to U+A7CF. `results/keyboard/counterexample.json` retains that input. The pinned table fixes it, and an ordinary Rust regression assertion covers it.

## Verification

Run from repository root with Rust on PATH and Node supporting `stripTypeScriptTypes` and Unicode 17.0:

```sh
node scripts/verify-keyboard.mjs
cargo test -p xterm-core --test keyboard_upstream --test keyboard_edges
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

The verifier uses the existing Win32 oracle method. It erases TypeScript annotations and removes import/export declarations, but retains every original Keyboard.test.ts body and assertion. It loads the original result enum and escape constants. It captures each successful upstream assertion's full event, options and result into `tests/fixtures/keyboard_upstream.tsv`; `--generate` refreshes that fixture only when requested. Ordinary verification fails on drift. Rust checks the original payload expectation plus cancellation and action for all fixture rows.

Observed Windows x86-64 results with rustc 1.85.1 and Node 24.15.0:

- 61 original tests, 156 assertions discovered, executed and passed. No skipped or unrun tests in this suite.
- 156 migrated assertion rows passed in Rust.
- Nine additional Rust tests passed for branches absent from original assertions, including the directed numeric and branch-priority gaps from task97.
- 251,452 full-result differential cases passed, including all 256 legacy key codes, 16 modifier masks and eight option sets, all 65,536 BMP single-unit keys, shifted and lowercase dead-key BMP cases, mobile arrows and 20,000 seeded inputs.
- Random seed is 1262836051, hexadecimal `0x4b455953`.

`results/keyboard/verification.json` records exact counts, test names, source hashes and tool versions. Temporary runners and local logs stay under `tmp/keyboard` or `tmp/keyboard-*.log`. The runner compiles with `-Dwarnings` and uses the same Rust implementation as the core crate. No JS terminal logic runs inside Rust.

This evidence covers the legacy keyboard evaluator only. Native macOS/Linux execution, browser DOM/input integration, Kitty keyboard protocol and full terminal equivalence are separate checks. Overall equivalence remains NOT PROVEN.
