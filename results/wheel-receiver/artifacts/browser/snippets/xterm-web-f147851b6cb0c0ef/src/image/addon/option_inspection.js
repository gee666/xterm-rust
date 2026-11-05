// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
export function imageOptionInspection(reader) {
  const view = {};
  // Keep source declaration order for inspection and serialization. Values are
  // read from Rust on every access, never copied into a second option authority.
  for (const key of [
    'enableSizeReports', 'pixelLimit', 'sixelSupport', 'sixelScrolling',
    'sixelPaletteLimit', 'sixelSizeLimit', 'storageLimit', 'showPlaceholder',
    'iipSupport', 'iipSizeLimit', 'kittySupport', 'kittySizeLimit'
  ]) {
    Object.defineProperty(view, key, {
      enumerable: true,
      get() { return reader.get(key); }
    });
  }
  return Object.freeze(view);
}
