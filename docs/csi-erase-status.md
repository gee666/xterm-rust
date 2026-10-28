# CSI erase status

Overall equivalence remains NOT PROVEN. Task 111 implements ED/EL/ECH/DECSCA in the actual Rust terminal engine and parser. `docs/acceptance/csi-erase.md` defines the source-specific acceptance checklist.

## Observed checks

The unchanged `InputHandler.test.ts` baseline ran through `node source_repo/bin/test_unit.js out-esbuild/common/InputHandler.test.js --reporter=json`. All 194 tests passed, with zero failed or pending. `results/csi-erase/original-input-handler.json` retains raw output, including two terminal report sequences printed before the JSON reporter object. Do not treat those leading report bytes as JSON syntax errors or discard the raw log.

Before the shared disk pause, eight native Rust tests passed. Four preserve the first common erase cases, one replays 337 source traces with 3,937 full-state snapshots and 2,140 writes, and three check callback-time marker/trim state and original error identity. The seed is `0xe2a5e111`. The differential compares both full buffers, packed cells/extended attributes, cursor/margins/modes, markers, user-scrolling state and ordered events. Source provenance and fixture hashes are in `results/csi-erase/oracle.json`. Artaxerxes independently executed the existing binaries and inspected source while rebuilds were paused.

The differential found and retained an actual ED3 event mismatch. Rust emitted a public scroll event, but the source only emits InputHandler's private notification. `TerminalEvent::InputScroll` now preserves that distinction, and headless explicitly ignores it. Xavier agreed to wire browser selection refresh and queued viewport synchronization. These browser effects need actual integration evidence; an enum alone does not complete them.

## Work added during the disk pause

The initial four-case inventory was not complete. Static review found 22 applicable common cases, six Playwright content cases and five legacy VT fixtures. Additional native assertion code is now in `csi_erase_cases.rs` and `csi_erase_playwright_assertions.rs`. These preserve wide-character repairs, pending-wrap cursor positions, all 29 following sequences, normal/resized-alt ED3, DECSCA and protection replies. The six browser content cases run against Rust state only and do not replace real browser execution.

`scripts/csi-erase-mapping.mjs` accounts for all 22 common and six browser cases, including a partial negative private-cursor ECH case assigned to task 152. The script is source-ready but must execute after the disk pause. It must reject missing mappings, not infer completeness from a small green suite.

`scripts/csi-erase-fixtures.mjs` preserves all five source inputs and expected text files and compares the unchanged JS engine using explicit ONLCR mapping. It is not yet executed. The original `Terminal2.test.ts` skips all PTY cases on Windows and explicitly skips EL because its pending-wrap behavior differs from xterm. Retain that legacy mismatch while still comparing Rust with the real unchanged JS engine for EL. An ONLCR replay is not a native PTY execution claim.

New assertion code and an explicit private InputScroll regression have not yet been compiled after the disk pause. Earlier binary results must not be relabeled as results for these additions. No ignored tests, weakened source expectations or invented pass counts were added to clear the gap.

## Remaining gates

- Compile and execute every newly added native assertion and all five VT replays, then record current debug/release, formatting and warning-free Clippy receipts.
- Close the negative raw-cursor precondition through task 152 without pre-clamping test input or half-wired duplicate state.
- Close terminal-level reentry during marker disposal through task 176. Releasing Buffer borrows alone does not prove synchronous public reset/reentry is safe while a terminal action holds state.
- Link and execute the actual browser InputScroll selection/viewport behavior under criterion 403.
- Obtain final independent review and retain actual native Windows/Linux/macOS and browser integration results through the project quality tasks.

The disk pause was coordinated by Chloe and Kassa after free space fell below the one-GiB reserve. Source and evidence remain intact. Resource pressure is a local coordination issue, not an external blocker or permission to declare completion.
