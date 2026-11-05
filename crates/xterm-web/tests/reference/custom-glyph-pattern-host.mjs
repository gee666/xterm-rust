// Copyright (c) 2026 xterm-rust contributors. MIT license.
// The probes forward to real browser objects; only the unchanged reference draws JS glyphs.
export function sourcePatternHost(ctx, variant) {
  return globalThis.CustomGlyphSource.tryDrawCustomGlyph(
    ctx, '\u2591', 1, 1, 8, 16, 8, 16, 15, 1,
    { error(message) { throw new Error(message); } }, '#112233', variant
  );
}

export function installPatternHostProbe(ctx, kind, color, token) {
  const trace = [];
  const undo = [];
  const matrixToken = Symbol('matrix constructor failure');
  const number = value => Object.is(value, -0) ? '-0' : String(value);
  const replace = (object, key, descriptor) => {
    const previous = Object.getOwnPropertyDescriptor(object, key);
    Object.defineProperty(object, key, { configurable: true, ...descriptor });
    undo.push(() => previous
      ? Object.defineProperty(object, key, previous)
      : Reflect.deleteProperty(object, key));
  };
  const method = (object, key, value) => replace(object, key, { value, writable: true });
  const patchCanvas = canvas => {
    const getContext = canvas.getContext;
    method(canvas, 'getContext', function(type, ...args) {
      trace.push(['context', type]);
      return Reflect.apply(getContext, this, [type, ...args]);
    });
    return canvas;
  };

  ctx.fillStyle = color;
  if (kind === 'canvas-throw' || kind === 'image-throw' || kind === 'unsupported-color') {
    // A legal string-valued host accessor lets us observe the source error phase.
    replace(ctx, 'fillStyle', { get() { return 'rgb(1,2,3)'; } });
  }

  const factoryWasAvailable = typeof globalThis.OffscreenCanvas !== 'undefined';
  if (kind === 'offscreen-toggle') {
    if (factoryWasAvailable) {
      method(globalThis, 'OffscreenCanvas', undefined);
    } else {
      const nativeCreate = document.createElement;
      // This late polyfill returns a genuine main-realm Canvas. The source's
      // already-captured default still uses its original DOM fallback.
      function OffscreenCanvas(width, height) {
        const canvas = Reflect.apply(nativeCreate, document, ['canvas']);
        canvas.width = width;
        canvas.height = height;
        return canvas;
      }
      method(globalThis, 'OffscreenCanvas', OffscreenCanvas);
    }
  }
  const offscreen = globalThis.OffscreenCanvas;
  if (typeof offscreen !== 'undefined') {
    method(globalThis, 'OffscreenCanvas', new Proxy(offscreen, {
      construct(target, args, newTarget) {
        trace.push(['canvas', 'offscreen', ...args]);
        if (kind === 'canvas-throw') throw token;
        return patchCanvas(Reflect.construct(target, args, newTarget));
      }
    }));
  }
  const createElement = document.createElement;
  method(document, 'createElement', function(tag, ...args) {
    if (tag === 'canvas') {
      trace.push(['canvas', 'dom']);
      if (kind === 'canvas-throw') throw token;
    }
    const element = Reflect.apply(createElement, this, [tag, ...args]);
    return tag === 'canvas' ? patchCanvas(element) : element;
  });

  const imageData = globalThis.ImageData;
  method(globalThis, 'ImageData', new Proxy(imageData, {
    construct(target, args, newTarget) {
      trace.push(['image', args.length, ...args.map(v => typeof v === 'number'
        ? v : Object.prototype.toString.call(v))]);
      if (kind === 'image-throw') throw token;
      const image = Reflect.construct(target, args, newTarget);
      const bytes = image.data;
      replace(image, 'data', { get() {
        trace.push(['data']);
        if (kind === 'data-throw') throw token;
        return bytes;
      } });
      return image;
    }
  }));

  const createPattern = ctx.createPattern;
  method(ctx, 'createPattern', function(canvas, repetition) {
    trace.push(['pattern', this === ctx, repetition]);
    if (kind === 'pattern-throw') throw token;
    const pattern = Reflect.apply(createPattern, this, [canvas, repetition]);
    if (pattern) {
      if (kind === 'transform-getter-throw') {
        replace(pattern, 'setTransform', { get() {
          trace.push(['get setTransform']);
          throw token;
        } });
      } else {
        const setTransform = pattern.setTransform;
        method(pattern, 'setTransform', function(matrix) {
          trace.push(['setTransform']);
          return Reflect.apply(setTransform, this, [matrix]);
        });
      }
    }
    return pattern;
  });

  const matrix = globalThis.DOMMatrix;
  method(globalThis, 'DOMMatrix', new Proxy(matrix, {
    construct(target, args, newTarget) {
      trace.push(['matrix', args.length]);
      if (kind === 'transform-getter-throw') throw matrixToken;
      const value = Reflect.construct(target, args, newTarget);
      const translate = value.translateSelf;
      method(value, 'translateSelf', function(x, y) {
        trace.push(['translate', number(x), number(y)]);
        if (kind === 'translate-throw') throw token;
        return Reflect.apply(translate, this, [x, y]);
      });
      return value;
    }
  }));

  return { trace, undo, token, matrixToken, factoryWasAvailable };
}

export function finishPatternHostProbe(probe, ok, result, error) {
  // Restore every instrumented browser member before Rust assertions can fail.
  while (probe.undo.length) probe.undo.pop()();
  const outcome = ok ? ['ok', result]
    : error === probe.token ? ['error', 'token']
    : error === probe.matrixToken ? ['error', 'matrix-token']
    : ['error', error?.name, error?.message];
  return JSON.stringify({ outcome, trace: probe.trace, factoryWasAvailable: probe.factoryWasAvailable });
}
