---
name: buffer-reflow-check
description: Use when checking BufferReflow helpers against the unchanged xterm.js source, exact full-cell snapshots, cursor exclusion, wide-wrap boundaries, line identities and layout deletion events.
---

Run `node scripts/buffer-reflow-oracle.mjs` from the repository root with Node 24, Rust/Cargo and installed `source_repo` development dependencies. It reuses upstream esbuild and Mocha, introduces no production dependency, and leaves source files and assertions unchanged.

The script bundles the actual reference helpers, BufferLine, CircularList and dependencies into `tmp/buffer-reflow-oracle/`. It regenerates 3,000 deterministic fixtures using seed `0xb0ffef10`, then runs all seven faithful helper assertion ports, direct error/event tests, and differential Rust tests. Widening compares full raw cells, UTF-16 strings, fg/bg, extended attributes/URL, trims/wraps, exact removals/layouts, callback-time list length, and surviving row identity/order. Shrinking compares exact wrap lengths. The checked-in fixture preserves every case for diagnosis. Subprocesses have 120-second timeouts. Native one-column narrow-cell behavior remains supported; one-column wide-cell zero-progress returns an error rather than looping forever.

`node scripts/check-oracle-startup.mjs --reflow` independently checks that missing dependencies and a blocked temporary directory cannot preserve an old passing receipt. It uses isolated temporary roots, not shared results.

Results and reference hashes are retained in `results/buffer-reflow/`. The script also invokes `scripts/buffer-baseline.mjs`, whose original Buffer/BufferSet/BufferReflow results live in `results/buffer/original.json`. This is scoped helper evidence, not complete terminal, addon, browser or three-platform equivalence. Check the recorded status before claiming a pass, since concurrent workspace edits may cause unrelated compile failures.
