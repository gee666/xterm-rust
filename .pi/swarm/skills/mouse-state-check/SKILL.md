---
name: mouse-state-check
description: Use when checking MouseStateService source assertions, regenerating mouse encoding fixtures, or comparing Rust protocol restrictions with unchanged JavaScript.
---

# Mouse state verification

From the repository root with Node.js 24.15 or newer:

```sh
node scripts/mouse-state-oracle.mjs
```

This checks the tracked corpus against unchanged source behavior and writes `results/mouse-state/oracle.json`. To regenerate intentionally after checking the reference hashes, run `node scripts/mouse-state-oracle.mjs --generate`.

The script runs all eight original MouseStateService test bodies. It transforms TypeScript in memory, removes only erased interface imports, and resolves the original lifecycle/event/type modules. Node assertions replace Chai, with string-valued `throws` expectations retaining Chai's message-substring check. No source or assertions are edited.

The corpus has 12,226 traces at seed `0x6d0a5e`. It exhausts defined buttons, actions and modifier masks across every default protocol/encoding, then adds numeric boundaries and 4,096 seeded inputs. Encoding runs even when restriction rejects because these are independent source methods. Rows include modifier mutations and exact UTF-16 units, including lone surrogates from negative coordinates. Schema is stored in the report. Inputs include NaN, infinity, negative zero, fractions and address limits. `-` means empty encoded units.

Run source replay, formatting, debug/release Rust suites and warning-denying Clippy with raw results:

```sh
node scripts/check-mouse-state.mjs
```

This includes `mouse_state`, `mouse_callbacks` and independently written `mouse_differential` tests. It writes `results/mouse-state/verification.json` and per-command logs, enforces a 512 MiB free-space preflight, preserves shared build profiles, and fails honestly on any shared-module build or lint error.

A separate `scripts/mouse-raw-button-oracle.mjs` checks 4,200 signed-button rows against unchanged source, including DOM button -1 and int32 extremes. The full checker invokes both source oracles and 26 Rust tests per profile, including custom mutable encoders, shared protocol edits and retained callback errors. The corpora are behavioral evidence for the mouse service, not for DOM event routing or the full terminal.
