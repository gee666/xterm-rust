---
name: scrollable-check
description: Use when changing the numeric ScrollState, Scrollable scheduler, or smooth-scroll animation policy.
---

# Scrollable source check

Run from your worktree with Node24, Rust/Cargo and the read-only supplied source_repo available:

```
node scripts/scrollable-oracle.mjs
cargo test -p xterm-web --test scrollable
cargo test -p xterm-web --lib scroll::
cargo clippy -p xterm-web --lib --test scrollable --example scrollable_trace -- -D warnings
```

The script executes the supplied Lifecycle.ts, Event.ts and scrollable.ts bodies without changing them. It compares20150 deterministic/seeded snapshots against the Rust runner, including raw/clamped numbers, event fields, pending frames, request/cancel history, reuse and callback failures. Seed1396920908. A mismatch retains its reproducing rows in tmp/scrollable-counterexample.json.

This is numeric model evidence, not browser or arbitrary-JavaScript-value parity. Raw non-number positions remain task215, and actual Viewport/DOM integration remains task71. The source tree contains no scrollable unit test suite; do not label generated comparisons as ported original assertions.
