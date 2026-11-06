---
name: dsr-option-check
description: Use when changing private DSR color-query raw option access, callback phases or error propagation through the actual Rust parser.
---

# Private DSR option checks

From a private worktree, use `node scripts/dsr-option-oracle.mjs` with installed source dependencies. `XTERM_SOURCE_ROOT` may point to a read-only supplied source tree with dependencies. The script verifies all 776 reference files before and after execution, runs the unchanged headless Events/onData assertion, then regenerates 468 private DSR cases in memory and compares the checked-in fixture. Only `--generate` rewrites that generated fixture.

Run `cargo test --locked -p xterm-core --test dsr_option -j2` under the current resource coordinator's compiler grant. The cases cover raw vtExtensions and nested colorSchemeQuery getters, two buffers, nine raw values and thirteen callback actions. They compare full cells/attributes/buffers/cursor/modes, callback snapshots, event order and exact original error causes. The large TSV is generated lossless state data, not a handwritten code module.

For the original six-case browser baseline, run `node scripts/dsr-browser-original.mjs`. It bundles the real source Terminal and default UnicodeGraphemesAddon, preserving original assertions and 10-second timeout with one worker and no retries. For actual Rust browser binding proof, build with `node scripts/build-demo.mjs --debug`, then run `node scripts/e2e-upstream.mjs --suite core --grep 'DSR: Device Status Report' --no-build --timeout-ms 180000` under a browser grant. The grep selects six unchanged original cases in each of Chromium, Firefox and WebKit. Do not edit assertions or count these as full-suite proof. Check raw receipts, input identity and reference hashes. No global build queue is required.

See docs/dsr-option-callbacks.md and results/dsr-option. Full equivalence remains NOT PROVEN.
