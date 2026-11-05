// One trace runs against original and exported Rust Terminal. No helper linkifier.
return (async () => {
  const trace = [];
  const calls = [];
  const errors = [];
  const marker = { marker: 'range getter' };
  const parent = document.createElement('div');
  document.body.append(parent);
  const capture = event => { errors.push(event.error); event.preventDefault(); };
  window.addEventListener('error', capture);
  let range;
  let firstRange;
  let first = true;
  const handler = {
    allowNonHttpProtocols: true,
    hover(event, text, value) {
      range = value;
      firstRange ??= value;
      calls.push(['hover', this === handler, text, value === firstRange]);
      if (first) { value.start.x = 8; value.end.x = 10; first = false; }
    },
    leave(event, text, value) { calls.push(['leave', this === handler, text, value === firstRange]); },
    activate(event, text, value) { calls.push(['activate', this === handler, text, value === firstRange]); }
  };
  const render = () => new Promise((resolve, reject) => {
    const timer = setTimeout(() => { subscription.dispose(); reject(new Error('render deadline')); }, 10000);
    const subscription = terminal.onRender(() => { clearTimeout(timer); subscription.dispose(); resolve(); });
    terminal.refresh(0, terminal.rows - 1);
  });
  let screen;
  const mouse = (type, x, y = 1, detail = 1) => {
    const rect = screen.getBoundingClientRect();
    const cell = terminal.dimensions.css.cell;
    screen.dispatchEvent(new MouseEvent(type, { bubbles: true, detail,
      clientX: rect.left + (x - 0.5) * cell.width, clientY: rect.top + (y - 0.5) * cell.height }));
  };
  const record = name => {
    trace.push({ name, pointer: screen.classList.contains('xterm-cursor-pointer'), calls: calls.splice(0),
      errors: errors.splice(0).map(error => error === marker ? 'marker' : error.name) });
  };
  try {
    terminal.options.linkHandler = handler;
    terminal.open(parent);
    terminal.focus();
    await new Promise(resolve => terminal.write('\x1b]8;;custom:resource\x1b\\abcdefg\x1b]8;;\x1b\\hijklmnopqrs!', resolve));
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
    record('mousedown and current observe same mutation');
    mouse('mousedown', 13, 1, 2);
    trace.push({ name: 'double click selects current mutated range', selection: terminal.getSelection() });
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
    trace.push({ name: 'hit test reads live proxy coordinates', reads: reads.splice(0) });
    const start = range.start;
    range.start = new Proxy({}, { get(target, key) { if (key === 'y') throw marker; return start[key]; } });
    mouse('mousemove', 18);
    record('hit test preserves thrown identity');
    range.start = { x: 16, y: 1 }; range.end = { x: 18, y: 1 };
    handler.leave = function(event, text, value) { calls.push(['new leave', this === handler, value === firstRange]); };
    mouse('mousemove', 15);
    record('replaced handler callback remains live');
    mouse('mouseleave', 15);
    terminal.reset();
    terminal.resize(10, 4);
    firstRange = undefined;
    handler.hover = function(event, text, value) { range = value; firstRange ??= value; calls.push(['wrapped hover', value === firstRange]); };
    await new Promise(resolve => terminal.write('\r\n'.repeat(6) + '\x1b]8;;custom:wrapped\x1b\\abcdefghijklmnopq\x1b]8;;\x1b\\!', resolve));
    await render();
    mouse('mousemove', 3, 3);
    record('wrapped link with scrollback');
    const offset = terminal.buffer.active.viewportY;
    trace.push({ name: 'wrapped coordinates', scrollback: offset > 0,
      start: { x: range.start.x, y: range.start.y - offset }, end: { x: range.end.x, y: range.end.y - offset } });
    range.start = { x: 5, y: offset + 4 }; range.end = { x: 7, y: offset + 4 };
    mouse('mousemove', 4, 3);
    record('wrapped mutation leaves prior row');
    mouse('mousemove', 6, 4);
    record('wrapped mutation reenters new row');
    // Changing rows asks the provider again and creates a new range. Mutate
    // that callback's current object before selection, not the prior cache.
    range.start = { x: 5, y: offset + 4 }; range.end = { x: 7, y: offset + 4 };
    mouse('mousedown', 6, 4, 2);
    trace.push({ name: 'wrapped selection samples mutation', selection: terminal.getSelection() });
    terminal.dispose();
    screen.dispatchEvent(new MouseEvent('mousemove', { bubbles: true }));
    record('disposed terminal has no pointer listeners');
    return trace;
  } finally {
    terminal.dispose();
    parent.remove();
    window.removeEventListener('error', capture);
  }
})();
