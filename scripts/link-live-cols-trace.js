// Identical program for the original and Rust exported opened Terminal.
return (async () => {
  const modes = ['unchanged', 'start y', 'end y', 'start x', 'end x',
    'start coercion', 'end coercion', 'reset end', 'throw start', 'throw end'];
  const trace = [], errors = [];
  const parent = document.createElement('div');
  document.body.append(parent);
  const capture = event => { errors.push(String(event.error)); event.preventDefault(); };
  window.addEventListener('error', capture);
  const assert = (value, message) => { if (!value) throw new Error(message); };
  let pending;
  const token = terminal.registerLinkProvider({ provideLinks(y, callback) { pending = { y, callback }; } });
  try {
    terminal.open(parent);
    await new Promise(resolve => terminal.write('first row\r\nsecond row', resolve));
    await new Promise((resolve, reject) => {
      const timer = setTimeout(() => { subscription.dispose(); reject(new Error('render deadline')); }, 10000);
      const subscription = terminal.onRender(() => { clearTimeout(timer); subscription.dispose(); resolve(); });
      terminal.refresh(0, terminal.rows - 1);
    });
    const screen = terminal.element.querySelector('.xterm-screen');
    let row = 1;
    const mouse = (kind, x) => {
      const rect = screen.getBoundingClientRect(), cell = terminal.dimensions.css.cell;
      const event = new MouseEvent(kind, { bubbles: true, clientX: rect.left + (x - 0.5) * cell.width,
        clientY: rect.top + (row - 0.5) * cell.height });
      screen.dispatchEvent(event);
      return event;
    };
    for (const [index, mode] of modes.entries()) {
      mouse('mouseleave', 1);
      terminal.resize(20, 4);
      row = index % 2 + 1;
      pending = undefined;
      const event = mouse('mousemove', 2 + index % 2);
      assert(pending?.y === row, `${mode}: fresh provider request`);
      const resolveReply = pending.callback;
      const marker = { mode }, reads = [], hover = [];
      let armed = true, caught;
      const read = (end, key, value) => {
        if (!armed) return value;
        reads.push(`${end}.${key}`);
        if (mode === `throw ${end}` && key === 'y') throw marker;
        // Reset each probe so fallback and selection observe the same case.
        if (end === 'start' && key === 'y') terminal.resize(20, 4);
        if (mode === `${end} ${key}`) terminal.resize(40, 4);
        if (mode === 'reset end' && key === 'y') terminal.resize(end === 'start' ? 40 : 20, 4);
        if (key === 'y' && mode === `${end} coercion`) return { valueOf() {
          reads.push(`${end}.valueOf`); terminal.resize(40, 4); return value;
        } };
        return value;
      };
      const point = (end, x) => ({ get x() { return read(end, 'x', x); }, get y() { return read(end, 'y', row); } });
      const raw = { text: mode, range: { start: point('start', 1), end: point('end', 20) },
        decorations: { underline: false, pointerCursor: false }, activate() {},
        hover(actual, text) {
          assert(this === raw && actual === event && text === mode, `${mode}: hover identity/receiver/text`);
          hover.push(text);
          armed = false;
        } };
      try { resolveReply([raw]); } catch (error) { caught = error; }
      const expected = !['end x', 'end coercion', 'reset end', 'throw start', 'throw end'].includes(mode);
      assert(hover.length === Number(expected), `${mode}: expected ${Number(expected)} hover, got ${hover.length}`);
      assert(caught === (mode.startsWith('throw') ? marker : undefined), `${mode}: exact thrown identity`);
      trace.push({ mode, hover, reads, columns: terminal.cols, caughtMarker: caught === marker });
      armed = false;
    }
    assert(errors.length === 0, `Unexpected DOM errors: ${errors}`);
    return trace;
  } finally {
    token.dispose();
    terminal.dispose();
    parent.remove();
    window.removeEventListener('error', capture);
  }
})();
