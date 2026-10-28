---
name: scrollbar-helpers-check
description: Use when checking scrollbar geometry, visibility, FastDomNode caching and Widget listeners against unchanged source and Rust in real browsers.
---

# Scrollbar helper checks

Use an isolated C-volume worktree on Windows. Do not queue on a shared Cargo target or change reference source. Read `docs/scrollbar-helpers.md` for ownership, scope and exact commands.

1. Run `node scripts/scrollbar-state-oracle.mjs` without `--generate`. It must verify the existing 1,024 numeric geometry snapshots, seed 0x5c7011. Native `scrollbar_state_differential` consumes that fixture.
2. Run native `scrollbar_visibility`, `scrollbar_state_differential`, and lib filter `scroll::scrollbar_state`, then repeat with `--release`.
3. Run `node scripts/scrollbar-helpers-source-check.mjs`. It executes 17 shared fixture cases against unchanged source in all three browsers, records source/dependency/fixture hashes and retains per-case results under `results/scrollbar-helpers/source/`.
4. Run `scripts/browser-host-check.mjs` with `WASM_TEST` equal to `scrollbar_helpers_browser`, `wasm_fast_dom_differential`, `wasm_widget_differential`, and `wasm_visibility_differential`. Expected counts per engine are 7, 8, 4, 5. The latter three suites use the exact same fixtures as the source runner. Rust adapters instantiate Rust helpers and do not execute upstream policy.
5. Run scoped native/WASM Clippy with `-D warnings` and direct rustfmt checks of owned files. Attempt required workspace quality commands too and retain unrelated failures separately.

The source script accepts `ESBUILD_MODULE` and `PLAYWRIGHT_MODULE`; the browser runner accepts `CARGO`, `WASM_BINDGEN`, `PLAYWRIGHT_MODULE`. Set explicit module/executable paths when a worktree has no installed source Node dependencies. wasm-bindgen CLI must be 0.2.100. Do not relax artifact path checks. The target needs the standard Cargo cache marker; use the current worktree creator which initializes it.

No sleeps or unbounded polling are needed. Shared visibility tests inject deterministic timers and restore them in finally blocks; the existing helper suite also tests real zero-delay task delivery. Source and Rust runners close their own browser/server resources. Preserve historical failures rather than labelling an old pass as latest. Track small raw reports/logs, not generated WASM bundles or private runtime files.

These checks do not cover raw positions215, complete Element167, trusted scrollbar/Arrow input166 or terminal Viewport71 integration. Do not claim full scrolling or platform equivalence from them.
