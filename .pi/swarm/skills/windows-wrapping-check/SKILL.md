---
name: windows-wrapping-check
description: Use when verifying Windows wrapping policy transitions, linefeed listener ordering, or CSI H handler precedence in the real terminal.
---

# Windows wrapping verification

Run `node scripts/windows-wrapping-oracle.mjs [source-root]` to regenerate 15 unchanged source-terminal traces and their source hashes. Then run `cargo test --locked -p xterm-core --test windows_wrapping_order --test windows_terminal --test platform_windows`.

The source root defaults to source_repo and needs esbuild. It is read-only. Expected TSV traces live in results/windows-wrapping/source.tsv and are consumed directly by Rust tests. Preserve callback failure and before/after registration assertions. See docs/windows-wrapping-order.md for limits and broader gates.
