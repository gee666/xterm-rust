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
    let index = 0, lastProvidedRow = 0, row;
    return cases((event, marker) => {
      provided = false; caught = [];
      screen.dispatchEvent(event);
      if (provided) lastProvidedRow = row;
      if (caught.length > 1) throw new Error(`Multiple DOM errors: ${caught}`);
      return { provided, error: caught.length === 0 ? 'none' : caught[0] === marker ? 'marker' : caught[0].name };
    }, () => {
      const rect = screen.getBoundingClientRect();
      // Blocked/error cases do not change the link cache's active row. Only
      // alternate after a real request, so every allowed case requests anew.
      const x = index++ % 16 + 1;
      row = lastProvidedRow === 1 ? 2 : 1;
      return new MouseEvent('mousemove', { bubbles: true,
        clientX: rect.left + (x - 0.5) * cell.width, clientY: rect.top + (row - 0.5) * cell.height });
    });
  } finally {
    token.dispose(); terminal.dispose(); parent.remove();
    window.removeEventListener('error', capture);
  }
})();
