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
```

Reuse the browser-host-check skill for WASM CLI and Playwright setup. It closes servers and browsers and uses the shared target directory with disk checks. Browser tests run Rust/WASM, not source_repo's terminal engine.

Oracle checks are read-only for tracked fixtures by default. Use --generate only after inspecting drift retained under tmp/font-oracle/*.actual. Never regenerate expected values merely to pass. It executes16 original parser test bodies and assertions, verifies8224 UTF16 parser rows and4156 fallback rows. Seed0xF017CA5E. Reference hashes, exact counts and outputs are in results/font-addons/. Browser results are per-target under results/browser-host/.

See docs/font-addons.md for typed API and empty-pattern safety deviations, fixture formats, current evidence and unimplemented GSUB/local-font/terminal integration. Native helper or component browser success does not prove complete addon equivalence.
