// Shared program for the original and Rust exported opened Terminal.
return (async () => {
  const modes = ['plain', 'second start outside', 'second end outside', 'throw second start',
    'throw second end', 'first start outside', 'first end outside', 'zero start'];
  const trace = [], errors = [], parent = document.createElement('div');
  document.body.append(parent);
  const assert = (value, message) => { if (!value) throw new Error(message); };
  const capture = event => { errors.push(event.error); event.preventDefault(); };
  window.addEventListener('error', capture);
  let pending;
  const token = terminal.registerLinkProvider({ provideLinks(y, callback) { pending = { y, callback }; } });
  const render = (start, end, marker) => new Promise((resolve, reject) => {
    const finish = event => {
      clearTimeout(timer); subscription.dispose(); window.removeEventListener('error', failed); resolve(event);
    };
    const failed = event => { if (marker && event.error === marker) finish({ start, end }); };
    const timer = setTimeout(() => {
      subscription.dispose(); window.removeEventListener('error', failed); reject(new Error('render deadline'));
    }, 10000);
    const subscription = terminal.onRender(event => { if (!marker) finish(event); });
    window.addEventListener('error', failed);
    terminal.refresh(start, end);
  });
  try {
    terminal.open(parent);
    await new Promise(resolve => terminal.write('row one\r\nrow two\r\nrow three', resolve));
    await render(0, terminal.rows - 1);
    const screen = terminal.element.querySelector('.xterm-screen');
    let row = 2;
    const mouse = (kind, x) => {
      const rect = screen.getBoundingClientRect(), cell = terminal.dimensions.css.cell;
      screen.dispatchEvent(new MouseEvent(kind, { bubbles: true,
        clientX: rect.left + (x - 0.5) * cell.width, clientY: rect.top + (row - 0.5) * cell.height }));
    };
    for (const [index, mode] of modes.entries()) {
      mouse('mouseleave', 1);
      await render(0, terminal.rows - 1);
      row = index % 2 + 2;
      pending = undefined;
      mouse('mousemove', 2 + index % 2);
      assert(pending?.y === row, `${mode}: fresh provider row`);
      const reads = [], events = [], marker = { mode };
      let armed = false, startReads = 0, endReads = 0;
      const point = end => ({ x: end ? 10 : 1, get y() {
        if (!armed) return row;
        const ordinal = end ? ++endReads : ++startReads;
        reads.push(`${end ? 'end' : 'start'}:${ordinal}`);
        if (mode === `throw second ${end ? 'end' : 'start'}` && ordinal === 2) throw marker;
        if (mode === `${ordinal === 1 ? 'first' : 'second'} ${end ? 'end' : 'start'} outside`) return end ? row + 1 : row - 1;
        return row;
      } });
      const link = { text: mode, range: { start: point(false), end: point(true) },
        decorations: { underline: false, pointerCursor: false }, activate() {},
        hover() { events.push('hover'); }, leave() { events.push('leave'); } };
      pending.callback([link]);
      assert(events.join() === 'hover', `${mode}: initial hover`);
      events.length = 0;
      pending = undefined;
      armed = true;
      errors.length = 0;
      const rendered = await render(mode === 'zero start' ? 0 : row - 1, row - 1, mode.startsWith('throw') ? marker : undefined);
      armed = false;
      const caughtMarker = errors.includes(marker);
      assert(errors.every(error => error === marker), `${mode}: unexpected DOM error`);
      const expectedReads = mode === 'first start outside' ? ['start:1']
        : mode === 'first end outside' || mode === 'zero start' ? ['start:1', 'end:1']
        : mode === 'second start outside' || mode === 'throw second start' ? ['start:1', 'end:1', 'start:2']
        : ['start:1', 'end:1', 'start:2', 'end:2'];
      assert(JSON.stringify(reads) === JSON.stringify(expectedReads), `${mode}: range recheck reads ${JSON.stringify(reads)}`);
      assert(events.join() === (['plain', 'zero start'].includes(mode) ? 'leave' : ''), `${mode}: leave phase`);
      assert(Boolean(pending) === ['plain', 'zero start', 'second start outside', 'second end outside'].includes(mode), `${mode}: provider refresh phase`);
      assert(caughtMarker === mode.startsWith('throw'), `${mode}: thrown identity`);
      trace.push({ mode, reads: [...reads], events: [...events], requested: Boolean(pending), caughtMarker,
        rendered: { start: rendered.start, end: rendered.end } });
    }
    return trace;
  } finally {
    token.dispose(); terminal.dispose(); parent.remove(); window.removeEventListener('error', capture);
  }
})();
