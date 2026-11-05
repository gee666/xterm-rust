// Shared original/exported Rust Terminal trace. No helper linkifier.
return (async () => {
  const trace = [], calls = [], errors = [];
  const marker = { marker: 'range getter' };
  const parent = document.createElement('div');
  document.body.append(parent);
  const capture = event => { errors.push(event.error); event.preventDefault(); };
  window.addEventListener('error', capture);
  let range, firstRange, activeLink, screen, probe, probeParent;
  let activeTerminal = terminal;
  const originalOpen = window.open, originalWarn = console.warn;
  let first = true;
  const options = {
    hover(event, text, value) {
      range = value;
      firstRange ??= value;
      calls.push(['hover', this === options, text, value === firstRange]);
      if (first) { value.start.x = 8; value.end.x = 10; first = false; }
    },
    leave(event, text) { calls.push(['leave', this === activeLink, text]); }
  };
  const addon = createAddon(function(event, text) {
    activeLink = this;
    calls.push(['activate', this.range === range, text]);
  }, options);
  const render = () => new Promise((resolve, reject) => {
    const timer = setTimeout(() => { subscription.dispose(); reject(new Error('render deadline')); }, 10000);
    const subscription = activeTerminal.onRender(() => { clearTimeout(timer); subscription.dispose(); resolve(); });
    activeTerminal.refresh(0, activeTerminal.rows - 1);
  });
  const mouse = (type, x, y = 1, detail = 1) => {
    const rect = screen.getBoundingClientRect();
    const cell = activeTerminal.dimensions.css.cell;
    screen.dispatchEvent(new MouseEvent(type, { bubbles: true, detail,
      clientX: rect.left + (x - 0.5) * cell.width, clientY: rect.top + (y - 0.5) * cell.height }));
  };
  const record = name => {
    trace.push({ name, pointer: screen.classList.contains('xterm-cursor-pointer'), calls: calls.splice(0),
      errors: errors.splice(0).map(error => error === marker ? 'marker' : error.name) });
  };
  try {
    addon.activate(terminal);
    terminal.open(parent);
    terminal.focus();
    await new Promise(resolve => terminal.write('http://x.cohijklmno', resolve));
    await render();
    screen = terminal.element.querySelector('.xterm-screen');
    mouse('mousemove', 3);
    record('hover mutates callback range');
    mouse('mousemove', 3);
    record('same cell ignores mutation');
    mouse('mousemove', 4);
    record('next cell leaves mutated range');
    mouse('mousemove', 9);
    record('cached mutated range reenters');
    mouse('mousedown', 9);
    range.start.x = 12; range.end.x = 14;
    mouse('mouseup', 13);
    record('mousedown equality reads live range and handler receiver');
    mouse('mousedown', 13, 1, 2);
    trace.push({ name: 'double click selects mutated range', selection: terminal.getSelection() });
    mouse('mouseup', 13);
    record('double click mouseup');
    terminal.clearSelection();
    range.start = { x: 16, y: 1 }; range.end = { x: 18, y: 1 };
    mouse('mousemove', 17);
    record('replacement endpoints stay live');
    const reads = [];
    for (const [end, values] of [['start', { x: 16, y: 1 }], ['end', { x: 18, y: 1 }]]) {
      range[end] = new Proxy(values, { get(target, key) { reads.push(`${end}.${String(key)}`); return target[key]; } });
    }
    mouse('mousemove', 17);
    trace.push({ name: 'same cell avoids proxy getters', reads: reads.splice(0) });
    mouse('mousemove', 16);
    trace.push({ name: 'hit test reads proxy coordinates', reads: reads.splice(0) });
    const start = range.start;
    range.start = new Proxy({}, { get(target, key) { if (key === 'y') throw marker; return start[key]; } });
    mouse('mousemove', 18);
    record('range getter preserves thrown identity');
    range.start = { x: 16, y: 1 }; range.end = { x: 18, y: 1 };
    options.leave = () => calls.push(['new leave']);
    mouse('mousemove', 15);
    record('cached leave remains original option value');
    const hoverReads = [];
    Object.defineProperty(options, 'hover', { configurable: true, get() {
      hoverReads.push('hover');
      return function(event, text, value) { calls.push(['new hover', this === options, value === firstRange]); };
    } });
    mouse('mousemove', 17);
    record('replaced hover remains live with options receiver');
    trace.push({ name: 'hover getter read twice', reads: hoverReads.splice(0) });
    // Activation exposes the real source link as its receiver. Replacing the
    // complete range must affect both its hover argument and Rust decisions.
    activeLink.range = { start: { x: 5, y: 1 }, end: { x: 7, y: 1 } };
    mouse('mousemove', 6);
    record('replacement link range feeds hover');
    mouse('mousedown', 6, 1, 2);
    trace.push({ name: 'selection reads replacement link range', selection: terminal.getSelection() });
    mouse('mouseup', 6);
    record('replacement selection mouseup');
    terminal.clearSelection();
    mouse('mouseleave', 6);
    addon.dispose();
    terminal.refresh(0, terminal.rows - 1);
    await render();
    mouse('mousemove', 5);
    record('disposed addon no longer provides links');
    for (const [name, handler, mode] of [
      ['default successful navigation', undefined, 'open'],
      ['default blocked window', undefined, 'blocked'],
      ['default tolerates opener exception', undefined, 'opener'],
      ['default location exception identity', undefined, 'location'],
      ['null activation', null], ['false activation', false], ['zero activation', 0],
      ['empty string activation', ''], ['object activation', {}],
      ['handler exception identity', () => { throw marker; }]
    ]) {
      let opened;
      window.open = () => {
        calls.push(['window.open']);
        if (mode === 'blocked') return null;
        opened = { opener: 'initial', location: {} };
        if (mode === 'opener') Object.defineProperty(opened, 'opener', { get() { return 'initial'; }, set() { throw marker; } });
        if (mode === 'location') Object.defineProperty(opened, 'location', { get() { throw marker; } });
        return opened;
      };
      console.warn = message => calls.push(['warning', message]);
      activeTerminal = createTerminal();
      probeParent = document.createElement('div');
      document.body.append(probeParent);
      probe = createAddon(handler, {});
      probe.activate(activeTerminal);
      activeTerminal.open(probeParent);
      activeTerminal.focus();
      await new Promise(resolve => activeTerminal.write('http://x.co', resolve));
      await render();
      screen = activeTerminal.element.querySelector('.xterm-screen');
      mouse('mousemove', 6);
      mouse('mousedown', 6);
      mouse('mouseup', 6);
      if (mode && !calls.some(call => call[0] === 'window.open')) throw new Error(`default activation not reached: ${name}`);
      if (!mode && errors.length !== 1) throw new Error(`raw activation not reached: ${name}`);
      if (opened && mode !== 'location') calls.push(['navigation', opened.opener, opened.location.href]);
      record(name);
      mouse('mouseleave', 6);
      probe.dispose();
      probe = undefined;
      activeTerminal.dispose();
      activeTerminal = terminal;
      probeParent.remove();
      probeParent = undefined;
    }
    return trace;
  } finally {
    probe?.dispose();
    if (activeTerminal !== terminal) activeTerminal.dispose();
    probeParent?.remove();
    window.open = originalOpen;
    console.warn = originalWarn;
    addon.dispose();
    terminal.dispose();
    parent.remove();
    window.removeEventListener('error', capture);
  }
})();
