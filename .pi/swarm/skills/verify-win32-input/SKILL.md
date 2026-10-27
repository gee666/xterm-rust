---
name: verify-win32-input
description: Use when verifying the Rust Win32 keyboard protocol encoder against upstream xterm.js assertions and deterministic differential cases.
---

# Win32 keyboard verification

From repository root, run `node scripts/verify-win32-input.mjs` with Node 24 or newer and rustc on PATH. Set RUSTC to the compiler path if necessary. The script uses Node's TypeScript transform to execute unmodified upstream test bodies with Node assertions. It requires the read-only source_repo tree.

Run `node scripts/verify-win32-input.mjs --generate` only after reviewing upstream changes. This regenerates Rust virtual-key and scan-code lookup data and the faithful upstream case fixture. Run rustfmt on changed Rust files afterward.

The verifier compiles a temporary Rust protocol runner under tmp/win32-input, executes every original Win32 test and compares complete Rust and JavaScript encoded records for all table codes, modifier combinations, key states and 10,000 seeded malformed UTF-16 cases. It saves counts, seed, versions and test names to results/win32-input/verification.json. A differential failure saves counterexample.json there.

Also run `cargo test -p xterm-core win32_input_mode`, `cargo fmt --all -- --check`, and workspace Clippy. These checks cover this encoder only, not browser adapters or complete terminal behavior.
