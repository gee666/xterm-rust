// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
// Read-only projection of the actual original engine to acceptance schema1.
// Packed bitfields use unsigned32 representation. Source undefined optional
// charset/cursor overrides map to Rust Option::None JSON null, not an omission.
export function sourceSnapshot(terminal, builtinCharsets) {
  const units = text => Array.from({ length: text.length }, (_, index) => text.charCodeAt(index));
  const optional = value => value === undefined ? null : value === null ? { explicitJsNull: true } : value;
  function charset(value) {
    if (value === undefined) return null;
    if (value === null) return { explicitJsNull: true };
    const builtin = [...'04567=ACEHKQRYZ'].find(id => builtinCharsets[id] === value) ?? null;
    const entries = Object.entries(value).map(([key, text]) => {
      if (key.length !== 1 || typeof text !== 'string') throw new Error('Non-string or multi-unit custom charset entry requires an explicit schema');
      return [key.charCodeAt(0), units(text)];
    }).sort((a, b) => a[0] - b[0]);
    return { builtin, entries };
  }
  function attributes(value) {
    return { fg: value.fg >>> 0, bg: value.bg >>> 0, ext: value.extended.ext >>> 0,
      urlId: value.extended.urlId, hasPayload: value.extended.payload !== undefined };
  }
  function line(value) {
    if (value === undefined) return null;
    return { isWrapped: value.isWrapped, cells: Array.from({ length: value.length }, (_, index) => {
      const cell = value.getCell(index);
      return { content: cell.content >>> 0, fg: cell.fg >>> 0, bg: cell.bg >>> 0,
        ext: cell.extended.ext >>> 0, urlId: cell.extended.urlId, hasPayload: cell.extended.payload !== undefined,
        chars: units(cell.getChars()), width: cell.getWidth(), code: cell.getCode() };
    }) };
  }
  function buffer(value) {
    return { cursorX: value.x, cursorY: value.y, baseY: value.ybase, viewportY: value.ydisp,
      scrollTop: value.scrollTop, scrollBottom: value.scrollBottom, maxLength: value.lines.maxLength,
      tabs: Object.entries(value.tabs).map(([key, present]) => {
        const number = Number(key);
        if (present !== true || !Number.isSafeInteger(number) || number < 0) throw new Error('Non-set tab entry requires an explicit schema');
        return number;
      }).sort((a, b) => a - b),
      saved: { x: value.savedX, y: value.savedY, attributes: attributes(value.savedCurAttrData),
        charset: charset(value.savedCharset), charsets: Array.from(value.savedCharsets, charset),
        glevel: value.savedGlevel, origin: value.savedOriginMode, wraparound: value.savedWraparoundMode },
      markers: value.markers.map(marker => ({ id: marker.id, line: marker.line, isDisposed: marker.isDisposed })),
      lines: Array.from({ length: value.lines.length }, (_, index) => line(value.lines.get(index))) };
  }
  const core = terminal._core;
  const handler = core._inputHandler;
  const modes = core.coreService.decPrivateModes;
  const kitty = core.coreService.kittyKeyboard;
  const currentCharset = core._charsetService;
  return { schema: 1, cols: core.cols, rows: core.rows, disposed: core._store.isDisposed,
    active: core.buffers.active === core.buffers.alt ? 'alternate' : 'normal',
    normal: buffer(core.buffers.normal), alternate: buffer(core.buffers.alt),
    attributes: attributes(handler.getAttrData()), title: units(handler._windowTitle), iconName: units(handler._iconName),
    charset: charset(currentCharset.charset), charsets: Array.from(currentCharset.charsets, charset),
    glevel: currentCharset.glevel, unicodeVersion: core.unicodeService.activeVersion,
    modes: { insert: core.coreService.modes.insertMode, applicationCursorKeys: modes.applicationCursorKeys,
      applicationKeypad: modes.applicationKeypad, bracketedPaste: modes.bracketedPasteMode,
      colorSchemeUpdates: modes.colorSchemeUpdates, cursorBlink: optional(modes.cursorBlink),
      cursorStyle: optional(modes.cursorStyle), origin: modes.origin, reverseWraparound: modes.reverseWraparound,
      sendFocus: modes.sendFocus, synchronizedOutput: modes.synchronizedOutput,
      win32Input: modes.win32InputMode, wraparound: modes.wraparound,
      cursorInitialized: core.coreService.isCursorInitialized, cursorHidden: core.coreService.isCursorHidden,
      mouseProtocol: core.mouseStateService.activeProtocol, mouseEncoding: core.mouseStateService.activeEncoding },
    kitty: { flags: kitty.flags, mainFlags: kitty.mainFlags, altFlags: kitty.altFlags,
      mainStack: [...kitty.mainStack], altStack: [...kitty.altStack] } };
}
