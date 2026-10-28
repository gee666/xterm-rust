---
name: native-demo-check
description: Use when validating or demonstrating the live native Rust terminal example, its command parser or script-mode assertions.
---

Run from the repository root with Rust1.85+, Cargo/rustfmt/Clippy and Node24+ available:

```sh
cargo run --locked -p xterm-core --example native -- --plain
cargo test --locked -p xterm-core --example native
node scripts/check-native-demo.mjs
```

Honor the shared disk/build policy. Do not start local builds during a coordinated disk pause. The checker requires1GiB free, uses the shared target directory, and never changes profiles or removes files. It reuses the existing quality process runner and immutable evidence-attempt helper, with no npm dependency.

The fixture crates/xterm-core/tests/fixtures/native-demo.txt drives the same executor as interactive commands. Its assertions inspect actual Rust cells/colors/widths/cursor/dimensions, resize reflow and outbound UTF16, not echoed text. Plain and ANSI runs, EOF disposal and six expected-failure cases are retained separately under results/demos/native/runs. Read latest.json for the current attempt; older successes do not establish a later run passed.

The source defaults to Unicode6: U+1F600 is width1, CJK width2. Default reflow skips the cursor's wrapped group; move the cursor below the group before checking widening. Do not change these assertions to match an incorrect Rust result. The unchanged JS source was separately checked for these cases.

Read docs/native-demo.md for commands, resource budget and UTF16/display mapping. Unknown commands, invalid dimensions, malformed escapes and failed assertions must return useful nonzero script errors. A source-changing run remains incomplete until a stable rerun. The native demo is not a PTY shell, browser renderer, full addon suite or proof of terminal equivalence. Native Linux/macOS and independent review remain separate gates.
