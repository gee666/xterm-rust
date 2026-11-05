
export function modesObject(cursor, keypad, paste, insert, mouse, origin, reverse, focus, show, sync, win32, wrap) {
  return { applicationCursorKeysMode: cursor, applicationKeypadMode: keypad,
    bracketedPasteMode: paste, insertMode: insert, mouseTrackingMode: mouse,
    originMode: origin, reverseWraparoundMode: reverse, sendFocusMode: focus,
    showCursor: show, synchronizedOutputMode: sync, win32InputMode: win32,
    wraparoundMode: wrap };
}
