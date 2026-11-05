// Shared original/Rust opened-Terminal program. No private controller replacement.
return (async () => {
  const modes = ['unchanged', 'first decoration read', 'last decoration read', 'underline range read', 'throw getter', 'throw hover', 'dispose in getter'];
  const trace = [], errors = [];
  const parent = document.createElement('div');
  document.body.append(parent);
  const capture = event => { errors.push(String(event.error)); event.preventDefault(); };
  window.addEventListener('error', capture);
  const assert = (value, message) => { if (!value) throw new Error(message); };
  let pending;
  const token = terminal.registerLinkProvider({ provideLinks(y, callback) { pending = { y, callback }; } });
  let disposed = false;
  try {
    terminal.open(parent);
    await new Promise(resolve => terminal.write('first row\r\nsecond row', resolve));
    await new Promise((resolve, reject) => {
      const timer = setTimeout(() => { subscription.dispose(); reject(new Error('render deadline')); }, 10000);
      const subscription = terminal.onRender(() => { clearTimeout(timer); subscription.dispose(); resolve(); });
      terminal.refresh(0, terminal.rows - 1);
    });
    const screen = terminal.element.querySelector('.xterm-screen');
    const widget = document.createElement('span');
    widget.className = 'xterm-hover';
    screen.append(widget);
    const cell = terminal.dimensions.css.cell;
    const mouse = (target, x, y, marker) => {
      const rect = screen.getBoundingClientRect();
      const event = new MouseEvent('mousemove', { bubbles: true, clientX: rect.left + (x - 0.5) * cell.width,
        clientY: rect.top + (y - 0.5) * cell.height });
      Object.defineProperty(event, 'marker', { value: marker });
      target.dispatchEvent(event);
      return event;
    };
    for (const [index, mode] of modes.entries()) {
      screen.dispatchEvent(new MouseEvent('mouseleave'));
      const y = index % 2 + 1;
      pending = undefined;
      const outer = mouse(screen, 4, y, 'outer');
      assert(pending?.y === y, `${mode}: expected fresh provider request`);
      const resolveReply = pending.callback;
      const marker = { mode };
      let nested, reads = 0, hoverEvent, calls = 0, caught;
      const raw = { text: mode, range: { start: { x: 2, y }, end: { x: 8, y } },
        activate() {}, hover(event, text) {
          calls++;
          hoverEvent = event;
          assert(this === raw && text === mode, `${mode}: hover receiver/text`);
          if (mode === 'throw hover') throw marker;
        } };
      if (mode === 'underline range read') {
        const range = raw.range;
        Object.defineProperty(raw, 'range', { get() {
          if (reads >= 4 && !nested) nested = mouse(widget, 5, y, 'nested');
          return range;
        } });
      }
      Object.defineProperty(raw, 'decorations', { configurable: true, get() {
        reads++;
        if (mode === 'first decoration read' && reads === 1 || mode === 'last decoration read' && reads === 4 || mode === 'throw hover' && reads === 4) {
          nested = mouse(widget, 5, y, 'nested');
        }
        if (mode === 'throw getter') throw marker;
        if (mode === 'dispose in getter' && reads === 4) { terminal.dispose(); disposed = true; }
        return { underline: mode === 'underline range read', pointerCursor: false };
      }, set(value) { Object.defineProperty(raw, 'decorations', { value, writable: true, configurable: true }); } });
      try { resolveReply([raw]); } catch (error) { caught = error; }
      const expectedEvent = mode === 'dispose in getter' || mode === 'throw getter' ? undefined : mode === 'underline range read' ? outer : nested ?? outer;
      assert(hoverEvent === expectedEvent, `${mode}: hover must receive ${expectedEvent?.marker ?? 'undefined'}, got ${hoverEvent?.marker ?? 'undefined'}`);
      assert(calls === (mode === 'throw getter' ? 0 : 1), `${mode}: hover count ${calls}`);
      assert(caught === (mode.startsWith('throw') ? marker : undefined), `${mode}: exact thrown identity`);
      trace.push({ mode, reads, calls, event: hoverEvent?.marker ?? 'undefined', exactEvent: hoverEvent === expectedEvent,
        caughtMarker: caught === marker, descriptorInstalled: typeof Object.getOwnPropertyDescriptor(Object.getOwnPropertyDescriptor(raw, 'decorations').value ?? {}, 'underline')?.get === 'function' });
    }
    assert(errors.length === 0, `Unexpected DOM errors: ${errors}`);
    return trace;
  } finally {
    token.dispose();
    if (!disposed) terminal.dispose();
    parent.remove();
    window.removeEventListener('error', capture);
  }
})();
