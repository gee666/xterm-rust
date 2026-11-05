---
name: font-gsub-single-check
description: Use when checking invalid GSUB single-substitution formats before coverage reads, including extension lookups and later metadata errors.
---

# GSUB single format checks

Use Node24.15.0/Rust1.85.1 and installed reference dependencies via SOURCE_PACKAGE_JSON. Keep owned TMP/TEMP/Cargo target on O:, Cargo -j2, and request a resource allocation before compiler/browser jobs.

`GSUB_CORPUS=single node scripts/font-gsub-lookup-oracle.mjs` replays608 source-hashed SFNT/WOFF constructor, FontImpl and actual-addon cases. It shares independent container writers with prior corpora. Existing fixtures are immutable, do not regenerate them to obtain a pass.

`cargo test --locked -j2 -p xterm-addons --test font_gsub_single` checks the608 new cases plus1384 retained lookup cases. Run strict affected native/WASM checks as documented in docs/font-gsub-single.md.

With a browser allocation, `GSUB_CORPUS=single node scripts/font-gsub-header-browser-source.mjs` runs unchanged source in all three engines. `WASM_PACKAGE=xterm-font-addons WASM_TEST=font_gsub_single_browser WASM_TEST_TIMEOUT_MS=360000 node scripts/browser-host-check.mjs` executes Rust parser and actual-addon async loading in Chromium/Firefox/WebKit. Set WASM_BINDGEN and PLAYWRIGHT_MODULE to existing matching installations, without copying authentication or creating reference junctions.

The native UnsupportedFormat enum intentionally retains the existing parser-family/value representation rather than the source's byte-offset prose. This task fixes read/error order, not all GSUB coverage/class parsing or full font equivalence. Retain command logs and artifact pins, release allocations, and clean only owned temporary outputs after review.
