// Copyright (c) 2018-2026 The xterm.js authors. All rights reserved. MIT license.
const names = ['dimensions', 'height-conversion-error', 'width-reentry', 'row-error',
  'redraw-error', 'loss-expiry', 'loss-restored', 'loss-twice', 'lost-log-error',
  'restored-log-error', 'timeout-error', 'clear-error', 'restore-error',
  'observer-resize', 'window-replacement', 'dispose', 'preserve-true', 'preserve-false',
  'context-unavailable', 'context-undefined', 'context-false', 'context-zero',
  'context-empty', 'context-nan', 'context-bigint-zero', 'context-throw',
  'height-short-circuit', 'dispose-resize-reentry', 'attributes-own'];
export function caseNames() { return JSON.stringify(names); }
export function oracleCase(name) { return contextCase(globalThis.sourceContextFactory, name); }
export function contextCase(factory, name) {
  const trace = [], sentinel = { sentinel: name }, parent = document.createElement('div');
  document.body.appendChild(parent);
  const create = document.createElement, Canvas = HTMLCanvasElement.prototype;
  const getContext = Canvas.getContext, resizeObserver = window.ResizeObserver;
  const setTimeoutOriginal = window.setTimeout, clearTimeoutOriginal = window.clearTimeout;
  const antialiasDescriptor = Object.getOwnPropertyDescriptor(Object.prototype, 'antialias');
  const observers = [], timers = new Map(), listeners = new Map();
  let selected, controller, lossEvent, nextTimer = 0, rowCount = 3, logging = true;
  let restoreError = name === 'restore-error';
  const record = (...args) => { if (logging) trace.push(args); };
  const finish = () => {
    const result = JSON.stringify({name, trace, canvas: selected && [selected.width, selected.height, selected.isConnected]});
    console.log('webgl-context-' + (factory === globalThis.sourceContextFactory ? 'source ' : 'rust ') + result);
    return result;
  };
  const rows = () => { record('rows', rowCount); if (name === 'row-error') throw sentinel; return rowCount; };
  const report = operation => { try { operation(); } catch (error) { record('error', error === sentinel ? 'sentinel' : error.name); } };
  const nativeFactory = function(label) { return class {
    constructor(callback) { this.callback = callback; this.id = observers.length; observers.push(this); if (selected) record('observer.new', label, this.id); }
    observe(element, options) { this.element = element; if (element === selected) record('observe', label, this.id, options.box); }
    disconnect() {
      if (this.element === selected) {
        record('disconnect', label, this.id);
        if (name === 'dispose-resize-reentry' && !this.reentered) {
          this.reentered = true;
          this.deliver(61, 59);
        }
      }
    }
    deliver(width, height) { this.callback([{target: this.element, devicePixelContentBoxSize: [{inlineSize: width, blockSize: height}]}]); }
  }; };
  let iframe;
  Canvas.getContext = function(type, attributes) {
    if (type !== 'webgl2') return getContext.call(this, type, attributes);
    selected = this;
    record('context', attributes.antialias, attributes.depth,
      attributes.preserveDrawingBuffer === undefined ? 'undefined' : attributes.preserveDrawingBuffer,
      Object.keys(attributes), Object.values(Object.getOwnPropertyDescriptors(attributes)).map(value =>
        [value.writable, value.enumerable, value.configurable]));
    const add = this.addEventListener, remove = this.removeEventListener;
    this.addEventListener = function(type, callback, ...rest) {
      if (type.startsWith('webglcontext')) { record('listen', type); listeners.set(type, callback); }
      return add.call(this, type, callback, ...rest);
    };
    this.removeEventListener = function(type, callback, ...rest) {
      if (type.startsWith('webglcontext')) record('unlisten', type, listeners.get(type) === callback);
      return remove.call(this, type, callback, ...rest);
    };
    if (name === 'context-unavailable') return null;
    if (name === 'context-undefined') return undefined;
    if (name === 'context-false') return false;
    if (name === 'context-zero') return 0;
    if (name === 'context-empty') return '';
    if (name === 'context-nan') return NaN;
    if (name === 'context-bigint-zero') return 0n;
    if (name === 'context-throw') throw sentinel;
    return getContext.call(this, type, attributes);
  };
  window.ResizeObserver = nativeFactory('A');
  if (name === 'attributes-own') Object.defineProperty(Object.prototype, 'antialias', {
    configurable: true, set() { throw sentinel; }
  });
  const host = {
    rows,
    log(kind) { record('log', kind); if (name === kind + '-log-error') throw sentinel; },
    restore() { record('restore'); if (restoreError) throw sentinel; },
    cursor() { record('cursor'); },
    redraw(start, end, sync) { record('redraw', start, end, sync); if (name === 'redraw-error') throw sentinel; },
    loss(event) { record('loss', event === lossEvent, event.defaultPrevented); }
  };
  try {
    try {
      controller = factory(document, name === 'preserve-true' ? true : name === 'preserve-false' ? false : undefined, host, parent);
    } catch (error) {
      record('constructor-error', error.name, error.message, error === sentinel);
      return finish();
    }
    window.setTimeout = function(callback, delay) {
      'use strict';
      record('set-timeout', delay, this === undefined);
      if (name === 'timeout-error') throw sentinel;
      const handle = ++nextTimer; timers.set(handle, callback); return handle;
    };
    window.clearTimeout = function(handle) {
      'use strict';
      record('clear-timeout', handle === undefined ? 'undefined' : handle, this === undefined);
      if (name === 'clear-error') throw sentinel;
      timers.delete(handle);
    };
    const lost = () => {
      lossEvent = new Event('webglcontextlost', {cancelable: true});
      report(() => listeners.get('webglcontextlost')(lossEvent));
      record('prevented', lossEvent.defaultPrevented);
    };
    const restored = () => report(() => listeners.get('webglcontextrestored')(new Event('webglcontextrestored')));
    const expire = handle => {
      const callback = timers.get(handle); timers.delete(handle);
      if (callback) report(() => callback());
    };
    if (name === 'dimensions' || name.startsWith('preserve-')) {
      report(() => controller.resize(32, 24));
      report(() => controller.resize(32, 24));
      rowCount = 7;
      report(() => controller.resize('32', 24));
      report(() => controller.resize(12.9, 7.9));
      record('physical', selected.width, selected.height);
    } else if (name === 'height-conversion-error') {
      report(() => controller.resize(29, { valueOf() { record('height.valueOf'); throw sentinel; } }));
    } else if (name === 'width-reentry') {
      report(() => controller.resize({valueOf() { record('width.valueOf'); controller.resize(7, 8); return 11; }}, 9));
    } else if (name === 'row-error' || name === 'redraw-error') {
      report(() => controller.resize(31, 23));
    } else if (name === 'observer-resize') {
      const observer = observers.find(value => value.element === selected);
      report(() => observer.deliver(43, 37));
      report(() => observer.deliver(43, 37));
    } else if (name === 'window-replacement') {
      iframe = document.createElement('iframe'); document.body.appendChild(iframe);
      iframe.contentWindow.ResizeObserver = nativeFactory('B');
      report(() => controller.window(iframe.contentWindow));
      report(() => controller.window(iframe.contentWindow));
      report(() => observers.at(-1).deliver(53, 47));
    } else if (name === 'height-short-circuit') {
      const descriptor = Object.getOwnPropertyDescriptor(Canvas, 'height');
      Object.defineProperty(selected, 'height', {
        get() { record('get.height'); return descriptor.get.call(this); },
        set(value) { record('set.height', value); descriptor.set.call(this, value); }
      });
      report(() => controller.resize(29, 23));
      report(() => controller.resize(29, 23));
      delete selected.height;
    } else if (name === 'dispose' || name === 'dispose-resize-reentry') {
      document.dispatchEvent(new MouseEvent('mousedown'));
      lost();
      report(() => controller.dispose());
      report(() => controller.dispose());
      record('attached', selected.isConnected);
      record('timers', timers.size);
    } else {
      lost();
      if (name === 'loss-twice') { lost(); expire(1); restored(); expire(2); }
      else if (name === 'loss-expiry' || name === 'lost-log-error' || name === 'timeout-error') expire(1);
      else { restored(); expire(1); }
    }
    if (name !== 'dispose' && name !== 'dispose-resize-reentry') report(() => controller.dispose());
    return finish();
  } finally {
    logging = false;
    window.setTimeout = setTimeoutOriginal; window.clearTimeout = clearTimeoutOriginal;
    Canvas.getContext = getContext; window.ResizeObserver = resizeObserver;
    if (antialiasDescriptor) Object.defineProperty(Object.prototype, 'antialias', antialiasDescriptor);
    else delete Object.prototype.antialias;
    try { if (controller) controller.cleanup(); } finally { parent.remove(); iframe?.remove(); }
    document.createElement = create;
  }
}
export function nativeCanvas(factory, preserve) {
  const parent = document.createElement('div'); document.body.appendChild(parent);
  let redraw = 0;
  const host = {rows: () => 3, log() {}, restore() {}, cursor() {}, redraw() { redraw++; }, loss() {}};
  const controller = factory(document, preserve, host, parent);
  try {
    const canvas = controller.canvas, gl = controller.gl;
    controller.resize(8, 6);
    gl.viewport(0, 0, 8, 6); gl.clearColor(0.2, 0.4, 0.6, 1); gl.clear(gl.COLOR_BUFFER_BIT);
    const pixel = new Uint8Array(4); gl.readPixels(3, 2, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixel);
    return JSON.stringify({pixels: [...pixel], size: [canvas.width, canvas.height], redraw,
      attached: canvas.isConnected, sameContext: gl === canvas.getContext('webgl2'),
      preserve: gl.getContextAttributes().preserveDrawingBuffer,
      error: gl.getError()});
  } finally { controller.cleanup(); parent.remove(); }
}
export function oracleNative(preserve) { return nativeCanvas(globalThis.sourceContextFactory, preserve); }
