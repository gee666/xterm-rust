---
name: cursor-blink-check
description: Use when changing WebGL cursor blink timers, restart/idle/frame ordering, callback reentry or live browser window handling.
---

# WebGL cursor blink checks

Read `docs/cursor-blink.md`. This tests the actual Rust `webgl::cursor_blink::CursorBlinkStateManager`, not a JS engine delegate. Task 306 still owns renderer/addon/Root/full-suite wiring.

Use a current team compiler/browser grant, Cargo `-j2`, private O: target and O: TEMP/TMP. Do not run concurrent source edits with the negative control. Existing source esbuild/Playwright and wasm-bindgen CLI 0.2.100 suffice.

1. Set `SOURCE_ROOT` to the unchanged supplied reference. Run `node scripts/cursor-blink-oracle.mjs`. It verifies all 776 baseline source hashes, records unchanged build inputs and 38 source scenarios, and generates `tmp/cursor-blink/prelude.js`.
2. Run native `cargo test -p xterm-web --test webgl --all-features -j2` and strict native lib/test Clippy.
3. Run strict WASM Clippy with `-p xterm-web --target wasm32-unknown-unknown --lib --test wasm_cursor_blink --all-features -j2 -- -D warnings`.
4. Set `WASM_TEST=wasm_cursor_blink`, `CARGO_FEATURES=acceptance-hooks`, `WASM_PRELUDE` to the generated prelude, and `WASM_BINDGEN`/`PLAYWRIGHT_MODULE` to existing tools. Run `node scripts/browser-host-check.mjs`. All three test functions must pass in Chromium, Firefox and WebKit with zero ignored/filtered. The second assertion waits for actual iframe 600 ms timers and native frames, not sleeps.
5. Run `node scripts/cursor-blink-negative.mjs`. The production positive-delay guard is inverted; Chromium must pass exactly two functions and fail the lifecycle comparison. Finally restores exact source hash. Rerun the full three-engine matrix and strict WASM Clippy after restoration.
6. `node scripts/cursor-blink-audit.mjs` verifies retained source/durable evidence and reference hashes. `--runtime` additionally verifies actual compiler WASM, generated binding/snippet modules, prelude and native EXE while they exist. `--record` is a producer snapshot, NOT independent verification. Never refresh a receipt to disguise a mismatch.

Preserve raw failures, source/native traces and immutable pins. After review, inspect owned processes, source cleanliness and every reparse point before removing only the owned O: runtime paths. Runtime pins are deliberately absent after cleanup. Durable verification must still pass. Exact workspace fmt OS206 remains an unresolved full-gate failure; changed-file formatting is not a replacement.
