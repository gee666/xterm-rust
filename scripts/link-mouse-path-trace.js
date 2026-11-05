return (async () => {
  const parent = document.createElement('div');
  document.body.append(parent);
  let provided = false, caught = [];
  const capture = event => { caught.push(event.error); event.preventDefault(); };
  window.addEventListener('error', capture);
  const token = terminal.registerLinkProvider({ provideLinks(_y, callback) { provided = true; callback(undefined); } });
  try {
    terminal.open(parent);
    await new Promise(resolve => terminal.write('first row\r\nsecond row', resolve));
    await new Promise((resolve, reject) => {
      const timer = setTimeout(() => { subscription.dispose(); reject(new Error('render deadline')); }, 10000);
      const subscription = terminal.onRender(() => { clearTimeout(timer); subscription.dispose(); resolve(); });
      terminal.refresh(0, terminal.rows - 1);
    });
    const screen = terminal.element.querySelector('.xterm-screen');
    const cell = terminal.dimensions.css.cell;
    let index = 0;
    return cases((event, marker) => {
      provided = false; caught = [];
      screen.dispatchEvent(event);
      if (caught.length > 1) throw new Error(`Multiple DOM errors: ${caught}`);
      return { provided, error: caught.length === 0 ? 'none' : caught[0] === marker ? 'marker' : caught[0].name };
    }, () => {
      const rect = screen.getBoundingClientRect();
      const x = index % 16 + 1, y = index++ % 2 + 1;
      return new MouseEvent('mousemove', { bubbles: true,
        clientX: rect.left + (x - 0.5) * cell.width, clientY: rect.top + (y - 0.5) * cell.height });
    });
  } finally {
    token.dispose(); terminal.dispose(); parent.remove();
    window.removeEventListener('error', capture);
  }
})();
