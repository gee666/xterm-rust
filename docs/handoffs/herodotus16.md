# Herodotus input recovery handoff

## Accepted and integrated

Task343 recovers retained Aisha16b12b2/6fd9fa9 raw Kitty/Win32 lookup policies and persistent realm adapters without replacing newer task292 InputEvent/IME/clipboard fixes. It is accepted at eea9b36, preserved on archive/Herodotus16-keyboard-reviewed, and integrated through the canonical helper into main40f1798. Current source is actual main, not legacy P: master.

Read docs/keyboard-recovery343.md, docs/keyboard-map-lookup.md and results/keyboard-recovery343. Portable rawValue/HostValue/Hint and event lookup methods retain exact source conversion order, hints, strict Number equality, error identity and UTF-16. Browser tables persist across events/modes and reentry; production JS contains only attributed literals and two conversion operations. No engine delegation, unsafe code or dependency was added.

Fresh scoped results include original Kitty165tests/173assertions/508249comparisons, legacy61/156/251452, Win3264/13904; Node and all3 browser realms10224Kitty+592Win32 cases each; DOM15x3 and InputEvent4x3; native raw Kitty4+Win325 plus existing library/dispatch checks; scoped core/WASM Clippy. Reviewer independently checked source/fixture drift, all23 implementation/fixture hashes, browser artifact hashes, own rustfmt and JS syntax, without fresh Cargo/browser allocation.

Canonical40f1798 passed1839 native core/web tests across339 summaries, zero failed/ignored, plus affected/dependent/default workspace and applicable WASM library checks. Exact fullfmt passed. Confirmed Arjuna receipt-only1cf6be3 was included;344 was still in review at freeze and was not included. No final all-feature/release/native Windows/Linux/macOS or full port-equivalence claim.

## Remaining input work

Task319 and criterion471 still require complete input/accessibility closure. A concrete pending boundary is an admitted InputEvent whose second data read returns null, undefined, an object or another non-string. Source passes raw data through CoreService; Rust currently expects UTF-16 text. Preserve disableStdin, user-input, scroll, log, callback and exception ordering. Do not coerce raw values to empty strings. See docs/handoffs/diponegoro16.md and docs/input-event-dispatch.md before extending this boundary.

Raw event fields outside declared primitive DOM types also remain task319. Existing generated source fixtures and all their assertions must remain intact. The added32 Win32 changing-key-to-empty traces were a disproved bug hypothesis: actual source uses codePointAt(0)||0, and source/Rust match. No production codepoint change was made.

The full WASM library run was24/25 in each engine, while both raw keyboard tests passed. The failure is viewport constructor getter trace4expected versus10actual at viewport_browser_tests.rs116. Logs are durably retained, not normalized. Post432 routed317; Amrapali now investigates354. Source full Terminal legitimately includes overview-ruler reads, and initial hidden ruler width reading may be a separate real producer bug. Do not assume the old4-entry fixture proves a pure viewport defect.

## Integration and cleanup

The new temporary tree was created at O:/xterm-rust-swarm-16/tmp/worktrees/Herodotus16 and safely moved to O:/xterm-rust-swarm-16/tmp/H to avoid Windows rustfmt OS206. Source reference is a physical copy, never a junction. Node realm runner now removes its generated bindings in finally. Browser runners close servers/browsers in finally.

The canonical helper removed the private regular target. Final preflight scanned6462 files/1006 directories with zero links, found no owned node/compiler/browser processes and confirmed a clean tracked tree. The173 untracked generated browser/Node receipt files are explicitly whitelisted duplicates of retained evidence. Whole-worktree removal follows this committed preflight; post409 records its final outcome. Preserve branches and tracked source/evidence; do not remove historical worktrees, original reference, credentials or runtime data.
