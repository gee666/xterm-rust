// Identical public Terminal trace for source and Rust. Intercept only host teardown.
return (async () => {
  const parent = document.createElement('div');
  parent.style.cssText = 'width:800px;height:160px';
  document.body.appendChild(parent);
  const original = document.removeEventListener;
  const own = Object.getOwnPropertyDescriptor(document, 'removeEventListener');
  const marker = phase === 'mousemove' ? { liveGeometry: action } : Symbol('liveGeometry');
  const snapshot = () => ({ cols: terminal.cols, rows: terminal.rows,
    position: terminal.getSelectionPosition() ?? null, text: terminal.getSelection() });
  const events = [];
  let subscription, armed = false, intercepted = false;
  try {
    terminal.open(parent);
    await new Promise(resolve => terminal.write('abcdefghijklmnopqrstuvwxyz0123456789', resolve));
    await new Promise((resolve, reject) => {
      let listener;
      const deadline = setTimeout(() => { listener.dispose(); reject(new Error('Selection render deadline')); }, 10000);
      listener = terminal.onRender(() => { clearTimeout(deadline); listener.dispose(); resolve(); });
      terminal.refresh(0, 3);
    });
    const screen = terminal.element.querySelector('.xterm-screen');
    const rect = screen.getBoundingClientRect();
    const cell = terminal._core._renderService.dimensions.css.cell;
    if (!(cell.width > 0 && cell.height > 0)) throw new Error('Invalid cell dimensions');
    for (const [kind, column] of [['mousedown', 1], ['mousemove', 4]]) {
      screen.dispatchEvent(new MouseEvent(kind, { clientX: rect.left + (column + 0.1) * cell.width,
        clientY: rect.top + 0.5 * cell.height, detail: 1, bubbles: true, cancelable: true }));
    }
    subscription = terminal.onSelectionChange(() => events.push(snapshot()));
    document.removeEventListener = function(type, ...args) {
      if (armed && type === phase) {
        armed = false;
        intercepted = true;
        terminal.resize(action.startsWith('shrink') ? 10 : 32, 4);
        if (action.endsWith('throw')) throw marker;
      }
      return Reflect.apply(original, this, [type, ...args]);
    };
    armed = true;
    let error = 'none';
    try { terminal.select(8, 0, 15); }
    catch (caught) { error = caught === marker ? 'marker' : `${caught?.name}:${caught?.message}`; }
    if (!intercepted) throw new Error(`Missing ${phase} removal`);
    const after = snapshot();
    const count = events.length;
    terminal.select(8, 0, 15);
    const repeated = snapshot();
    return { action, phase, error, after, repeated, initialEvents: count, events };
  } finally {
    armed = false;
    if (own) Object.defineProperty(document, 'removeEventListener', own);
    else delete document.removeEventListener;
    subscription?.dispose();
    try { terminal.clearSelection(); } finally { terminal.dispose(); parent.remove(); }
  }
})();
