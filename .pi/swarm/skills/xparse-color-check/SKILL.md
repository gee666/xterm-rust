---
name: xparse-color-check
description: Use when checking the Rust OSC X11 color parser and formatter against the unchanged xterm.js implementation and assertions.
---

Run from the repository root with Node 24 or newer and the Rust MSRV toolchain on PATH:

```sh
cargo test -p xterm-core --test xparse_color
node scripts/check-xparse-color.mjs
cargo clippy -p xterm-core --lib --test xparse_color --example xparse_color_oracle -- -D warnings
```

The script executes all seven upstream XParseColor test cases and 49 assertions. It reads the original test file, strips TypeScript syntax and replaces its imports with the upstream implementation and Node strict assertions. No source file or assertion is modified on disk. The two assertion methods used here have matching equality behavior for these string, undefined, and numeric-array cases.

It builds the Rust line-protocol test adapter, compares every possible channel value at widths one through four, compares formatter depths for all byte values, and adds 12,000 malformed-input cases with xorshift32 seed 1480806220. Results are written under results/xparse-color/. A mismatch retains its operation and outputs in counterexample.json. The script checks 153,872 operations in total. This does not establish terminal equivalence or test browser OSC integration.

`CARGO` and `RUSTC` can override executable paths. The Node runner limits build and adapter runs to 120 seconds each. The shell caller must also set a timeout. Node may report native TypeScript/module-detection warnings because the original package does not declare ESM. Do not modify source_repo to suppress those warnings.
