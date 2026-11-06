// Copyright (c) 2018-2026 The xterm.js authors. All rights reserved. MIT license.
import {releaseTestContext} from './webgl_context.js';
export async function nativeContextCycle(factory) {
  const parent = document.createElement('div'); document.body.appendChild(parent);
  const trace = [];
  const host = {rows: () => 3, log: kind => trace.push(kind), restore: () => trace.push('restore'),
    cursor() {}, redraw: (_s, _e, sync) => trace.push(sync ? 'sync-redraw' : 'redraw'),
    loss: () => trace.push('context-loss')};
  let controller, deadline, frame;
  const pending = [];
  function event(canvas, type) {
    return new Promise(resolve => {
      const handler = value => { canvas.removeEventListener(type, handler); resolve(value); };
      canvas.addEventListener(type, handler);
      pending.push(() => canvas.removeEventListener(type, handler));
    });
  }
  try {
    controller = factory(document, true, host, parent);
    const canvas = controller.canvas, gl = controller.gl;
    const extension = gl.getExtension('WEBGL_lose_context');
    if (!extension) throw new Error('WEBGL_lose_context unavailable: actual loss/restoration unrun');
    const timeout = new Promise((_, reject) => { deadline = setTimeout(() => reject(new Error('Native context restoration deadline')), 10000); });
    const cycle = (async () => {
      const lost = event(canvas, 'webglcontextlost');
      extension.loseContext();
      const lostEvent = await lost;
      const restored = event(canvas, 'webglcontextrestored');
      // Chromium/WebKit require the lost-event dispatch to finish before a
      // restore request. A real animation frame establishes that boundary.
      await new Promise(resolve => { frame = requestAnimationFrame(resolve); });
      extension.restoreContext();
      await restored;
      // This filters only observer resize notifications. Context callbacks and
      // the final asynchronous restoration redraw remain individually asserted.
      return JSON.stringify({trace: trace.filter(value => value !== 'sync-redraw'),
        prevented: lostEvent.defaultPrevented, lost: gl.isContextLost(), error: gl.getError()});
    })();
    return await Promise.race([cycle, timeout]);
  } finally {
    clearTimeout(deadline);
    cancelAnimationFrame(frame);
    for (const remove of pending) remove();
    try { controller?.cleanup(); }
    finally { parent.remove(); releaseTestContext(controller?.gl); }
  }
}
export function oracleContextCycle() { return nativeContextCycle(globalThis.sourceContextFactory); }

// Rust ownership safety check, not a source leak-equivalence assertion.
export function dropProbe(callback) {
  const Original = window.ResizeObserver, set = window.setTimeout, clear = window.clearTimeout;
  let retained, deliver, target, disconnects = 0, clears = 0, pending = false;
  window.ResizeObserver = class {
    constructor(callback) { deliver = callback; }
    observe(element) { target = element; }
    disconnect() { disconnects++; }
  };
  window.setTimeout = function(value, delay) {
    if (delay !== 3000) throw new Error('Unexpected drop-probe timer');
    retained = value; pending = true; return 431;
  };
  window.clearTimeout = function(handle) {
    if (handle !== 431) throw new Error('Unexpected drop-probe handle');
    clears++; pending = false;
  };
  try {
    callback(canvas => canvas.dispatchEvent(new Event('webglcontextlost', {cancelable: true})), () => {
      retained();
      deliver([{target, devicePixelContentBoxSize: [{inlineSize: 5, blockSize: 6}]}]);
    });
    return JSON.stringify({disconnects, clears, pending});
  } finally { window.ResizeObserver = Original; window.setTimeout = set; window.clearTimeout = clear; }
}
