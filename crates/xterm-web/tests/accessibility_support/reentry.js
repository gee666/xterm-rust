// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Real DOM overrides that reenter the actual Rust Terminal, not a terminal model.
export function observeResetRender(root, reset) {
  const rows = [...root.querySelectorAll('.xterm-accessibility-tree > div')];
  const first = rows[0], last = rows.at(-1);
  const text = Object.getOwnPropertyDescriptor(Node.prototype, 'textContent');
  return new Promise((resolve, reject) => {
    let fired = false, inReset = false, finished = false;
    const cleanup = () => {
      clearTimeout(timer);
      delete first.textContent;
      delete last.textContent;
    };
    const timer = setTimeout(() => {
      if (finished) return;
      finished = true;
      cleanup();
      reject(new Error('Reset render did not reach the last row'));
    }, 10000);
    Object.defineProperty(first, 'textContent', {
      configurable: true,
      get() { return text.get.call(this); },
      set(value) {
        if (!fired) {
          fired = true;
          inReset = true;
          try { reset(); } finally { inReset = false; }
        }
        text.set.call(this, value);
      }
    });
    Object.defineProperty(last, 'textContent', {
      configurable: true,
      get() { return text.get.call(this); },
      set(value) {
        text.set.call(this, value);
        if (!fired || inReset || finished) return;
        finished = true;
        const snapshot = rows.map(row => text.get.call(row));
        cleanup();
        resolve(snapshot);
      }
    });
  });
}

export function resizeOnComparison(node, resize) {
  const compare = node.compareDocumentPosition;
  let fired = false;
  Object.defineProperty(node, 'compareDocumentPosition', {
    configurable: true,
    value(other) {
      if (!fired) { fired = true; resize(); }
      return compare.call(this, other);
    }
  });
  return () => { delete node.compareDocumentPosition; };
}
