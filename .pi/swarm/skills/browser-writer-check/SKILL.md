---
name: browser-writer-check
description: Use when checking browser Terminal write queue data/completion reentry, 12ms slicing, disposal or first-frame ordering against source.
---

# Browser writer check

Read `docs/browser-writer-queue.md` for scope and the retained uncontrolled vt34
failure. These five shared scenarios do not replace the original full capture.

Run in an isolated worktree, with a private target and TEMP/TMP under its tmp/
directory. Coordinate a heavy/browser slot before building. Use Cargo -j2.

1. Run `REFERENCE_ROOT=/path/to/unchanged/source_repo node scripts/terminal-writer-oracle.mjs`.
2. Run `WASM_TEST=wasm_terminal_writer node scripts/browser-host-check.mjs`.
3. Set WASM_BINDGEN to CLI 0.2.100 and PLAYWRIGHT_MODULE to an existing Playwright
   installation when needed. The source reference needs its existing esbuild and
   Playwright dependencies. Do not change source assertions or install new copies
   without a concrete need.
4. Expect five source cases and five Rust tests per browser in Chromium, Firefox
   and WebKit. Check exact event arrays and event-time cursor values, not just
   completion counts or final cells.
5. Retain source captures/hashes, Rust logs/counts/tool versions and artifact
   hashes in durable results. Browser-host-check keeps generated WASM packages;
   delete only owned generated outputs after preserving evidence and confirming
   no process uses them. The source runner removes its own bundle in finally.

The clock fixes only the source queue's performance budget branch. Timers and
animation frames remain real. Do not normalize or discard the old uncontrolled
vt34 mismatch based on these tests. Full capture, asynchronous handler and
resize-during-callback work remains separate.
