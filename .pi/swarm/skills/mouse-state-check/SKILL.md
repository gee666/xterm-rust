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

This includes `mouse_state`, `mouse_callbacks`, `mouse_custom`, independently written `mouse_differential`, and `mouse_registry_names` tests. It writes `results/mouse-state/verification.json` and per-command logs, enforces a 1 GiB free-space preflight, preserves shared build profiles, and fails honestly on any shared-module build or lint error.

A separate `scripts/mouse-raw-button-oracle.mjs` checks 4,200 signed-button rows against unchanged source, including DOM button -1 and int32 extremes. The full checker invokes both source oracles and 29 Rust integration tests per profile, including custom mutable encoders, shared protocol edits, retained callback errors and registry name order. Coordinate heavy jobs through the current resource owner, with Cargo `-j2` and owned O: target/TEMP/TMP. Do not use the retired build-slot queue.

The name-order fixture records three unchanged-source sequences, 178 registrations and 187 snapshots for each registry at seed `0x51a7e501`. It covers array-index boundaries, noncanonical numeric-looking keys, replacements, reset and disposal. `protocol_names` and `encoding_names` represent source `Object.keys` on own string registrations. Raw inherited lookup and `__proto__` mutation remain separate. Run helper tests with `cargo test -p xterm-core --lib object_keys`.

The corpora are behavioral evidence for the mouse service, not for DOM event routing or the full terminal.
