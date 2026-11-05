---
name: font-addon-check
description: Use when checking CSS font parsing, fallback ligature ranges or Rust/WASM web-font loading and relayout against the supplied source.
---

# Font addon checks

Run from repository root with Node24.15.0 and Rust1.85.1 on PATH:

```sh
node scripts/font-oracle.mjs
cargo test -p xterm-addons --test font_addons
cargo clippy -p xterm-addons --lib --test font_addons -- -D warnings
WASM_TEST=web_fonts_browser node scripts/browser-host-check.mjs
WASM_TEST=web_fonts_async_browser node scripts/browser-host-check.mjs
WASM_TEST=wasm_font_terminal node scripts/browser-host-check.mjs
WASM_TEST=wasm_font_shaping node scripts/browser-host-check.mjs
cargo build -j2 -p xterm-font-addons --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/debug/xterm_font_addons.wasm --target nodejs --out-dir tmp/font-node
SOURCE_PACKAGE_JSON=/path/to/installed/source_repo/package.json node scripts/ligature-legacy-check.mjs tmp/font-node/xterm_font_addons.js
```

Reuse the browser-host-check skill for WASM CLI and Playwright setup. It closes servers and browsers and uses the private target directory with disk checks. Set TEMP and TMP separately to the owned O: scratch directory. Browser tests run Rust/WASM, not source_repo's terminal engine.

Oracle checks are read-only for tracked fixtures by default. Use --generate only after inspecting drift retained under tmp/font-oracle/*.actual. Never regenerate expected values merely to pass. It executes16 original parser test bodies and assertions, verifies8224 UTF16 parser rows and4156 fallback rows. Seed0xF017CA5E. Reference hashes, exact counts and outputs are in results/font-addons/. Browser results are per-target under results/browser-host/.

The legacy runner executes all8 unchanged Node test bodies and assertions against source and Rust/WASM, adapting only imports and TypeScript. That legacy suite mocks font-finder, but current source uses browser font discovery, so it does not prove installed-font shaping. The separate wasm_font_shaping test loads real Fira Code bytes into FontFace and the Rust addon, disables all fallbacks, and checks actual terminal DOM spans and load-triggered refresh in three engines.

Raw character-joiner service checks use SOURCE_PACKAGE_JSON=/path/to/installed/source_repo/package.json node scripts/joiner-raw-oracle.mjs and WASM_TEST=wasm_joiner_raw node scripts/browser-host-check.mjs. The oracle executes24 unchanged original tests,37 directed cases and256 seeded cases, seed0x335C0DE. Three WASM tests compare293 raw cases, replay27 captured original calls and exercise actual Terminal mutation/error propagation. See results/joiner-raw and docs/raw-character-joiners.md. Raw DOM row checks use scripts/joiner-row-oracle.mjs, WASM_TEST=wasm_joiner_row and WASM_TEST=wasm_joiner_row_live with the same dependency/browser runner setup. They execute38 unchanged source tests,295 finite cases with seed0x348C0DE, explicit Rust-only safety errors and five live Terminal paths. See docs/raw-dom-joiners.md and results/joiner-row. Task364 retains broader callback/live-state and source-hang safety decisions. Use current resource grants, private O: targets and Cargo-j2, not a global queue.

See docs/font-addons.md and docs/font-legacy-and-shaping.md for contracts, commands, evidence and remaining raw joiner/optional metadata gaps. Native helper or component browser success does not prove complete addon equivalence.
