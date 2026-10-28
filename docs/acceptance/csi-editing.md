# CSI cursor and editing acceptance

Task 107 owns cursor motion, tabs, character/line/column edits, scrolling, margins and REP from `source_repo/src/common/InputHandler.ts`. Task 111 owns selective display/line/character erase and character protection. Modes, SGR and device/window replies have separate owners. All handlers must be registered on the actual Rust EscapeSequenceParser, not just exposed as detached methods.

## Cursor and tab behavior

Match CUU/CUD margin stopping when the cursor is inside, above or below the scroll region. Compare origin mode both enabled and disabled, nonzero top margin, pending-wrap X equal to columns, zero/default/large parameters, CNL/CPL resetting X and CHA/HPA/VPA/CUP/HVP one-based coordinates. Preserve the source's restrict-before-relative-move and origin-offset ordering rather than normalizing coordinates. Dirty rows cover old, restricted and final cursor positions exactly.

CHT/CBT do nothing for a pending-wrap cursor. Otherwise they repeatedly use the active buffer's live tab set, defaulting count zero to one and clamping at either edge. TBC parameter zero removes only the current column; three clears all stops; unsupported parameters leave stops unchanged. Margins default top to one and absent/zero/oversized bottom to row count. Only bottom greater than top changes margins and homes the cursor through origin-aware placement.

## Character, column and row edits

ICH/DCH/ECH restrict the cursor first and use erase attributes retaining the current background, not foreground/protection/styles. Preserve BufferLine's wide-boundary repairs and combined/extended cell state. Compare full rows, not translated text alone.

IL/DL have no effect outside scroll margins after restriction. Inside they perform one delete and one insert per requested line in source order and reset X to zero. Compare actual row object identities, marker disposal/shift ordering, deletion/insertion callback snapshots and dirty ranges for counts less than/equal to/greater than region height.

SU/SD leave the cursor untouched, remove/insert within margins, never invent scrollback, and preserve source's different blank fill: SU uses erase attributes; SD uses default attributes. SL/SR and DECIC/DECDC apply only if the cursor lies inside margins, affect every row in that region, clear wrapped flags and leave cursor unchanged. DECIC/DECDC retain source pending-wrap column behavior through BufferLine insertion/deletion semantics. Exercise zero/default/large counts, outside-region cursor, nonzero history base, protected/wide/combined content and background colors.

## Erase and REP

ED/EL/DECSED/DECSEL cover every supported parameter and unknown values, selective protected cells, BCE, existing wrap flags, exact marker cleanup and ED3 scrollback/base/display/user-scroll effects. ED2 honors scrollOnEraseInDisplay with the last nonempty row determining scroll count. Use the detailed source branches unchanged as the reference, including absolute versus viewport row arguments when clearing wraps.

REP reads the parser's carried Unicode join state and the entire preceding grapheme from the current line. It repeats by calling the actual printing path, preserves attributes, Unicode widths, insertion/wrap/scroll effects and works across input chunks. Any intervening sequence clears predecessor state as the outer parser dictates. Test absent predecessor, ASCII, spaces, BMP/astral/combining graphemes, wide cells, right-edge wrap enabled/disabled and repeated REP. Allocation failure must return an error, never panic or silently truncate.

## Evidence

Port all applicable original InputHandler assertions and preserve source mappings/counts. Add deterministic full-terminal JS/Rust traces covering all commands and parameter/default branches, all cells/attributes/wraps, cursor/margins, marker events and dirty ranges. Retain failing traces and seeds. Run debug/release, fmt, warning-free Clippy and independent review. A subset of cursor examples does not satisfy this checklist; record remaining unrun source cases explicitly.
