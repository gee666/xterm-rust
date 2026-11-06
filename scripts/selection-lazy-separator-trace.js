// Shared original/Rust public Terminal input. No private selection state is replaced.
return (async () => {
  const parent = document.createElement('div');
  parent.style.cssText = 'width:800px;height:160px';
  document.body.appendChild(parent);
  const marker = poison === 'primitive-throw' ? Symbol('unused separator') : { unusedSeparator: poison };
  const calls = [], events = [];
  const snapshot = () => ({ text: terminal.getSelection(), position: terminal.getSelectionPosition() ?? null });
  let subscription;
  try {
    terminal.open(parent);
    await new Promise(resolve => terminal.write('hello-rust abcdefghi!', resolve));
    await new Promise((resolve, reject) => {
      let listener;
      const timer = setTimeout(() => { listener?.dispose(); reject(new Error('Selection render deadline')); }, 10000);
      listener = terminal.onRender(() => { clearTimeout(timer); listener.dispose(); resolve(); });
      terminal.refresh(0, 3);
    });
    const screen = terminal.element.querySelector('.xterm-screen');
    const rect = screen.getBoundingClientRect();
    const cell = terminal._core._renderService.dimensions.css.cell;
    if (!(cell.width > 0 && cell.height > 0)) throw new Error('Invalid cell dimensions');
    const mouse = (kind, column, detail = 1, shiftKey = false) => {
      const event = new MouseEvent(kind, { clientX: rect.left + (column + 0.1) * cell.width,
        clientY: rect.top + 0.5 * cell.height, detail, shiftKey, bubbles: true, cancelable: true });
      screen.dispatchEvent(event);
      return event.defaultPrevented;
    };
    if (action === 'up') { mouse('mousedown', 1); mouse('mousemove', 4); }
    if (action === 'shift') terminal.select(1, 0, 3);
    subscription = terminal.onSelectionChange(() => events.push(snapshot()));
    const separator = poison === 'string-throw' ? { toString() { calls.push('toString'); throw marker; } } : {
      [Symbol.toPrimitive](hint) { calls.push(hint); if (poison === 'primitive-throw') throw marker; return '-'; }
    };
    terminal.options.wordSeparator = separator;
    let prevented = null;
    if (action === 'normal') { prevented = mouse('mousedown', 1); mouse('mousemove', 4); }
    if (action === 'shift') prevented = mouse('mousedown', 6, 1, true);
    if (action === 'triple') prevented = mouse('mousedown', 3, 3);
    mouse('mouseup', action === 'shift' ? 6 : 4);
    const after = snapshot();
    // A normal string still drives actual word selection after the lazy paths.
    terminal.options.wordSeparator = '- ';
    mouse('mousedown', 2, 2);
    mouse('mouseup', 2);
    const word = snapshot();
    return { action, poison, calls, prevented, after, word, events };
  } finally {
    subscription?.dispose();
    terminal.options.wordSeparator = '- ';
    try { terminal.clearSelection(); } finally { terminal.dispose(); parent.remove(); }
  }
})();
