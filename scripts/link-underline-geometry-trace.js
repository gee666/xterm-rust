// Shared original/Rust program using public providers, scrolling and live DOM.
return (async () => {
  const modes = ['plain', 'range scroll', 'start.x scroll', 'start.y scroll', 'end.x scroll',
    'end.y scroll', 'coerce start.y', 'coerce end.y', 'range resize', 'end.x resize', 'throw range', 'throw end.y'];
  const trace = [], errors = [];
  const parent = document.createElement('div');
  document.body.append(parent);
  const assert = (value, message) => { if (!value) throw new Error(message); };
  const capture = event => { errors.push(String(event.error)); event.preventDefault(); };
  window.addEventListener('error', capture);
  let pending;
  const token = terminal.registerLinkProvider({ provideLinks(y, callback) { pending = { y, callback }; } });
  try {
    terminal.open(parent);
    await new Promise(resolve => terminal.write(Array.from({ length: 10 }, (_, i) => `row${i}abcdefghij`).join('\r\n'), resolve));
    const rendered = () => new Promise((resolve, reject) => {
      const timer = setTimeout(() => { subscription.dispose(); reject(new Error('render deadline')); }, 10000);
      const subscription = terminal.onRender(() => { clearTimeout(timer); subscription.dispose(); resolve(); });
      terminal.refresh(0, terminal.rows - 1);
    });
    const screen = terminal.element.querySelector('.xterm-screen');
    const mouse = (kind, row) => {
      const rect = screen.getBoundingClientRect(), cell = terminal.dimensions.css.cell;
      screen.dispatchEvent(new MouseEvent(kind, { bubbles: true,
        clientX: rect.left + 2.5 * cell.width, clientY: rect.top + (row - 0.5) * cell.height }));
    };
    const scenarios = modes.flatMap(mode => [true, false].map(show => ({ mode, show })));
    for (const [index, { mode, show }] of scenarios.entries()) {
      mouse('mouseleave', 1);
      terminal.resize(20, 4);
      terminal.scrollToTop();
      await rendered();
      // Alternating rows avoids source's cached request for the previous line.
      const row = index % 2 + 1;
      pending = undefined;
      mouse('mousemove', row);
      assert(pending?.y === row, `${mode}: fresh provider request`);
      const marker = { mode }, reads = [];
      let armed = false, changed = false, caught, hovered = 0;
      const read = (key, value) => {
        if (!armed) return value;
        reads.push(key);
        if (mode === `throw ${key}`) throw marker;
        const mutate = () => {
          if (changed) return;
          changed = true;
          if (mode.endsWith('resize')) terminal.resize(40, 4);
          else terminal.scrollToLine(1);
        };
        if (mode === `${key} scroll` || mode === `${key} resize`) mutate();
        if (mode === `coerce ${key}`) return { valueOf() { reads.push(`coerce ${key}`); mutate(); return value; } };
        return value;
      };
      const range = { start: { get x() { return read('start.x', 2); }, get y() { return read('start.y', row); } },
        end: { get x() { return read('end.x', 8); }, get y() { return read('end.y', row + 1); } } };
      const raw = { text: mode, get range() { read('range', 0); return range; },
        decorations: { underline: !show, pointerCursor: false }, activate() {},
        hover(event, text) { assert(this === raw && text === mode && event.type === 'mousemove', `${mode}: hover identity`); hovered++; } };
      pending.callback([raw]);
      assert(hovered === 1, `${mode}: one hover before arming`);
      armed = true;
      try { raw.decorations.underline = show; } catch (error) { caught = error; }
      armed = false;
      assert(caught === (mode.startsWith('throw') ? marker : undefined), `${mode}: exact thrown identity`);
      const underlines = Array.from(screen.querySelectorAll('.xterm-rows > div'), row =>
        Array.from(row.querySelectorAll('span')).filter(span => span.style.textDecoration === 'underline').map(span => span.textContent).join(''));
      if (show && !mode.endsWith('resize') && !mode.startsWith('throw')) {
        const first = mode === 'range scroll' ? row - 2 : row - 1;
        const expected = underlines.map((_, y) => y >= Math.max(first, 0) && y <= first + 1);
        assert(underlines.every((text, y) => !!text === expected[y]), `${mode}: underline rows ${JSON.stringify(underlines)}`);
      }
      trace.push({ mode, show, reads, underlines, columns: terminal.cols,
        viewport: terminal.buffer.active.viewportY, caughtMarker: caught === marker });
    }
    assert(errors.length === 0, `Unexpected DOM errors: ${errors}`);
    return trace;
  } finally {
    token.dispose(); terminal.dispose(); parent.remove(); window.removeEventListener('error', capture);
  }
})();
