// Shared original/Rust opened-Terminal program. Each case gets a fresh request.
return (async () => {
  const results = [];
  const parent = document.createElement('div');
  document.body.append(parent);
  const errors = [];
  const capture = event => { errors.push(event.error); event.preventDefault(); };
  window.addEventListener('error', capture);
  const assert = (value, message) => { if (!value) throw new Error(message); };
  try {
    terminal.open(parent);
    await new Promise(resolve => terminal.write('abcdefghijklmnopqrst\r\nsecond line', resolve));
    await new Promise((resolve, reject) => {
      const timer = setTimeout(() => { token.dispose(); reject(new Error('render deadline')); }, 10000);
      const token = terminal.onRender(() => { clearTimeout(timer); token.dispose(); resolve(); });
      terminal.refresh(0, terminal.rows - 1);
    });
    const screen = terminal.element.querySelector('.xterm-screen');
    const cell = terminal.dimensions.css.cell;
    let row = 1;
    const mouse = (kind, x) => {
      const rect = screen.getBoundingClientRect();
      screen.dispatchEvent(new MouseEvent(kind, { bubbles: true,
        clientX: rect.left + (x - 0.5) * cell.width, clientY: rect.top + (row - 0.5) * cell.height }));
    };
    const marker = { marker: 'hover failure' };
    for (const mode of ['nested undefined', 'nested empty', 'nested link', 'three providers',
      'throw before commit', 'nested throw caught']) {
      mouse('mouseleave', 1);
      // A different row invalidates the source's provider cache for each case.
      row = 1 + results.length % 2;
      const callbacks = [];
      const calls = [];
      const count = mode === 'three providers' ? 3 : 2;
      const tokens = Array.from({ length: count }, (_, i) => terminal.registerLinkProvider({
        provideLinks(y, callback) { calls.push(`provide${i}:${y}`); callbacks[i] = callback; }
      }));
      let hovers = 0;
      let caught;
      const high = { text: 'high', range: { start: { x: 2, y: row }, end: { x: 12, y: row } },
        decorations: { underline: false, pointerCursor: true }, activate() { calls.push('activate'); },
        hover() {
          calls.push(`hover${++hovers}`);
          if (mode === 'throw before commit' && hovers === 1) throw marker;
          if (mode === 'nested throw caught' && hovers === 2) throw marker;
          if (hovers !== 1) return;
          if (mode === 'three providers') {
            callbacks[1](undefined);
            callbacks[2](undefined);
          } else if (mode !== 'throw before commit') {
            try {
              callbacks[1](mode === 'nested empty' ? [] : mode === 'nested link' ? [{
                text: 'low', range: high.range, activate() {}, hover() { calls.push('low hover'); }
              }] : undefined);
            } catch (error) { caught = error; calls.push('nested caught'); }
          }
        }
      };
      // Alternate cells so the source's same-cell optimization cannot skip a case.
      mouse('mousemove', 3 + results.length);
      assert(callbacks.length === count, `${mode}: providers not requested`);
      // The built-in OSC provider replies with a truthy empty array. Complete
      // lower providers first so the source's all-replies fallback can select high.
      for (let i = 1; i < count; i++) callbacks[i](undefined);
      assert(hovers === 0, `${mode}: unexpected hover before high reply`);
      try { callbacks[0]([high]); } catch (error) { caught = error; calls.push('outer caught'); }
      if (mode === 'throw before commit') callbacks[1](undefined);
      const beforeRepeat = hovers;
      callbacks[count - 1](undefined);
      assert(hovers === beforeRepeat, `${mode}: successful outer return did not commit ${JSON.stringify({ beforeRepeat, hovers, calls })}`);
      mouse('mousedown', 4); mouse('mouseup', 4);
      const record = { mode, calls, hovers, caughtMarker: caught === marker,
        pointer: screen.classList.contains('xterm-cursor-pointer'), errors: errors.length,
        accessorsInstalled: typeof Object.getOwnPropertyDescriptor(high.decorations, 'underline')?.get === 'function' };
      assert(hovers === 2, `${mode}: expected 2 hovers, got ${hovers}`);
      assert(!errors.length, `${mode}: unexpected DOM error`);
      assert(record.caughtMarker === mode.includes('throw'), `${mode}: thrown identity`);
      assert(calls.at(-1) === 'activate', `${mode}: live current link not activated`);
      results.push(record);
      mouse('mouseleave', 1);
      tokens.forEach(token => token.dispose());
    }
    return results;
  } finally {
    window.removeEventListener('error', capture);
    terminal.dispose();
    parent.remove();
  }
})();
