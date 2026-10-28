---
name: mouse-live-oracle
description: Use when validating MouseService live event getters, callback mutations and short-circuit host reads against unchanged JavaScript methods and Rust DOM bindings.
---

# Mouse live-read verification

Reuse browser-mouse-check for the complete original15-test baseline and pure coordinate/wheel/report corpus. This skill adds observer cases, not replacement assertions.

Run `node scripts/mouse-live-oracle.mjs` from repository root with Node24 supporting stripTypeScriptTypes. It invokes no Cargo, bundler or browser and writes under20MiB, so the source-only check requires no build lease. It executes unchanged MouseService/MouseCoords/getCoordsRelativeToElement/getWindow methods after annotation/import/DI erasure, compares the ten-case fixture byte-for-byte and records exact reference hashes in results/mouse-live/source.json. Use --generate only for reviewed source changes.

Run actual Rust browser assertions using the controlled build queue:

```
node scripts/build-slot.mjs --owner NAME --lane wasm --wait-ms 600000 --timeout-ms 300000 -- node scripts/check-mouse-live.mjs
```

The suite currently plans24 Rust tests, including the ten source-driven ordered traces and live getter/error/recursion/ownerDocument cases. Inspect actual reported counts; a build failure means tests unrun. The scoped checker hashes relevant production/test sources before and after the owned build and rejects changed snapshots, then links the immutable three-browser artifact report. Source metadata groups plain render fields to the corresponding MouseReadHost methods. Complete original suites, real gesture recognition and full Terminal integration remain separate gates.

Use the shared profiles/target and existing dependencies only. Honor the1GiB resource guard and retain slot receipts, failed builds and browser results. Do not launch a second build manually when queued or bypass a disk pause. See docs/mouse-live-reads.md and docs/browser-mouse-dom.md for scope and remaining evidence.
