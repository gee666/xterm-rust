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

Mac kern diagnostic checks use `node scripts/font-kern-warning-oracle.mjs`, `node scripts/ligature-addon-oracle.mjs --kern-warnings`, `cargo test --locked -j2 -p xterm-addons --test font_metadata_constructor` and `WASM_PACKAGE=xterm-font-addons WASM_TEST=font_kern_warning_browser node scripts/browser-host-check.mjs`. Set SOURCE_PACKAGE_JSON for the unchanged installed oracle dependency. These compare192 constructor cases and10 async addon scenarios. Keep this browser binary separate from `font_metadata_browser`, since their different hosts share a realm-wide font catalog. The retained metadata binary needs its documented Firefox timeout180000. See docs/font-kern-warning.md and results/font-kern-warning; do not claim live ArrayBuffer mutation parity from immutable Rust snapshot tests.

TrueType constructor boundary checks use `node scripts/font-outline-oracle.mjs --source=/path/to/source_repo`, `node scripts/font-outline-addon-oracle.mjs --source=/path/to/source_repo`, `cargo test --locked -j2 -p xterm-addons --test font_outline` and `WASM_PACKAGE=xterm-font-addons WASM_TEST=font_outline_browser node scripts/browser-host-check.mjs`. Set SOURCE_PACKAGE_JSON for the unchanged source dependencies. These compare612 constructor recipes and4 actual addon loading/cache/recovery scenarios, seed0x10CA453. Check input hashes and retain source failures; arbitrary glyph body/loca offset values intentionally stay lazy. See docs/font-outline-validation.md and results/font-outline. CFF/CFF2 and full rendering parity remain separate work.

GSUB header checks use `node scripts/font-gsub-header-addon.mjs`, `cargo test --locked -j2 -p xterm-addons --test font_gsub_header`, `node scripts/font-gsub-header-browser-source.mjs` and `WASM_PACKAGE=xterm-font-addons WASM_TEST=font_gsub_header_browser node scripts/browser-host-check.mjs`. Set `SOURCE_PACKAGE_JSON` to installed read-only dependencies, and set `WASM_BINDGEN` and `PLAYWRIGHT_MODULE` for external tools. The 1,496-case SFNT/WOFF corpus covers version decoding, null root/script/language pointers and retained-only variation records. Reuse bounded runners with an approved resource allocation. See docs/font-gsub-header.md for scope and receipts, not full GSUB equivalence.

GSUB lookup read-order checks use `node scripts/font-gsub-lookup-oracle.mjs`, `cargo test --locked -j2 -p xterm-addons --test font_gsub_lookup`, `GSUB_CORPUS=lookup node scripts/font-gsub-header-browser-source.mjs` and `WASM_PACKAGE=xterm-font-addons WASM_TEST=font_gsub_lookup_browser node scripts/browser-host-check.mjs`. Reuse the existing header byte builder and actual-addon scenarios. The1,384-case SFNT/WOFF corpus tests early invalid-type rejection and late mark-filter reads; type9/null/class/shaping gaps stay open. See docs/font-gsub-lookup.md and results/font-gsub-lookup, with installed read-only dependencies and approved O: resources.

See docs/font-addons.md and docs/font-legacy-and-shaping.md for contracts, commands, evidence and remaining raw joiner/optional metadata gaps. Native helper or component browser success does not prove complete addon equivalence.


Task483 GSUB type9 checks reuse the473 oracle/builders with GSUB_CORPUS=type9, native font_gsub_type9 and actual WASM font_gsub_type9_browser. The source-browser runner accepts the same selector. Keep default lookup/header modes intact. See docs/font-gsub-type9.md for null-pointer, sequential error ordering and referenced-null constructor cases. Full342/325 remains open.
