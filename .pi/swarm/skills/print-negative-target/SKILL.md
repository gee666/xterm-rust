---
name: print-negative-target
description: Use when changing print combining-cell targeting or custom Unicode width/join handling.
---

Run `node scripts/print-negative-target-oracle.mjs`, optionally with `--source-root PATH`, then `cargo test -p xterm-core --test print_negative_target -j2`. The runner verifies all original reference hashes, executes the unchanged eight-test InputHandler print suite and compares the 64-case, 160-snapshot custom-provider corpus. It removes its owned bundle in finally. Only use `--generate` when deliberately reviewing source-fixture changes.

The corpus includes a negative target after a zero-width first cell, normal target controls, widths 0 through 3, chunk splits, insert mode and ordered follow-up events. Negative typed-array writes must not be clamped onto visible cell 0; subsequent filler/cursor/preceding state work still runs. See `docs/print-negative-target.md`. Keep source assertions intact. These checks are scoped evidence, not full terminal/browser equivalence.
