// Copyright (c) 2022 The xterm.js authors. All rights reserved. MIT license.
// TaskQueue.ts chooses the idle implementation once, at module evaluation.
const sourceHasIdle = 'requestIdleCallback' in globalThis;
const closureGcSupported = typeof WeakRef === 'function' && typeof FinalizationRegistry === 'function';
export function sourceIdleSupported() { return sourceHasIdle; }

export function requireQueueRuntime() {
  // wasm-bindgen releases JS-owned closures through FinalizationRegistry. The
  // source's ES2021 browser path also uses WeakRef in its scrolling helpers.
  if (!closureGcSupported) {
    throw new Error('Browser task queues require ES2021 weak-reference support');
  }
}
export function warnQueue(message) { console.warn('xterm.js: ' + message); }
