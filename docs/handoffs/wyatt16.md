# Mouse policy recovery and remaining raw service work

## Integrated component

Task291 was independently accepted and canonically merged into main at `137dbf5f7259e3e2bac70adca44a0f312812febc`. This recovers Tuah's reviewed `c307ed3` shared policies and module-format fix without unrelated branch history. Current main's newer browser runner remains unchanged.

Fresh Windows evidence:

- `node scripts/mouse-fields-oracle.mjs` verified all 1,125 source cases without rewriting expectations.
- Focused native tests passed 27 core plus 13 web policy/routing tests.
- `WASM_PACKAGE=xterm-wasm-support WASM_TEST=wasm_mouse_fields node scripts/browser-host-check.mjs` passed all 1,125 cases in each of Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4.
- Native and WASM library warning-denied Clippy and focused rustfmt passed. Independent review reran native tests, source fixtures, Clippy and formatting; it inspected the browser reports.
- `node scripts/worktree-merge.mjs` passed its canonical changed-crate build/all-target tests, affected-dependent all-target checks, workspace check and applicable WASM library checks. Its log reports 1,501 passing native tests across 231 test/example invocations, including zero-test invocations. This is not the full all-feature/release/platform matrix.

Exact commands, source/artifact hashes and logs are in `results/mouse-fields/*-recovery.log`, `results/mouse-fields/gated-recovery-merge.log` and `results/browser-host/xterm-wasm-support__wasm_mouse_fields/2026-11-05T08-27-23-449Z-9ffc7e6d-086e-40f2-9873-0eff7e9f9d97/`. Generated browser binding/WASM artifacts were removed after use; their hashes remain in the receipt.

## Task305, not implemented

Task305 owns the full raw registry and live callback facade. The recovered component supplies only five builtin restrictions and three builtin encoders through one Rust algorithm. It does not complete MouseStateService's JavaScript object semantics or the terminal mouse path.

Read `docs/mouse-interop-source-contract.md`, `docs/mouse-registry-gaps.md`, `scripts/mouse-interop-oracle.mjs`, original `source_repo/src/common/services/MouseStateService.ts` and the current core service before designing the host boundary. Source-only probes already cover inherited names, numeric key order, `__proto__`, exact callback receivers, poisoned `.call`, strict-false wheel results and thrown identity. Do not count those probes as Rust parity.

Current integration seams:

- Core `mouse_state.rs` owns terminal mode state, typed registration vectors and infallible typed restriction/encoding callbacks. Raw JS callbacks can throw or return objects. A lossless raw path needs fallible boundaries rather than swallowed exceptions or string conversion.
- `terminal/input_services.rs::custom_wheel` calls a `Function` with undefined receiver through `.call1`. Source calls its stored handler as a direct method of the actual MouseStateService. The receiver must expose real live methods/state, not an empty fake object. Source wheel calls ignore a poisoned function `.call` property; event listeners intentionally observe it.
- `terminal/mod.rs`, `browser_services.rs`, `browser_open.rs` and `operations.rs` currently store a separate `custom_wheel` cell. They need a coherent service boundary, not a duplicate terminal protocol authority.
- `terminal/input_mouse.rs` uses the shared native mouse service. DOM policy/routing tests already cover its typed path. Raw restriction and encoder calls must retain the original JS event identity, repeated accessors, extra properties and returned value identity.
- Protocol restriction receiver is its protocol object. Encoding receiver is the encoding registry. Wheel receiver is the real MouseStateService. Default protocol objects are module-shared upstream; account for externally mutated identity across service instances.
- Diponegoro agreed not to edit `input_services.rs::custom_wheel`; his InputEvent work is separate. Coordinate any broader shared Root edits with current owners.

## Temporary ownership

Only new temporary root was `O:/xterm-rust-swarm-16/tmp/worktrees/Wyatt16`, branch `swarm/Wyatt16`. No servers or background processes remain. The canonical helper removed its private target after the source merge. Browser artifacts were removed after retaining receipts. Before removing the worktree, preserve this documentation/evidence commit, inspect dirty and untracked paths, and verify no reparse points. The helper made a physical `source_repo` copy, not a junction. Never remove older C:/Tuah or other retained worktrees.

Overall equivalence remains NOT PROVEN. Recovery is integrated and tested; raw facade, complete upstream/browser/demo acceptance, final quality/platform matrix and final comparisons remain separate work.
