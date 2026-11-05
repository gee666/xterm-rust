// Copyright (c) 2016 The xterm.js authors. All rights reserved.
// Licensed under MIT. Only the reference side executes original JS method bodies.
const turn = () => new Promise(resolve => setTimeout(resolve, 0));

async function session(area, overlay, invoke, snapshot, behavior) {
  const trace = [];
  const sentinel = { startFailure: true };
  const text = Object.getOwnPropertyDescriptor(Node.prototype, 'textContent');
  let captured;
  let nested = false;
  const onError = event => {
    captured = event.error;
    event.preventDefault();
  };
  window.addEventListener('error', onError);
  try {
    area.value = 'ab';
    invoke('keydown');
    area.value = 'abX';
    await turn();
    trace.push(['already-sent', snapshot()]);
    area.setSelectionRange(0, 0);
    Object.defineProperty(overlay, 'textContent', {
      configurable: true,
      get() { return text.get.call(this); },
      set(value) {
        trace.push(['textContent', value, snapshot()]);
        if (behavior === 'nested-end-throw') invoke('compositionend');
        if (behavior === 'nested-start' && !nested) {
          nested = true;
          invoke('compositionstart');
        }
        if (behavior === 'throw' || behavior === 'nested-end-throw') throw sentinel;
        text.set.call(this, value);
      },
    });
    try { invoke('compositionstart'); } catch (error) { captured = error; }
    trace.push(['started', snapshot(), captured === sentinel]);
    delete overlay.textContent;
    invoke('compositionend');
    area.value = 'YZabX';
    await turn();
    trace.push(['deferred', snapshot()]);
    return trace;
  } finally {
    delete overlay.textContent;
    window.removeEventListener('error', onError);
  }
}

export async function compareCompositionStart(area, overlay, snapshot, bodies, behavior) {
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
    keydown: new Function('ev', bodies[3]),
    _handleAnyTextareaChanges: new Function('C0', 'return function() {' + bodies[4] + '\n}')({ DEL: '\x7f' }),
  };
  const expected = await session(referenceArea, referenceOverlay,
    name => model[name](name === 'keydown' ? { keyCode: 229 } : undefined),
    () => [model._isComposing, data.map(item => [...item]), referenceOverlay.classList.contains('active')], behavior);
  const actual = await session(area, overlay,
    name => area.dispatchEvent(name === 'keydown'
      ? new KeyboardEvent(name, { keyCode: 229 })
      : new CompositionEvent(name)), snapshot, behavior);
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error(behavior + ': source=' + JSON.stringify(expected) + ' Rust=' + JSON.stringify(actual));
  }
  return { behavior, expected, actual };
}
