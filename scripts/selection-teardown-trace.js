// Shared original/Rust public Terminal trace. No private selection service access.
return (async () => {
  const parent = document.createElement('div');
  parent.style.cssText = 'width:800px;height:160px';
  document.body.appendChild(parent);
  const original = document.removeEventListener;
  const ownRemoval = Object.getOwnPropertyDescriptor(document, 'removeEventListener');
  const marker = phase === 'mousemove' ? { selectionTeardown: action } : Symbol('selectionTeardown');
  const snapshot = () => ({ text: terminal.getSelection(), position: terminal.getSelectionPosition() ?? null });
  const events = [];
  let subscription, armed = false, intercepted = false, during = null;
  try {
    terminal.open(parent);
    await new Promise(resolve => terminal.write('abcdefghijklmnopqrst', resolve));
    await new Promise((resolve, reject) => {
      let listener;
      const timer = setTimeout(() => { listener.dispose(); reject(new Error('Selection render deadline')); }, 10000);
      listener = terminal.onRender(() => { clearTimeout(timer); listener.dispose(); resolve(); });
      terminal.refresh(0, 3);
    });
    const screen = terminal.element.querySelector('.xterm-screen');
    const rect = screen.getBoundingClientRect();
    const cell = terminal._core._renderService.dimensions.css.cell;
    if (!(cell.width > 0 && cell.height > 0)) throw new Error('Invalid selection cell dimensions');
    const mouse = (kind, column) => screen.dispatchEvent(new MouseEvent(kind, {
      clientX: rect.left + (column + 0.1) * cell.width,
      clientY: rect.top + 0.5 * cell.height, detail: 1, bubbles: true, cancelable: true
    }));
    mouse('mousedown', 1);
    mouse('mousemove', 4);
    const before = snapshot();
    subscription = terminal.onSelectionChange(() => events.push(snapshot()));
    document.removeEventListener = function(type, ...args) {
      if (armed && type === phase) {
        armed = false;
        intercepted = true;
        during = snapshot();
        if (action === 'throw') throw marker;
        const operation = action.replace('-throw', '');
        if (operation === 'nested') terminal.select(7, 0, 2);
        if (operation === 'all') terminal.selectAll();
        if (operation === 'lines') terminal.selectLines(0, 1);
        if (operation === 'clear') terminal.clearSelection();
        if (action.endsWith('-throw')) throw marker;
      }
      return Reflect.apply(original, this, [type, ...args]);
    };
    armed = true;
    let error = 'none';
    try { terminal.select(2, 0, 3); }
    catch (caught) { error = caught === marker ? 'marker' : `${caught?.name}:${caught?.message}`; }
    const after = snapshot();
    if (!intercepted) throw new Error(`Missing ${phase} drag listener removal`);
    return { action, phase, before, during, after, error, events };
  } finally {
    armed = false;
    if (ownRemoval) Object.defineProperty(document, 'removeEventListener', ownRemoval);
    else delete document.removeEventListener;
    subscription?.dispose();
    try { terminal.clearSelection(); } finally { terminal.dispose(); parent.remove(); }
  }
})();
