# Browser input acceptance

This checklist narrows task 33. Source remains read-only. Pure Rust state tests do not prove DOM integration.

## Clipboard and context menu

- Preserve LF and CRLF normalization to CR, standalone CR, empty text, consecutive CR, non-BMP UTF-16 units, and trailing newlines.
- Bracket only when DEC bracketed paste is enabled and ignoreBracketedPasteMode is false. Sanitize every ESC to U+241B only inside brackets. Emit a user-input event, then clear textarea.
- Copy writes text/plain only when clipboardData exists and always prevents default. Paste stops propagation even without clipboardData, then emits only when data exists.
- Context menu moves textarea relative to screen rect by minus 10 CSS pixels, sets 20px dimensions and z-index 1000, focuses, optionally selects word, copies selection text and selects textarea. Check empty selection and moved document.

## IME composition

- compositionstart uses ordered selectionStart/End, falling back to UTF-16 textarea length; saves suffix and clears overlay/dataAlreadySent.
- Updates surround overlay data with U+200E. Deferred selection end samples after native textarea mutation.
- compositionend waits one event turn. Send captured composition before a newly started one; trim unchanged suffix; include non-composition input typed immediately after compositionend. Do not duplicate dataAlreadySent.
- CapsLock, modifier and 229 keydowns suppress regular key processing while composing. Other keydowns commit immediately, cancel delayed send, then continue. A standalone 229 schedules at most one textarea diff; insertion sends first-occurrence replacement, deletion sends DEL, equal-length replacement sends entire new value.
- Verify Korean syllable transitions, Japanese kana/kanji conversion, middle insertion with trailing screen-reader text, non-BMP and unpaired UTF-16 units, cancellation, repeated updates and end/start races. Preserve all eight upstream assertions.
- Overlay uses clamped cursor column, viewport visibility, CSS cell dimensions/font settings, remaining width, RTL direction, minimum 1px textarea dimensions and one deferred remeasurement. Disposal cancels callbacks and listeners.

## Mouse and wheel

- Coordinates subtract bounding rect and integer CSS padding, ceil to one-based cells, clamp viewport and extra selection end column; invalid character metrics return no coordinates. Report coordinates clamp pixels then floor cells to zero-based values.
- Buttons prioritize left/middle/right from buttons bitmask; missing buttons uses button fallback. Ignore unsupported event types/buttons and zero wheel. Alt-required gating strips Alt only for non-wheel reports.
- Protocol changes attach/remove MOVE/WHEEL listeners and global UP/DRAG listeners; use current ownerDocument, remove globals on last release, preserve forced selection and focus ordering.
- Wheel honors custom veto, shift suppression, sensitivities, DPR/cell height, pixel partial accumulator, trackpad threshold 50 and multiplier 0.3, line/page modes and sign changes. Alt buffer produces cursor keys; normal buffer scrolls; touch uses same state and resets on start.
- Reports reject invalid cells/button-action pairs, convert cell coordinates to one-based, debounce grid/pixel moves, apply restrictions, then select binary/default versus data encoding. Verify Alt cursor CSS and blur/dispose.

## Keyboard service and browser events

- Win32 wins over Kitty and legacy, but requires both extension option and DEC mode. Kitty requires extension and protocol flags; repeat/press/release and report-event-types gating match upstream.
- Preserve Mac option-as-meta, application cursor keys, keydown/keyup, keypress/input suppression, custom handler veto, cancellation, focus and disabled stdin ordering.
- Run original Clipboard, CompositionHelper, Mouse and MouseService tests unchanged as baseline. Port assertions with source-to-test mapping. Compare seeded operation traces to JS including UTF-16 offsets and event ordering.
- Chromium, Firefox and WebKit tests must dispatch actual events against Rust-backed terminal. Assert clipboard, IME overlay/output, keyboard protocol priority, pointer reporting, wheel, touch, listener cleanup and disposal. Mark each browser unavailable/unrun explicitly; no completion based only on model tests.
