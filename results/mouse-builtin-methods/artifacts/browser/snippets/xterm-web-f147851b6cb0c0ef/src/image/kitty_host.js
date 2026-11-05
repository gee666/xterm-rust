// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
// DOM event closures stay in the JS heap so an abandoned image can be collected.
// Protocol, pixel conversion, placement and terminal state all remain Rust-owned.
export function pngBitmapFallback(blob) {
  const url = URL.createObjectURL(blob);
  const image = new Image();
  return new Promise((resolve, reject) => {
    image.addEventListener('load', () => {
      URL.revokeObjectURL(url);
      const canvas = window.document.createElement('canvas');
      canvas.width = image.width | 0;
      canvas.height = image.height | 0;
      canvas.getContext('2d')?.drawImage(image, 0, 0);
      createImageBitmap(canvas).then(resolve).catch(reject);
    });
    image.addEventListener('error', () => {
      URL.revokeObjectURL(url);
      reject(new Error('Failed to load image'));
    });
    image.src = url;
  });
}
