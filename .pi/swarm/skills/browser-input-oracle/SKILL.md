---
name: browser-input-oracle
description: Use when validating Rust clipboard text processing and IME composition policy against unchanged upstream browser input assertions and seeded UTF-16/deferred traces.
---

# Browser input oracle

Reuse the TypeScript-erasure and fixture-drift approach from verify-keyboard. From repository root with Node supporting stripTypeScriptTypes and cargo/rustc on PATH:

```
node scripts/browser-input-oracle.mjs
cargo test -p xterm-web --test input_upstream
```

The loader removes only TS types, imports/exports and constructor DI decorators. Original Clipboard and CompositionHelper assertions run unchanged. The scheduler preserves FIFO zero-delay task order without sleeps. It records native-textarea mock changes and policy callbacks, then compares exact UTF-16, emitted event boundaries, overlay text, composing state and pending callbacks with the compiled Rust input_trace example.

Use --generate only after source review to regenerate crates/xterm-web/tests/fixtures/browser-input-upstream.tsv. Ordinary runs reject fixture drift. The seeded corpus includes all BMP units and synthetic composition/task/selection races. See results/browser-input/verification.json for exact counts and seed; keep counterexamples after fixes.

This checks policy, not DOM delivery, geometry, IME platform integration, cancellation or browser timing. Those require actual Rust-backed Chromium/Firefox/WebKit tests. Preserve .cargo/config.toml shared low-artifact profiles, do not create a duplicate target directory, and stop the build when the resource preflight reports insufficient disk.
