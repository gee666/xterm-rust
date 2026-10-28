---
name: move-to-cell-check
description: Use when checking normal or alternate buffer alt-click cursor sequences, wrapped Unicode counts and MoveToCell acceptance cases.
---

Install the original locked npm dependencies. Run from repository root:

```sh
node scripts/acceptance-oracle.mjs --case move-01
node scripts/acceptance-oracle.mjs --case move-02
cargo test -p xterm-core --test move_to_cell
```

The adapter in scripts/acceptance-adapters/move-to-cell.mjs executes the unchanged original four-test Mocha suite, four faithful Rust source cases and 512 native traces per case from seed 0x6d6f7665. It independently captures escape returns, errors, full canonical buffer/cursor state and ordered events, with artifact/source hashes. Ordinary runs reject fixture drift; use MOVE_GENERATE=1 only for reviewed regeneration.

Shared acceptance runner owns attempt directories and process-tree cleanup. Respect the 1024 MiB reserve and shared Cargo target/profile; do not launch workspace-wide builds just for this helper. Actual browser click routing is task70, not proved by native method traces. The source has no reverseWraparound argument and its asymmetric backwards slices must not be silently corrected.
