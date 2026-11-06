# Janaka553 handoff

Task553 fixes one exported marker field contract. Full task331 and full Rust xterm.js equivalence remain NOT PROVEN.

## Source and change

Base is main `224d1b339a43eddf899c2944d9dad734d8c44234`. Before edits, main was tracked-clean with unrelated untracked `swarm/` preserved. The old P: checkout and Sarah537's accepted archive/cleanup handoff were inspected, not replayed.

`source_repo/src/common/buffer/Marker.ts` declares `onDispose` as an own writable, enumerable, configurable data property initialized from the native emitter. Rust previously exported only a readonly prototype getter. `marker_api.rs` now installs the public data property once when creating a handle. The Rust emitter still controls disposal and buffer membership. Replacing or deleting the public property does not replace that emitter. No native terminal engine runs in JavaScript.

This branch has no private producer prerequisites. It does not contain Sarah537. When composing with537, preserve its `BrowserMarker::native()` helper. This change removes only the old `BrowserMarker::new`, stored event field and exported getter. Do not delay the accepted public decoration integration for this scoped fix.

Raw marker arrays, `line`/`isDisposed`, structural decoration markers, cross-WASM handles, global-object tampering and adversarial GC remain task331 work. This scoped descriptor trace is not proof of those contracts.

## Verification

Windows x86_64, rustc/cargo1.85.1, Node24.15.0, wasm-bindgen0.2.100. Compiler/browser allocation was granted in post409 comment4771, started4780 and fully released4820. No CI, main-ref or canonical integration grant was requested or used.

- `REFERENCE_ROOT=<unchanged P: reference> MARKER_ORACLE_REPORT=results/marker-event-property/source.json node scripts/terminal-marker-oracle.mjs`: five shared captures pass in Chromium, Firefox and WebKit. The reference sources and assertions were not edited. The bundle is generated only in owned O: scratch.
- `WASM_TEST=wasm_terminal_markers node scripts/browser-host-check.mjs`: five Rust test functions per engine pass initially and after the control. Existing four marker functions are retained unchanged. New tests check descriptors, raw replacements, direct calls, deletion, throwing getter identity, old event functions, resource order, trim and cached handle identity. Positive matrices have zero page errors and zero error-level console messages.
- `node scripts/marker-event-property-negative.mjs 224d1b339a43eddf899c2944d9dad734d8c44234`: exact old `marker_api.rs` compiles and passes four functions, failing only the new descriptor assertion in Chromium. A `finally` restores exact production SHA. Final compiler WASM, served WASM and binding hashes equal the initial fixed run and differ from the old-production run. The async test resolves failure observations rather than rejecting across the test transport.
- `cargo clippy -p xterm-web --all-targets --all-features -j2 -- -D warnings`: pass.
- `cargo clippy -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_terminal_markers --all-features -j2 -- -D warnings`: pass.
- `cargo test -p xterm-web --all-targets --all-features -j2`: 426 tests pass across180 summaries, zero failed/ignored/measured/filtered. This is not a complete workspace/release/platform gate.
- Exact `cargo fmt --all -- --check` fails with Windows OS206. All five separate whole-member checks pass. This is not a waiver or an exact-workspace pass. `git diff --check` and both JavaScript syntax checks pass.

Read `results/marker-event-property/{source,initial,control,final,index}.json` and retained raw logs. The index pins source inputs, durable receipts and temporary runtime files available during review. Both original and owned physical references match all776 baseline hashes before and after tests. The original baseline is `docs/baseline/reference-manifest.json`.

## Ownership and cleanup

Only owned temporary root is `O:/xterm-rust-swarm-16/tmp/worktrees/JanakaMarkerEvent`, including regular target/tmp, a physical reference copy and browser driver outputs. Existing P: Playwright, esbuild and wasm-bindgen tools were read, not copied, installed, modified or cleaned. TEMP is set first; TMP and TMPDIR are assigned separately to the same owned O: scratch path.

No jobs remain. Review precedes cleanup. Preserve the committed branch/evidence and inspect every path for links/redirects before deletion. Remove only owned generated outputs, then use normal non-force worktree removal. This document does not claim cleanup has happened. A later post/private receipt must establish actual absence and unregistration. Never replay another owner's cleanup or touch reference junctions.
