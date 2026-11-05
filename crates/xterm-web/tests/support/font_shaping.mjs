// Actual loaded OpenType font, Rust terminal writes and renderer-owned joiners.
function nextRender(terminal) {
  let subscription, timeout;
  const promise = new Promise((resolve, reject) => {
    timeout = setTimeout(() => {
      subscription.dispose();
      reject(new Error('Font terminal did not render within 5 seconds'));
    }, 5000);
    subscription = terminal.onRender(() => {
      clearTimeout(timeout);
      subscription.dispose();
      resolve();
    });
  });
  return { promise, cancel() { clearTimeout(timeout); subscription.dispose(); } };
}
function spans(terminal) {
  return [...terminal.element.querySelector('.xterm-rows').firstElementChild.children]
    .map(element => element.textContent).filter(text => text.trim());
}
export async function terminalShapedFont(terminal, addon, bytes) {
  const parent = document.createElement('div');
  const owner = Object.getOwnPropertyDescriptor(window, 'queryLocalFonts');
  const face = new FontFace('Fira Code', bytes);
  let blobReads = 0, release;
  const pendingBlob = new Promise(resolve => { release = resolve; });
  const metadata = [{ family: 'Fira Code', blob() { blobReads++; return pendingBlob; } }];
  let render;
  document.body.appendChild(parent);
  try {
    await face.load();
    document.fonts.add(face);
    Object.defineProperty(window, 'queryLocalFonts', { configurable: true, value: async () => metadata });
    terminal.open(parent);
    await new Promise(resolve => terminal.write('a===b', resolve));
    addon.activate(terminal);
    render = nextRender(terminal);
    terminal.refresh(0, terminal.rows - 1);
    await render.promise;
    const before = spans(terminal);
    render = nextRender(terminal);
    release(new Blob([bytes]));
    // The addon's successful font load must refresh the real renderer itself.
    await render.promise;
    const loaded = spans(terminal);
    const style = getComputedStyle(terminal.element.querySelector('.xterm-rows').firstElementChild.children[1]);
    const computedFamily = style.fontFamily, computedFeatures = style.fontFeatureSettings;
    const expected = document.createElement('div');
    expected.style.fontFeatureSettings = '"calt" on';
    const expectedFeatures = expected.style.fontFeatureSettings;
    addon.dispose();
    render = nextRender(terminal);
    terminal.refresh(0, terminal.rows - 1);
    await render.promise;
    return JSON.stringify({ before, loaded, disposed: spans(terminal), blobReads,
      fontStatus: face.status, fontReady: document.fonts.check('20px "Fira Code"'),
      computedFamily, computedFeatures, expectedFeatures });
  } finally {
    render?.cancel();
    terminal.dispose();
    parent.remove();
    document.fonts.delete(face);
    if (owner) Object.defineProperty(window, 'queryLocalFonts', owner);
    else delete window.queryLocalFonts;
  }
}
