---
name: upstream-browser-review
description: Use when independently reviewing the supplied xterm.js 6 browser baseline's raw reports, source hashes, attachments, counters and preserved failures.
---

# Original browser evidence review

Read `docs/upstream-browser-baseline.md` and choose its immutable complete runtime summary. Do not use a newer filtered/discovery pointer as if it represented the full matrix.

```sh
node scripts/verify-upstream-browser-evidence.mjs "$SUMMARY"
node scripts/build-slot.mjs --owner browser-accounting-review --lane wasm -- node scripts/upstream-browser-analyze.mjs "$SUMMARY"
```

The first command reads files only and creates a small per-attempt audit receipt. It compares all 14 raw Playwright reports with stored cases/results, rechecks all 776 original hashes, verifies every referenced attachment as a regular retained file, checks server shutdown and rejects counter disagreement. Expected supplied-reference totals are 4,116 discovered, 3,824 executed, 3,752 passed, 72 failed, 292 skipped and zero unrun. These fixed totals must not be edited to accommodate a mismatch.

The analyzer launches and closes three actual browsers to identify installed versions. It requires a controlled WASM/browser lease. Its success means accounting succeeded, not that the original matrix passed.

A scoped runtime reproduction can use:

```sh
node scripts/build-slot.mjs --owner browser-fit-review --lane wasm --timeout-ms 180000 -- node scripts/upstream-browser-baseline.mjs --run --suite=addon-fit --timeout-ms=120000
```

That original Fit scope currently reproduces 4 pass/5 fail in Chromium, 4 pass/5 fail in Firefox and 9 pass/0 fail in WebKit. Exit 1 is retained as a failed original suite. The managed server must stop. Never widen original assertions, reuse an unrelated server, or call a filtered run a full matrix. The latest baseline pointer can advance to the filtered attempt; the complete historical receipt remains immutable.

For a negative control, copy a complete summary under repository-root `tmp/`, change one stored project counter, and require audit exit 1 without modifying raw reports or source. Run the unchanged receipt again afterward. Receipts are under `results/upstream-browser-review/`; independent review examples are under `results/reviews/joyce-browser-baseline/`.

Neither audit success nor original JavaScript execution establishes Rust behavior, a green original baseline, native Linux/macOS runtime, or complete terminal equivalence.
