// Copyright (c) 2016 The xterm.js authors. All rights reserved.
// Licensed under MIT. Rust owns IME behavior; only the oracle executes JS bodies.
const turn = () => new Promise(resolve => setTimeout(resolve, 0));

async function session(area, overlay, start, end, snapshot, behavior) {
  const trace = [];
  const sentinel = { compositionFailure: true };
  const classList = overlay.classList;
  const original = classList.remove;
  let nested = false;
  let captured;
  const onError = event => {
    captured = event.error;
    event.preventDefault();
  };
  window.addEventListener('error', onError);
  try {
    area.value = 'old';
    area.setSelectionRange(0, 0);
    start();
    trace.push(['started', snapshot()]);
    Object.defineProperty(classList, 'remove', {
      configurable: true,
      value(...args) {
        trace.push(['remove', snapshot()]);
        if (behavior === 'throw') throw sentinel;
        if (!nested && behavior !== 'plain') {
          nested = true;
          area.value = 'aXtail';
          area.setSelectionRange(1, 2);
          start();
          trace.push(['nested-start', snapshot()]);
          if (behavior === 'nested-end') end();
        }
        return original.apply(this, args);
      },
    });
    try { end(); } catch (error) { captured = error; }
    trace.push(['ended', snapshot(), captured === sentinel]);
    area.value = behavior === 'plain' ? 'new' : 'aBtail';
    await turn();
    trace.push(['deferred', snapshot()]);
    return trace;
  } finally {
    delete classList.remove;
    window.removeEventListener('error', onError);
  }
}

export async function compareCompositionEnd(area, overlay, snapshot, bodies, behavior) {
  const referenceArea = document.createElement('textarea');
  const referenceOverlay = document.createElement('div');
  const data = [];
  const model = {
    _textarea: referenceArea,
    _compositionView: referenceOverlay,
    _compositionPosition: { start: 0, end: 0 },
    _compositionSuffix: '',
    _dataAlreadySent: '',
    _isComposing: false,
    _isSendingComposition: false,
    _coreService: { triggerDataEvent(value, user) { data.push([value, user]); } },
    compositionstart: new Function(bodies[0]),
    compositionend: new Function(bodies[1]),
    _finalizeComposition: new Function('waitForPropagation', bodies[2]),
  };
  const expected = await session(referenceArea, referenceOverlay,
    () => model.compositionstart(), () => model.compositionend(),
    () => [model._isComposing, data.map(item => [...item]), referenceOverlay.classList.contains('active')], behavior);
  const dispatch = name => area.dispatchEvent(new CompositionEvent(name));
  const actual = await session(area, overlay,
    () => dispatch('compositionstart'), () => dispatch('compositionend'), snapshot, behavior);
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error(`${behavior}: source=${JSON.stringify(expected)} Rust=${JSON.stringify(actual)}`);
  }
  return { behavior, expected, actual };
}
