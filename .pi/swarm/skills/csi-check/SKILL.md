---
name: csi-check
description: Use when verifying cursor, tab, margin, edit, scrolling, erase and REP handlers through the real Rust terminal parser and the unchanged JavaScript source.
---

Run `node scripts/check-csi.mjs` from the repository root with Node 24, Rust/Cargo and the reference's installed development dependencies. `--debug-only` skips the release rerun and explicitly records that choice. Use the shared target and configured profiles. The runner reuses quality-process and quality-resources for process-tree timeouts and disk-headroom monitoring. It marks the next receipt incomplete before setup, retains every attempt, hashes source dependencies before/after and rejects changing snapshots.

`scripts/csi-baseline.mjs` executes 52 unchanged original InputHandler cases through real Mocha/Chai. Its mapping distinguishes 37 fully mapped cases from 15 partial cases whose negative direct internal cursor precondition cannot currently be represented by Rust's unsigned Buffer fields. Those assertions belong to task 152 and must never be counted as passed Rust coverage.

`scripts/csi-oracle.mjs --generate` explicitly regenerates the checked-in CSI fixture. Normal checks regenerate in memory and assert exact equality. It parses 249 traces and 2,515 operation snapshots through the actual source terminal, using all 37 command families, defaults/zero/ordinary/large parameters, nonzero margins, origin mode, Unicode/combined cells, chunks and seed `0xc5107`. Snapshot projections are shared with the whole-terminal runner. Rust tests compare both buffers, full cell/attribute/UTF-16 state, cursor/base/display/margins/modes and ordered events. Callback tests separately verify row identity/event observations, dynamic margins and exact error cause retention.

Read `results/csi/latest.json` and follow its report. A failed or incomplete status is not a passing gate. Counterexamples live in `results/csi/counterexamples.txt`. The corpus and faithful case migration are scoped checks, not browser, addon or three-platform equivalence. Details are in `docs/acceptance/csi-editing.md` and `docs/csi-editing.md`.
