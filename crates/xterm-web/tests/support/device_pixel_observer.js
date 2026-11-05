// Copyright (c) 2022-2026 The xterm.js authors. All rights reserved. MIT license.
// Shared host inputs for unchanged-source and Rust implementations.
export function observerTraces(factory) {
  const results = [];
  const cases = ['normal', 'unsupported', 'observe-throw', 'observe-getter-throw',
    'construct-throw', 'callback-throw', 'disconnect-throw', 'box-getter-throw',
    'getters', 'zero', 'inherited', 'find', 'dispose-reentry', 'sync-unsupported',
    'sync-supported', 'coercion', 'width-throw', 'retained-predicate',
    'empty-box', 'primitive-box', 'null-box', 'has-proxy', 'short-circuit', 'bigint',
    'observe-sync-unsupported-throw'];
  for (const name of cases) {
    const trace = [], sentinel = { sentinel: name }, element = {}, other = {};
    let notify, disposable, predicate, reentered = false;
    const parent = { ResizeObserver: class {
      constructor(callback) {
        trace.push('construct'); notify = callback;
        if (name === 'construct-throw') throw sentinel;
        if (name === 'sync-unsupported') callback([{ target: element }]);
        if (name === 'sync-supported') callback([{target: element, devicePixelContentBoxSize: [{inlineSize: 3, blockSize: 4}]}]);
      }
      get observe() {
        trace.push('get-observe');
        if (name === 'observe-getter-throw') throw sentinel;
        return function(target, options) {
          trace.push(['observe', target === element, options.box, this instanceof parent.ResizeObserver]);
          if (name === 'observe-sync-unsupported-throw') {
            notify([{target: element}]);
            throw sentinel;
          }
          if (name === 'observe-throw') throw sentinel;
        };
      }
      disconnect() {
        trace.push('disconnect');
        if (name === 'disconnect-throw') throw sentinel;
        if (name === 'dispose-reentry' && !reentered) { reentered = true; disposable.dispose(); }
      }
    }};
    const report = error => trace.push(error === sentinel ? 'sentinel' : error.name);
    try {
      disposable = factory(element, parent, function(width, height) {
        'use strict';
        trace.push(['callback', this === undefined,
          typeof width === 'bigint' ? `${width}n` : width,
          typeof height === 'bigint' ? `${height}n` : height]);
        if (name === 'callback-throw') throw sentinel;
      });
      const good = {target: element, devicePixelContentBoxSize: [{inlineSize: 12.5, blockSize: 27.25}]};
      let entries = [good];
      if (name === 'normal') {
        notify([{target: other, get devicePixelContentBoxSize() { throw sentinel; }}]);
        notify([{target: other}, good, {target: element, devicePixelContentBoxSize: []}]);
      } else if (name === 'unsupported') {
        notify([{target: element}]);
        notify([{target: element}]);
      } else if (name === 'box-getter-throw') {
        entries = [{target: element, get devicePixelContentBoxSize() { throw sentinel; }}];
      } else if (name === 'getters') {
        let read = 0;
        entries = [{get target() { trace.push('target'); return element; },
          get devicePixelContentBoxSize() {
            trace.push('box' + ++read);
            return [{get inlineSize() { trace.push('width'); return 7.25; },
              get blockSize() { trace.push('height'); return 10.5; }}];
          }}];
      } else if (name === 'zero') {
        for (const [width, height] of [[0,1], [1,0], [-1,2], [2,-1], [NaN,1], [1,NaN]])
          notify([{target: element, devicePixelContentBoxSize: [{inlineSize: width, blockSize: height}]}]);
        entries = [];
      } else if (name === 'inherited') {
        entries = [Object.assign(Object.create({devicePixelContentBoxSize: good.devicePixelContentBoxSize}), {target: element})];
      } else if (name === 'find') {
        entries = {find(predicate) { trace.push('find'); return predicate(good) ? good : undefined; }};
      } else if (name === 'coercion') {
        entries = [{target: element, devicePixelContentBoxSize: [{
          inlineSize: {valueOf() { trace.push('coerce-width'); return 3; }, toJSON() { return 'raw-width'; }},
          blockSize: '4.25'}]}];
      } else if (name === 'width-throw') {
        entries = [{target: element, devicePixelContentBoxSize: [{
          inlineSize: {valueOf() { throw sentinel; }},
          get blockSize() { trace.push('read-height-before-coerce-width'); return 4; }}]}];
      } else if (name === 'retained-predicate') {
        entries = {find(fn) { predicate = fn; return good; }};
      } else if (name === 'empty-box' || name === 'primitive-box' || name === 'null-box') {
        entries = [{target: element, devicePixelContentBoxSize:
          name === 'empty-box' ? [] : name === 'primitive-box' ? 'raw' : null}];
      } else if (name === 'has-proxy') {
        entries = [new Proxy(good, {has(target, key) { trace.push('has-' + key); return Reflect.has(target, key); }})];
      } else if (name === 'short-circuit') {
        entries = [{target: element, devicePixelContentBoxSize: [{inlineSize: 0,
          blockSize: {valueOf() { throw sentinel; }}}]}];
      } else if (name === 'bigint') {
        entries = [{target: element, devicePixelContentBoxSize: [{inlineSize: 3n, blockSize: 4n}]}];
      }
      try { notify(entries); } catch (error) { report(error); }
      if (predicate) trace.push(['retained', predicate(good), predicate({target: other})]);
      try { disposable.dispose(); } catch (error) { report(error); }
      try { disposable.dispose(); } catch (error) { report(error); }
      // Source disposal disconnects only. Already queued deliveries still run.
      try { notify([good]); } catch (error) { report(error); }
    } catch (error) { report(error); }
    results.push({ name, trace });
  }
  return JSON.stringify(results);
}

export async function nativeObserver(factory) {
  const iframe = document.createElement('iframe');
  document.body.appendChild(iframe);
  const win = iframe.contentWindow, doc = win.document;
  const element = doc.createElement('div');
  element.style.cssText = 'width:123.25px;height:37.5px';
  doc.body.appendChild(element);
  const Original = win.ResizeObserver;
  let delivery, timer, disposable, constructors = 0, disconnects = 0;
  const callbacks = [], sizes = [];
  const next = () => new Promise((resolve, reject) => {
    delivery = resolve;
    timer = setTimeout(() => reject(new Error('ResizeObserver delivery deadline')), 3000);
  });
  win.ResizeObserver = class extends Original {
    constructor(callback) {
      constructors++;
      super(entries => {
        const box = entries[0].devicePixelContentBoxSize?.[0];
        sizes.push(box ? [box.inlineSize, box.blockSize] : null);
        callback(entries);
        clearTimeout(timer);
        delivery();
      });
    }
    disconnect() { disconnects++; super.disconnect(); }
  };
  try {
    const first = next();
    disposable = factory(element, win, (w, h) => callbacks.push([w, h]));
    await first;
    // An unsupported implementation disconnects itself on first delivery.
    if (sizes[0] !== null) {
      const resized = next();
      element.style.width = '141.75px';
      await resized;
    }
    disposable.dispose();
    disposable.dispose();
    return JSON.stringify({constructors, disconnects, callbacks, sizes});
  } finally {
    clearTimeout(timer);
    if (disposable) disposable.dispose();
    iframe.remove();
  }
}

export function dropProbe() {
  let disconnects = 0, calls = 0, notify;
  const element = {};
  return {element, parent: {ResizeObserver: class {
    constructor(callback) { notify = callback; }
    observe() {}
    disconnect() { disconnects++; }
  }}, callback() { calls++; },
  deliver() { notify([{target: element, devicePixelContentBoxSize: [{inlineSize: 1, blockSize: 2}]}]); },
  snapshot() { return JSON.stringify({disconnects, calls}); }};
}

export function oracleTraces() { return observerTraces(globalThis.sourceObserveDevicePixelDimensions); }
export function oracleNative() { return nativeObserver(globalThis.sourceObserveDevicePixelDimensions); }
