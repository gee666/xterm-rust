// Copyright (c) 2016 The xterm.js authors. All rights reserved.
// Licensed under MIT. Only the oracle executes the unchanged source method.
export const boundsCases = [
  ['finite', 6.5, 4.5], ['zero', 0, 0], ['negative', -4, -3],
  ['NaN-width', NaN, 4.5], ['NaN-height', 6.5, NaN],
  ['NaN-both', NaN, NaN], ['infinite', Infinity, Infinity],
];

export function geometryModel(area, overlay, body) {
  return {
    _textarea: area, _compositionView: overlay, _isComposing: true,
    _bufferService: { buffer: { isCursorInViewport: true, x: 80, y: 2 }, cols: 80 },
    _optionsService: { rawOptions: { fontFamily: 'monospace', fontSize: 15 } },
    _renderService: { dimensions: { css: { cell: { width: 9, height: 18 } } } },
    updateCompositionElements: new Function('dontRecurse', body),
  };
}

const turn = () => new Promise(resolve => setTimeout(resolve, 0));
function snapshot(area) {
  return ['width', 'height', 'lineHeight'].map(key => area.style[key]);
}
async function session(area, overlay, invoke, width, height) {
  const original = Object.getOwnPropertyDescriptor(overlay, 'getBoundingClientRect');
  Object.defineProperty(overlay, 'getBoundingClientRect', {
    configurable: true, value: () => ({ width, height }),
  });
  area.style.width = '7px';
  area.style.height = '11px';
  area.style.lineHeight = '13px';
  try {
    invoke();
    const trace = [snapshot(area)];
    await turn();
    trace.push(snapshot(area));
    return trace;
  } finally {
    if (original) Object.defineProperty(overlay, 'getBoundingClientRect', original);
    else delete overlay.getBoundingClientRect;
  }
}

export async function compareCompositionBounds(area, overlay, body, name) {
  const [, width, height] = boundsCases.find(item => item[0] === name);
  const referenceArea = document.createElement('textarea');
  const referenceOverlay = document.createElement('div');
  const model = geometryModel(referenceArea, referenceOverlay, body);
  const expected = await session(referenceArea, referenceOverlay,
    () => model.updateCompositionElements(), width, height);
  area.dispatchEvent(new CompositionEvent('compositionstart'));
  await turn(); // Let the initial cursor-sync geometry finish before seeding CSS.
  const actual = await session(area, overlay,
    () => area.dispatchEvent(new CompositionEvent('compositionupdate', { data: 'x' })), width, height);
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error(name + ': source=' + JSON.stringify(expected) + ' Rust=' + JSON.stringify(actual));
  }
  console.log('IME bounds ' + JSON.stringify({ name, expected, actual }));
}
