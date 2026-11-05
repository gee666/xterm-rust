---
name: distribution-size
description: Use when measuring release native or web distribution sizes and refreshing original-versus-port source inventories without hiding missing assets or claiming functionality equivalence.
---

# Distribution and source sizes

Run `node --test scripts/distribution-size.test.mjs scripts/measure-original-size.test.mjs scripts/count-loc.test.mjs` first.

Use `node scripts/distribution-size.mjs --manifest FILE --root DIR --output FILE` for an explicitly staged shipped distribution. Every regular file must be selected with a role or excluded with a reason. The output must be outside the input directory. Native artifacts report raw bytes only. JS/web files report independent-file gzip9 and Brotli11 generic/window22 totals and verified compression round trips. Duplicate references to the same path count once; distinct paths never deduplicate by content. Linked paths, traversal, missing/undeclared assets and incomplete release provenance fail.

For original core ESM builds, use `scripts/measure-original-size.mjs --source ISOLATED-REFERENCE --dependencies EXISTING-NODE-MODULES --staging NEW-OWNED-DIR --output DURABLE-RESULTS`. It executes unchanged upstream `bin/esbuild.mjs --prod` and `--prod --headless`, reuses existing dependencies through a temporary link, verifies unchanged source inventories, preserves build/manifest/size JSON and removes its own link/staging in finally. Generated upstream outputs remain in the isolated reference until worktree cleanup. Never point staging at a retained directory. Reference source must have no node_modules entry.

Refresh `node scripts/count-loc.mjs --check-baseline` after integration stabilizes. This counts physical/nonblank lines including comments, not comment-stripped SLOC. Generated runtime glue must be present for final complete distribution counts.

See docs/comparison.md for manifest schema, raw measurements and missing distributions. Keep native executable size separate from JS download size. Sizes do not validate outputs, timing or equivalent functionality. Final matched release measurements and original/Rust addon totals remain required even if one partial inventory passes.
