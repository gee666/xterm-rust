---
name: terminal-engine-oracle
description: Use when comparing live Rust terminal writes, streaming boundaries, resize and scroll operations with the untouched xterm.js headless engine.
---

Run `node scripts/terminal-engine-oracle.mjs --generate` to generate reviewed deterministic source fixtures and source hashes under results/terminal-engine. Then run `cargo test -p xterm-core --test terminal_engine_differential`. Normal `node scripts/terminal-engine-oracle.mjs` rejects fixture drift. The oracle bundles unchanged source_repo headless source through its existing esbuild dependency, never calls JavaScript from Rust, and compares both buffers, every cell's width/text/fg/bg/extended attributes, cursor, scroll regions, viewport positions, modes and ordered source events.

Run `node scripts/engine-replies-oracle.mjs` for 20 unchanged original save/restore, cursor-style, window-option, XTVERSION and DECSTR cases plus their real CoreTerminal Rust assertion ports. Original and Rust outcomes and each attempted command log are retained under results/terminal-engine/replies. This runner does not claim the other InputHandler suites.

Keep failures visible and retain seeds. Seed and trace counts live in the script and report. This finite fixture corpus is not full equivalence proof. Existing headless-check and public-api-oracle skills cover separate original assertion suites and reentrant public callbacks. Use the repository's shared Cargo target and profiles; monitor disk before builds.
