---
name: osc-link-service-check
description: Use when verifying the OSC8 link-id and marker store against the original assertions and deterministic real BufferService operation traces.
---

Use the shared `upstream-baseline` skill prerequisites to install the original npm dependencies once. Keep shared target profiles unchanged and monitor free disk space.

From the repository root with Node24 and Rust1.85 or newer on PATH:

```sh
node scripts/check-osc-link-service.mjs
cargo test -p xterm-core --test osc_link_service
cargo test -p xterm-core --lib osc_link_service
cargo clippy -p xterm-core --lib --test osc_link_service --example osc_link_oracle -- -D warnings
```

The Node script builds original TypeScript source and the intact original OscLinkService.test.ts into tmp/osc-link-service using the original esbuild dependency. Actual Mocha/Chai execute all three original tests and five assertions. Original files stay unchanged.

It compares1,231 snapshots,31 directed and1,200 seeded operations, using original and Rust BufferService objects. The shared Rust trace runner lives in tests/support/osc_link_trace.rs and is reused by the fixture test and example, avoiding a separate service mock. The fixture records all known link data and marker line/disposed state. Seed is1330856780. A mismatch retains the full operation prefix and results.

Default verification checks fixture drift before comparing Rust. Use `--write-fixtures` only for intentional reviewed regeneration. Temporary files remain under tmp/. Both subprocesses have120-second timeouts; the calling shell also needs a timeout. Failure output must not be replaced by stale successful evidence. Results are under results/osc-link-service/.

See docs/osc-link-service.md and docs/acceptance/osc-link-service.md for live data mutation, separator collisions, id saturation, weak ownership and typed borrow-error boundaries. OSC parser/cell/browser integration is separate criterion250 and must not be claimed from this service suite alone.
