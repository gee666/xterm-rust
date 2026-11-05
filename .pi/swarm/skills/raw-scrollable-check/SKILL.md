---
name: raw-scrollable-check
description: Use when changing generic live Scrollable or callback-safe animation phases, to compare Rust engine values, events, frames and reentrant coercions against unchanged source.
---

# Raw live Scrollable checks

Read docs/raw-scrollable-engine.md, docs/raw-scrollable-getters.md and docs/raw-scroll-animation.md. Work in an owned O: worktree with TEMP/TMP and target there. Coordinate four heavy jobs and at most two browser matrices, Cargo-j2. Reuse installed ESBUILD_MODULE, PLAYWRIGHT_MODULE and matching WASM_BINDGEN0.2.100. Never change source_repo.

Verify `node scripts/raw-scroll-state-source.mjs --live`, plus default and `--animation` fixtures. Use `--write` only after inspecting reference extraction and cases. The live fixture bundles original Event/Lifecycle, not a simplified emitter. Production never imports it.

Run `cargo test -p xterm-web --all-features --lib --test scrollable --test viewport_controller -j2`. Run focused strict native Clippy and WASM Clippy with `--lib --test raw_scrollable_browser --test raw_scroll_animation_browser --test raw_scroll_state_browser`. Run `WASM_TEST=raw_scrollable_browser node scripts/browser-host-check.mjs` and the existing animation/state browser targets sequentially. Browser runner owns and closes browsers/server in finally.

Live test605cases/engine compare independent source/Rust programs, including identity, value errors, construction/acceptance/tick reentry, scheduler failures and seeded mixed actions. Getter test5804programs/engine exercise raw update entrypoints and selected-state, repeated-read, proxy, reentry/error order. They do not establish raw Element/state-object/browser scheduler/clock parity. Retain concise reports/logs and original source hashes before removing owned generated artifacts and worktree. Do not call scoped passing tests full equivalence.
