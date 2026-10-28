# CSI erase acceptance

Task 111 covers `source_repo/src/common/InputHandler.ts` methods `selectProtected`, `_eraseInBufferLine`, `_resetBufferLine`, `eraseInDisplay`, `eraseInLine` and `eraseChars`. Registrations must run through the actual Rust parser and terminal engine. Neighboring cursor and edit behavior belongs to task 107.

## Cells, protection and wrapped rows

- DECSCA `CSI Ps " q` sets protection only for 1, clears it for 0/2 and leaves it unchanged for other values. Missing/default/zero parameters follow the original Params representation. No unrelated attribute changes are allowed.
- EL modes 0/1/2 erase right/left/all. Mode 1 includes the cursor cell. Missing mode is 0. Invalid modes still restrict the cursor and dirty its row. Selective forms retain protected cells; normal forms erase them.
- EL0 clears wrap only at column zero. EL1 never clears wrap. EL2 always clears wrap, including selectively preserved protected content.
- ED0 erases from the cursor to the end of its row and resets every later row. ED1 includes the cursor, resets earlier rows backwards, clears the current wrap and uses the source's absolute `y + 1` next-row wrap call when the cursor reaches the right edge. That absolute call must not silently gain a history-base offset.
- ED2 resets rows backwards when scrollOnEraseInDisplay is false. When true, it scrolls once for each row up to the last nonempty row, even for a selective command; protected text does not prevent this source branch. Empty screens do not scroll. Background-colored empty cells do not count as content.
- ECH defaults zero/missing count to one. It restricts pending-wrap X to the last column, works inside/outside margins, leaves X/Y and wrapped flags unchanged after restriction, and does not honor protection.
- Erased cells retain only BCE background color. Foreground, styles, protection, underline metadata and hyperlinks are cleared. Wide character boundary repairs and combined strings must match BufferLine behavior. Compare packed full cells, not just translated text.
- Cover cursor positions 0, interior, last column, pending wrap and beyond columns; empty/missing rows; all supported and unsupported modes; default/zero/maximum parameter values; Unicode/wide/combined/protected cells; normal/alternate buffers; nonzero history and margins.

## Markers, history and callbacks

- `_resetBufferLine` fills cells before clearing markers, and clears wrap only after marker callbacks finish. Range erasure alone does not dispose markers. Protected cells can remain while their whole-row marker is disposed, as upstream does.
- Marker callbacks may inspect/mutate the buffer without a live Buffer RefMut held by the command. A callback exception stops immediately, retaining disposed marker state, filled cells and the old wrapped flag. Preserve the original error cause through TerminalError and parser errors.
- ED3 trims exactly `lines.length - rows` when positive. Trim observers see the already-trimmed lines and old bases. Then ybase/ydisp saturate at zero. Normal-buffer ED3 clears isUserScrolling; alternate-buffer ED3 must leave that normal viewport flag unchanged. No history means no trim or internal scroll notification.
- Distinguish InputHandler's private scroll notification from the public terminal scroll event. Source headless does not forward ED3's notification. Browser integration must refresh selection and queue viewport synchronization through that private notification, not invent a public onScroll event.
- Dirty row and render event ranges, marker disposal order, trim events, cursor events and scroll events must match source order. Do not hold list/line/terminal-state borrows across user callbacks.

## Evidence

`csi_erase_upstream` preserves the first four InputHandler erase cases: eraseInLine, eraseInLine reflow, ED2 with scrollOnEraseInDisplay turned on, and eraseInDisplay. `csi_erase_cases` covers the remaining wide-cell, pending-wrap, following-sequence, ED3, DECSCA and protection-query assertions. There are 22 applicable common cases, with the negative private-coordinate half of ECH explicitly tracked by task 152. `csi_erase_playwright_assertions` maps six original browser content cases to native assertions, not actual browser execution. The source-derived mapping script must account for every case and partial/unrun status. Run the unchanged InputHandler suite as the original baseline.

Preserve all five erase VT input/expected files: t0054-ECH, t0055-EL, t0056-ED, t0057-ED3 and t0091-alt_screen_ED3. The original Terminal2 suite skips all PTY cases on Windows and explicitly skips EL because it differs from xterm at the pending-wrap cursor. Compare Rust with the unchanged JS engine for all five, and retain any legacy text mismatch. An explicit ONLCR replay is not an observed native PTY or browser run.

`scripts/csi-erase-oracle.mjs` uses the actual unchanged headless engine and shared full-state projection. Replay its deterministic fixture with seed `0xe2a5e111` in `csi_erase_differential`. Compare both full buffers, packed attributes/extended data, cursor/margins/modes, marker states, isUserScrolling and live ordered events. Retain all counterexamples, source hashes, counts and exact commands. `csi_erase_edges` exercises callback-time state and original error identity.

Run scoped debug/release tests, rustfmt and warning-free Clippy, then obtain independent source and integration review. Final workspace/native/browser gates remain required. Finite passing traces do not prove whole-terminal equivalence.
