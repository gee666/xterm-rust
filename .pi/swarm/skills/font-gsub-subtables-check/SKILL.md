---
name: font-gsub-subtables-check
description: Use when checking zero GSUB subtable pointers, parser read order and deferred FontImpl null access against the original font parser and actual addon.
---

# GSUB subtable pointer checks

Run from the owned O: producer worktree. Reuse the installed original reference dependencies through SOURCE_PACKAGE_JSON. Do not change source_repo. Get the coordinator's compiler/browser grants first. Use CARGO_BUILD_JOBS=2 and a regular private O: target.

```sh
SOURCE_PACKAGE_JSON=P:/workshop/Windows_Worshop/oira666/xterm-rust/source_repo/package.json GSUB_CORPUS=subtables node scripts/font-gsub-lookup-oracle.mjs
cargo test --locked -j2 -p xterm-addons --all-targets --all-features
cargo clippy --locked -j2 -p xterm-addons --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-font-addons --target wasm32-unknown-unknown --test font_gsub_subtables_browser --test font_gsub_refs_browser -- -D warnings
SOURCE_PACKAGE_JSON=P:/workshop/Windows_Worshop/oira666/xterm-rust/source_repo/package.json GSUB_CORPUS=subtables node scripts/font-gsub-header-browser-source.mjs
WASM_PACKAGE=xterm-font-addons WASM_TEST=font_gsub_subtables_browser node scripts/browser-host-check.mjs
node results/font-gsub-subtables/verify.mjs
```

Set WASM_BINDGEN to the existing 0.2.100 executable and PLAYWRIGHT_MODULE to the installed original Playwright module. Do not reinstall dependencies. scripts/font-gsub-subtables-recipes.mjs builds 1224 directed SFNT/WOFF cases. --generate on the source oracle is only for an inspected new corpus, never to make Rust pass.

Native and browser tests retain lookup1384/type91038/null570/refs240 assertions. The new cases test null-only and mixed subtable arrays, types1-8, calt selection, scalar/range references, unreachable records, missing offsets/filter words and later META errors. The source browser baseline runs unchanged parser, FontImpl and real LigaturesAddon. The Rust browser test loads actual RustLigaturesAddon and checks loading/fallback/cache/refresh. This is scoped proof, not full font or terminal equivalence.

verify.mjs checks producer hashes, all776 reference files, reports and live native/WASM/bindings/CSS pins. After cleanup, use --retained to verify committed proof without removed generated binaries. Preserve reports before removing owned outputs. Do not replay old private509 or any cleanup. See docs/font-gsub-subtables.md for boundaries.
