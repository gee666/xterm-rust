# Win32 keyboard protocol

`crates/xterm-core/src/input/win32_input_mode.rs` ports the complete encoder in `source_repo/src/common/input/Win32InputMode.ts`. It has no OS calls or browser dependency. This is a protocol component, not a Windows console adapter or a complete terminal.

## Contracts and mapping

- `Win32InputMode.evaluateKeyboardEvent` maps to `Win32InputMode::evaluate_keyboard_event`.
- `Win32ControlKeyState` members map to constants in `control_key_state`, retaining every original numeric flag.
- Browser event data uses the shared `input::keyboard::KeyboardEvent`. Both `key` and `code` borrow UTF-16 units. `key_code` has the DOM unsigned-long range; the binding must perform WebIDL conversion before constructing an event.
- `Win32KeyboardEvent`, `Win32KeyboardResult`, and `KeyboardResultType` re-export the shared keyboard types. The original `type` member maps to `result_type`. Outgoing keys are `Some(Vec<u16>)`, cancellation is always true, and the result kind is `SendKey`.
- The encoder preserves keydown/key-up selection, left/right modifier decisions, enhanced keys, Ctrl special characters, fallback virtual-key codes and unknown scan-code zero. Repeat count remains 1 exactly as upstream. Lock flags remain defined but are not synthesized from events.
- A JS astral `key` has length 2, so it encodes Unicode value 0. A lone surrogate has length 1 and retains its numeric value. Rust's Unicode scalar count would change this behavior.
- `win32_key_codes.rs` contains the source virtual-key, approximate US scan-code and enhanced-key tables. Sorted tables allow shared UTF-16 events without allocating a temporary string. The differential matrix checks every mapped code with every modifier combination and key state.

## Reproduce

Use Node 24 or newer and Rust 1.85.1 or newer from the repository root:

```sh
node scripts/verify-win32-input.mjs
cargo test -p xterm-core --lib win32_input_mode
cargo test -p xterm-core --lib win32_input_mode --release
cargo clippy -p xterm-core --lib -- -D warnings
cargo fmt --all -- --check
```

If `rustc` is not on PATH, set `RUSTC` to its executable when running the Node script. Temporary runner source and binaries stay under `tmp/win32-input`. `--generate` regenerates the source lookup tables and faithful upstream case fixture; run rustfmt afterward and review source changes before accepting a new fixture.

The Node verifier executes the original test bodies and assertions after removing imports and TypeScript syntax. Node strict assertions replace the three Chai assertion methods used by this suite. The original production class supplies expected records. Every captured record then runs through compiled Rust. Rust's fixture test compares the entire result, including fields the original test checks only partially.

## Observed evidence

Native Windows x64, Node v24.15.0, rustc 1.85.1. See `results/win32-input/verification.json` for hashes, tool versions, seed and all test names.

- Original suite: 64 discovered, 64 executed, 64 passed, 0 failed/skipped/unrun.
- Differential: 13,904 complete encoded records matched, using seed 5,714,738. This includes the 64 upstream records, 3,840 mapped-code/modifier/key-state combinations and 10,000 generated UTF-16 cases.
- Rust: 4 focused tests pass in debug and release. These cover all 64 original records plus lone surrogates, astral length, Alt/Meta suppression of Ctrl conversion and maximal fallback key codes.
- Focused rustfmt and core-library Clippy with `-D warnings` pass. `results/win32-input/release-tests.log` retains the release output.
- No unsafe code or external dependencies were added.

Browser event routing, native Linux/macOS execution and complete terminal behavior are not covered by these results. Overall xterm.js equivalence is NOT PROVEN.
