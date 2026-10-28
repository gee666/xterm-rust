---
name: buffer-service-oracle
description: Use when changing BufferSet activation, BufferService scrolling, live option subscriptions, cached blank lines or buffer event ordering.
---

# Buffer service oracle

Read docs/buffer-service.md and docs/acceptance/buffer.md section E. From the repository root, run node scripts/buffer-service-oracle.mjs with Node24 and the existing source_repo development dependencies installed. It reuses buffer-baseline.mjs esbuild/Mocha method and does not modify reference source or assertions.

Ordinary verification executes all13 original tests and detects fixture drift. Use --generate only for reviewed upstream changes. It saves64x64 deterministic operation snapshots, seed0x05e7b0ff, source hashes and counts to results/buffer-service/oracle.json. Rust replay compares both actual buffers, all generated cell content/text/fg/bg, cursor/viewport/region/capacity/tabs and service event order. The corpus has no extended attribute words and must not be presented as full terminal equivalence. Six additional source-generated reentrant traces record callback-time full state for marker mutations, nested scroll, activation/reset identity, switch during trim and resize during trim. Replay with --test buffer_service_reentrant. Parent task122 separately covers callbacks inside resize/reflow itself.

Run cargo test -p xterm-core --test buffer_set_upstream --test buffer_service_upstream --test buffer_service_edges --test buffer_service_differential, repeat with --release, and run focused Clippy with --lib and the same test selectors followed by -- -D warnings. Keep explicit shell timeouts and do not create duplicate target directories. See docs/buffer-service.md for API/ownership/error/disposal contracts and actual native/browser evidence limits.
