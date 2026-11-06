---
name: font-cff2-fonts-check
description: Use when checking eager CFF2 FDArray font/private dictionary and local Subrs constructor errors against unchanged source, including fractional private fields and inter-font ordering.
---

# Eager CFF2 font dictionaries

Read docs/font-cff2-fonts.md and docs/intent-nefertiti546.md. Follow task533, excluding its seven private prerequisite copies from pickup. Reuse the shared test-only font_cff_container.rs writer, original source hash737212d8..., existing addon lifecycle scenario and browser-host-check runner. Seed0x546cff02 currently defines408 recipes. Never launch known infinite-size/NaN-address or multi-operand Subrs allocation hazards against source. Directed native safety tests are observations, not equivalence claims.

With a fresh resource allocation, O:private target/TEMP/TMP/TMPDIR and Cargo-j2:

```sh
SOURCE_PACKAGE_JSON=/absolute/reference/package.json node scripts/font-cff2-fonts-oracle.mjs
cargo test -j2 -p xterm-addons --all-targets --all-features
node scripts/font-cff2-fonts-negative.mjs
cargo clippy -j2 -p xterm-addons --all-targets --all-features -- -D warnings
cargo clippy -j2 -p xterm-font-addons --target wasm32-unknown-unknown --all-targets --all-features -- -D warnings
SOURCE_PACKAGE_JSON=/absolute/reference/package.json node scripts/font-cff2-fonts-browser-source.mjs
WASM_PACKAGE=xterm-font-addons WASM_TEST=font_cff2_fonts_browser WASM_TEST_TIMEOUT_MS=600000 node scripts/browser-host-check.mjs
```

Oracle fixtures are immutable by default. Initial --generate records unchanged source outcomes only after input inspection. Each source worker has128MiB heap and3000ms deadline; retain exact stderr and resource failures, reject unknown child failures. New native tests retain the complete predecessor assertion chain. New browser tests execute only this corpus and actual addon lifecycle, avoiding redundant predecessor matrices. Use matching wasm-bindgen0.2.100 and existing Playwright dependencies without installations. The negative requires four designated executed duplicate-key mismatches and exact source restoration, followed by a positive replay.

Record actual commands/counts/raw logs, source/compiler hashes and initial failures. No full342/platform/equivalence claim follows from scoped green gates. Preserve source/evidence before cleanup; close owned processes, inspect every owned path for links, remove only this task's O:tree and verify original reference hashes afterward.
