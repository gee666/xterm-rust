---
name: font-gsub-refs-check
description: Use when checking deferred GSUB referenced lookup errors, scalar/range reachability and constructor precedence after task497.
---

# GSUB referenced lookup checks

Reuse the shared lookup oracle and independent container writers. With installed source dependencies via SOURCE_PACKAGE_JSON, run `GSUB_CORPUS=refs node scripts/font-gsub-lookup-oracle.mjs` read-only. Do not regenerate an existing fixture to clear a failure.

Run `cargo test --locked -j2 -p xterm-addons --test font_gsub_refs`. This checks240 source-hashed cases plus all1384lookup/1038type9/570null cases. Then run strict affected native/WASM lint and dependents.

With an approved browser allocation, run `GSUB_CORPUS=refs node scripts/font-gsub-header-browser-source.mjs` and `WASM_PACKAGE=xterm-font-addons WASM_TEST=font_gsub_refs_browser WASM_TEST_TIMEOUT_MS=360000 node scripts/browser-host-check.mjs`. The latter executes real Rust synchronous loading and actual addon async loading, not source delegation.

Use an owned O: target/tmp, retain commands/raw results and exact artifact pins, release resources and clean owned generated outputs after review. See docs/font-gsub-refs.md for scope and results. Full equivalence is not established by these probes.
