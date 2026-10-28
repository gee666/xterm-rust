# Core engine contract and acceptance

Coleman owns `crates/xterm-core/src/terminal/` under task 28. Grayson owns Buffer/BufferSet. Max owns EscapeSequenceParser. The terminal never invokes the upstream JavaScript engine.

## Planned shared API

`xterm_core::terminal::CoreTerminal` owns parser, decoders and shared state. `CoreTerminal::new(cols: usize, rows: usize, scrollback: usize) -> Result<Self, TerminalError>` validates dimensions. `write_utf16(&mut self, data: &[u16])`, `write_bytes(&mut self, data: &[u8])` and `resize(&mut self, cols: usize, rows: usize)` return `Result<(), TerminalError>`. Writes are synchronous until async handler support is connected. No accepted write may silently discard data. `state(&self) -> Ref<'_, TerminalState>` provides borrowed live state; callers must release that borrow before writes. State holds buffers, attributes, modes, charset and pending events. `drain_events(&mut self) -> Vec<TerminalEvent>` preserves event ordering. `input(&mut self, data: &[u16], was_user_input: bool)` is a data-event path, never display echo. Public APIs are not complete until the source mapping and tests below pass.

Snapshots and selection/search/serialize adapters must borrow the live buffer and not maintain divergent copies. Browser adapters must keep parser replies separate from incoming display writes.

## Required acceptance paths

All source branches in `common/InputHandler.ts`, `common/CoreTerminal.ts`, `common/input/WriteBuffer.ts` and relevant `Async.ts` routines must map to Rust files and assertions. A passing subset does not satisfy these gates.

### Printing and controls

Test ASCII, astral scalars, lone UTF-16 surrogates, split UTF-8 and surrogate pairs at every boundary, BOM and malformed decoder output. Test normal/fullwidth/combining/grapheme-join cells, right-margin wrap enabled/disabled, wide cells at last column, insertion, erase attributes, charset remapping and REP predecessor clearing. Compare cells including packed fg/bg, combined UTF-16 data and extended attributes. Test BEL, LF/VT/FF, CR, BS, HT, SO/SI, IND/NEL/HTS, reverse index at each scroll boundary. Check dirty ranges, cursor, linefeed, accessibility and scroll event order.

### CSI, ESC, OSC and DCS

Inventory every registration in InputHandler's constructor. Check zero/default/multiple/colon parameters, prefixes and intermediates separately. Cover cursor movement with origin mode/margins; selective/nonselective erase; insert/delete cells/rows/columns and fullwidth repairs; scroll regions and left/right scrolling; tabs; SGR reset, styles and underline/color variants; save/restore cursor/attributes/charset; standard/private set/reset/query modes; alt-screen and cursor save variants; Kitty stack operations; device/status/version/window replies; protected cells; title/icon stacks; indexed/special colors; OSC8 hyperlinks; DECRQSS. Unknown sequences must retain upstream fallback behavior. Input abort, reset and disposal during a partially received string must not leak payload to display.

### Queue, API and lifecycle

Faithfully port WriteBuffer assertions for FIFO/callback order, sync nesting limits, user-input fast path, the exact 50,000,000-unit overflow comparison, empty string versus empty byte-array behavior, offset compaction, 12ms slicing and parsed-event ordering. Pending handlers must stop input without replay, resume false into the next handler, and preserve reset/disposal/rejection behavior. Validate write/resize flush ordering, dimension minimums, scroll-to-line bounds, clear semantics, reset retained options, options updates, data/binary events and disabled stdin. Native synchronous APIs must document language differences, not claim JavaScript event-loop parity.

### Evidence

Retain original assertions and publish discovered/mapped/executed/passed/failed/skipped/unrun counts. Run deterministic whole-terminal differential traces and seeded randomized operation traces comparing full cells, cursor, margins, modes, both buffers, scrollback, events and replies. Retain failing seeds and chunkings under results. Run formatter, warning-free Clippy, workspace tests and independent integration review. Missing handlers and unrun checks remain explicit gaps; overall equivalence is NOT PROVEN.
