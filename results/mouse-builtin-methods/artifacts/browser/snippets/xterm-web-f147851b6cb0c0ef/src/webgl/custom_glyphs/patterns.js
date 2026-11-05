// Copyright (c) 2021 The xterm.js authors. All rights reserved. MIT license.
// Only DOM construction/property operations live here. Glyph data, colors and drawing stay Rust.
const hasOffscreenFactory = typeof OffscreenCanvas !== 'undefined';
export function customGlyphHasOffscreenFactory() { return hasOffscreenFactory; }
export function customGlyphOffscreenCanvas(width, height) { return new OffscreenCanvas(width, height); }
export function customGlyphDomCanvas(width, height) {
  const canvas = document.createElement('canvas');
  canvas.width = width;
  canvas.height = height;
  return canvas;
}
export function customGlyphPatternContext(canvas) { return canvas.getContext('2d'); }
export function customGlyphPatternImage(width, height) { return new ImageData(width, height); }
export function customGlyphPatternByte(image, index, value) { image.data[index] = value; }
export function customGlyphPatternPut(context, image) { context.putImageData(image, 0, 0); }
export function customGlyphPatternCreate(context, canvas) { return context.createPattern(canvas, null); }
export function customGlyphPatternTransform(pattern, x, y) {
  // Method lookup precedes argument construction, including their distinct throws.
  if (x !== 0 || y !== 0) {
    pattern.setTransform(new DOMMatrix().translateSelf(x, y));
  } else {
    pattern.setTransform(new DOMMatrix());
  }
}
