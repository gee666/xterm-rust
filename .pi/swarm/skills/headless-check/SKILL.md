---
name: headless-check
description: Use when running unchanged headless public Terminal assertions or checking the native API wrapper against the source baseline.
---

Run `node scripts/headless-oracle.mjs` from the repository root after installing the reference's npm dependencies. Node 24, esbuild and Mocha are used. The script bundles unchanged source/test bodies into repository-root tmp/headless-oracle/, runs every test with Mocha and records source hashes, tool versions, exact suite outcomes and counts in results/headless/upstream.json. It does not edit source_repo or invoke Rust. These are 45 JavaScript baseline cases, not Rust coverage.

Run `node scripts/check-headless.mjs` for scoped original and Rust cases plus warnings-denied Clippy. It reuses quality-process.mjs for bounded process-tree handling, checks free disk space and retains each run under results/headless/runs/. Before/after hashes cover core source, tests and Cargo configuration. Concurrent changes invalidate a passing snapshot rather than being ignored. results/headless/latest.json points to the newest report; failures are never deleted.

Native public checks include headless_upstream, headless_contract, headless_reentry, headless_queue, headless_queue_edges, public_api_validation, headless_options, headless_modes, public_api_integration and public_adapter_terminal. The 45 original headless case names are mapped by scripts/check-headless-mapping.mjs. Numeric helpers alone do not prove engine dispatch or the browser ABI. Full wrapper acceptance is docs/public-api-acceptance.md PA01-PA06.

Run `node scripts/headless-reentry-oracle.mjs` for six directed unchanged-source reentrant mutations at a queued-write/flushSync boundary. These record exact callback ordering, final state and errors in results/headless/reentry-source.json. The matching native assertions remain mandatory; a source-only result is not port coverage.

Use the shared target directory and repository Cargo profiles. Monitor disk before large builds. Save failed commands and logs, do not regenerate passing evidence over a failed run without retaining the failure.
