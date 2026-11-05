// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
// Keep this Promise continuation in the JS heap: the browser can collect an
// unsettled callback without leaking a wasm-bindgen Closure. All drawing and
// placeholder policy remains Rust. A late result intentionally matches source.
export function assignPlaceholderBitmap(slot, promise) {
  promise.then(bitmap => slot.bitmap = bitmap);
}
