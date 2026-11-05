// Exercises only public methods on the actual exported Rust Terminal and addon.
function render(terminal) {
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      subscription.dispose();
      reject(new Error('Terminal did not render within 5 seconds'));
    }, 5000);
    const subscription = terminal.onRender(() => {
      clearTimeout(timeout);
      subscription.dispose();
      resolve();
    });
    terminal.refresh(0, terminal.rows - 1);
  });
}
function spans(terminal) {
  return [...terminal.element.querySelector('.xterm-rows').firstElementChild.children]
    .map(element => element.textContent).filter(text => text.trim());
}
export async function terminalJoinerTrace(terminal, addon) {
  const parent = document.createElement('div');
  document.body.appendChild(parent);
  try {
    terminal.open(parent);
    await new Promise(resolve => terminal.write('a===b', resolve));
    await render(terminal);
    const before = spans(terminal);
    const calls = [];
    const id = terminal.registerCharacterJoiner(text => { calls.push(text); return [[1, 4]]; });
    await render(terminal);
    const joined = spans(terminal);
    const removed = terminal.deregisterCharacterJoiner(id);
    await render(terminal);
    const after = spans(terminal);
    addon.activate(terminal);
    await render(terminal);
    const fallback = spans(terminal);
    const style = terminal.element.style.fontFeatureSettings;
    const expected = document.createElement('div');
    expected.style.fontFeatureSettings = '"calt" on';
    const expectedStyle = expected.style.fontFeatureSettings;
    addon.dispose();
    await render(terminal);
    const disposed = spans(terminal);
    return JSON.stringify({ before, joined, after, fallback, disposed, calls,
      style, expectedStyle, removedIsUndefined: removed === undefined,
      styleAfter: terminal.element.style.fontFeatureSettings });
  } finally {
    terminal.dispose();
    parent.remove();
  }
}
export async function terminalJoinerMutation(terminal) {
  const parent = document.createElement('div');
  document.body.appendChild(parent);
  try {
    terminal.open(parent);
    await new Promise(resolve => terminal.write('a===b', resolve));
    const calls = [];
    let second;
    let changed = false;
    terminal.registerCharacterJoiner(text => {
      calls.push(['first', text]);
      if (!changed) {
        changed = true;
        terminal.deregisterCharacterJoiner(second);
        terminal.registerCharacterJoiner(inner => { calls.push(['third', inner]); return [[1, 4]]; });
      }
      return [];
    });
    second = terminal.registerCharacterJoiner(text => { calls.push(['second', text]); return []; });
    await render(terminal);
    return JSON.stringify({ calls, spans: spans(terminal) });
  } finally {
    terminal.dispose();
    parent.remove();
  }
}
