// Shared public Terminal observations. No selection service is replaced.
return (async () => {
  const parent = document.createElement('div');
  parent.style.cssText = 'width:800px;height:160px';
  document.body.appendChild(parent);
  const originals = new Map();
  const patch = (object, key, value) => {
    originals.set(key, { object, descriptor: Object.getOwnPropertyDescriptor(object, key) });
    object[key] = value;
  };
  const add = document.addEventListener, remove = document.removeEventListener;
  const set = window.setInterval, clear = window.clearInterval;
  const callbacks = new Map(), timers = new Set();
  const marker = phase === 'mouseup' ? Symbol('selection teardown') : { selectionTeardown: action };
  const trace = [], events = [];
  let armed = false, starting = false, intercepted = false, subscription;
  const snapshot = () => ({ position: terminal.getSelectionPosition() ?? null, text: terminal.getSelection() });
  const observe = (kind, argument) => {
    if (!armed) return;
    trace.push({ kind, argument, ...snapshot() });
    if (kind !== phase || intercepted || action === 'plain') return;
    intercepted = true;
    if (action === 'throw') throw marker;
    if (action === 'reenter') terminal.select(2, 0, 2);
  };
  try {
    terminal.open(parent);
    await new Promise(resolve => terminal.write('abcdefghijklmnopqrstuvwxyz0123456789', resolve));
    await new Promise((resolve, reject) => {
      let listener;
      const deadline = setTimeout(() => { listener.dispose(); reject(new Error('Selection render deadline')); }, 10000);
      listener = terminal.onRender(() => { clearTimeout(deadline); listener.dispose(); resolve(); });
      terminal.refresh(0, 3);
    });
    patch(document, 'addEventListener', function(type, callback, ...args) {
      if (starting && ['mousemove', 'mouseup'].includes(type)) callbacks.set(type, callback);
      return Reflect.apply(add, this, [type, callback, ...args]);
    });
    patch(document, 'removeEventListener', function(type, callback, ...args) {
      if (['mousemove', 'mouseup'].includes(type)) observe(type, callbacks.get(type) === callback);
      return Reflect.apply(remove, this, [type, callback, ...args]);
    });
    patch(window, 'setInterval', function(...args) {
      const id = Reflect.apply(set, this, args);
      if (starting) timers.add(id);
      return id;
    });
    patch(window, 'clearInterval', function(id) {
      observe('clearInterval', id === undefined ? 'undefined' : timers.has(id) ? 'drag' : 'other');
      const result = Reflect.apply(clear, this, [id]);
      timers.delete(id);
      return result;
    });
    const screen = terminal.element.querySelector('.xterm-screen');
    const rect = screen.getBoundingClientRect();
    const cell = terminal._core._renderService.dimensions.css.cell;
    if (!(cell.width > 0 && cell.height > 0)) throw new Error('Invalid cell geometry');
    starting = true;
    for (const [kind, column] of [['mousedown', 1], ['mousemove', 4]]) {
      screen.dispatchEvent(new MouseEvent(kind, { clientX: rect.left + (column + 0.1) * cell.width,
        clientY: rect.top + cell.height / 2, detail: 1, bubbles: true, cancelable: true }));
    }
    starting = false;
    if (timers.size !== 1 || callbacks.size !== 2) throw new Error('Missing real drag setup');
    subscription = terminal.onSelectionChange(() => events.push(snapshot()));
    armed = true;
    let error = 'none';
    try { terminal.select(8, 0, 7); }
    catch (caught) { error = caught === marker ? 'marker' : `${caught?.name}:${caught?.message}`; }
    const first = snapshot(), firstEvents = events.length;
    const firstTrace = trace.splice(0);
    terminal.select(8, 0, 7);
    const retry = snapshot(), retryEvents = events.length;
    const retryTrace = trace.splice(0);
    terminal.select(8, 0, 7);
    const repeated = snapshot();
    return { phase, action, error, intercepted, first, firstEvents, firstTrace,
      retry, retryEvents, retryTrace, repeated, repeatedTrace: trace, events };
  } finally {
    armed = false;
    for (const [key, { object, descriptor }] of originals) {
      // Restoration precedes terminal disposal so cleanup cannot affect the trace.
      if (descriptor) Object.defineProperty(object, key, descriptor);
      else delete object[key];
    }
    for (const id of timers) Reflect.apply(clear, window, [id]);
    subscription?.dispose();
    try { terminal.clearSelection(); } finally { terminal.dispose(); parent.remove(); }
  }
})();
