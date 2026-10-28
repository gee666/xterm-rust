import { DemoAddons, addonTypes, requiredMethod } from './addons.mjs';
import { setupControls } from './controls.mjs';

const byId = id => document.getElementById(id);
const LOG_ENTRIES = 200;
const sample = '\x1b[2J\x1b[H\x1b[1mRust terminal\x1b[0m: \x1b[31mred\x1b[0m \x1b[32mgreen\x1b[0m\r\n'
  + 'Unicode: A界e\u0301 😀\r\nSearch Rust across real buffer cells.\r\n'
  + '\x1b[4;1HCURSOR\x1b[4;1H>\x1b[6;1Hhttps://xtermjs.org/\r\n';
const state = { bindings: undefined, terminal: undefined, addons: undefined, handles: [], log: [],
  outgoing: [], errors: [], searchResults: undefined, controls: undefined, disposed: false };
function log(name, value) {
  state.log.push({ name, value });
  if (state.log.length > LOG_ENTRIES) state.log.shift();
  byId('events').textContent = state.log.map(item => `${item.name} ${JSON.stringify(item.value)}`).join('\n');
  if (name === 'addon:search:onDidChangeResults') state.searchResults = value;
}
function terminal() {
  if (!state.terminal) throw new Error('No active terminal. Reopen the session first.');
  return state.terminal;
}
function addons() {
  if (!state.addons) throw new Error('No active addon manager. Reopen the session first.');
  return state.addons;
}
function snapshot() {
  const current = terminal();
  const active = current.buffer.active;
  const lines = [];
  for (let row = 0; row < current.rows; row++) {
    const line = requiredMethod(active, 'getLine')(active.viewportY + row);
    lines.push(line ? requiredMethod(line, 'translateToString')(true) : null);
  }
  return { cols: current.cols, rows: current.rows, cursorX: active.cursorX, cursorY: active.cursorY,
    viewportY: active.viewportY, baseY: active.baseY, bufferType: active.type,
    selection: requiredMethod(current, 'getSelection')(), lines, addons: addons().names() };
}
function refresh() {
  byId('addon-state').textContent = JSON.stringify(state.addons?.names() ?? []);
  if (!state.terminal) { byId('snapshot').textContent = 'Terminal disposed'; return; }
  byId('snapshot').textContent = JSON.stringify(snapshot(), null, 2);
}
function fail(name, error) {
  const message = `${name}: ${error?.stack ?? error}`;
  state.errors.push(message);
  byId('error').hidden = false;
  byId('error').textContent = message;
  byId('status').textContent = 'Action failed. See the error and lifecycle log.';
  log('error', message);
}
async function action(name, callback) {
  byId('error').hidden = true;
  try {
    const result = await callback();
    log(name, 'completed');
    byId('status').textContent = state.terminal ? 'Rust terminal ready' : 'Terminal disposed';
    return result;
  } catch (error) { fail(name, error); throw error; }
}
// DOM handlers observe rejected actions here. The visible error is retained;
// programmatic callers still receive the rejection instead of a false success.
function observe(promise) {
  promise.catch(error => log('action:rejected', String(error)));
}
function click(id, callback) {
  byId(id).addEventListener('click', () => observe(action(id, callback)));
}
function writeText(text) {
  return new Promise((resolve, reject) => {
    try { requiredMethod(terminal(), 'write')(text, () => { refresh(); resolve(); }); }
    catch (error) { reject(error); }
  });
}
async function close() {
  if (!state.terminal) return;
  const failures = [];
  try { state.addons.dispose(); } catch (error) { failures.push(error); }
  for (const handle of state.handles) {
    try { requiredMethod(handle, 'dispose')(); } catch (error) { failures.push(error); }
  }
  state.handles = [];
  try { requiredMethod(state.terminal, 'dispose')(); } catch (error) { failures.push(error); }
  if (failures.length) throw new AggregateError(failures, 'Terminal teardown failed');
  state.terminal = undefined; state.addons = undefined; state.disposed = true;
  refresh();
}
async function open() {
  if (state.terminal) await close();
  const current = new state.bindings.Terminal({ cols: 80, rows: 24, allowProposedApi: true,
    fontFamily: 'monospace', fontSize: 15, scrollback: 1000 });
  state.terminal = current;
  state.addons = new DemoAddons(state.bindings, current, log, refresh);
  state.disposed = false;
  state.outgoing = [];
  byId('outgoing').textContent = '';
  requiredMethod(current, 'open')(byId('terminal-container'));
  state.handles.push(requiredMethod(current, 'onData')(data => {
    state.outgoing.push(data);
    byId('outgoing').textContent = state.outgoing.map(value => JSON.stringify(value)).join('\n');
    log('outgoing:data', data);
  }));
  for (const name of ['onBinary', 'onResize', 'onTitleChange']) {
    state.handles.push(requiredMethod(current, name)(value => log(name, value)));
  }
  state.controls.rebuildOptions();
  await writeText(sample);
  requiredMethod(current, 'focus')();
  refresh();
}
const app = { terminal, addons, log, refresh, isOpen: () => !!state.terminal,
  action: (name, callback) => { const result = action(name, callback); observe(result); return result; },
  get searchResults() { return state.searchResults; } };
state.controls = setupControls(app);
for (const name of Object.keys(addonTypes)) {
  const option = document.createElement('option'); option.value = name; option.textContent = name;
  byId('addon-select').append(option);
}
click('sample', () => writeText(sample));
click('reset', () => { terminal().reset(); refresh(); });
click('dispose', close);
click('reopen', open);
byId('write-form').addEventListener('submit', event => {
  event.preventDefault();
  observe(action('write', () => {
    const value = byId('write-text').value;
    const text = value.startsWith('"') ? JSON.parse(value) : value;
    if (typeof text !== 'string') throw new TypeError('Display input must be a string');
    return writeText(text);
  }));
});
byId('input-form').addEventListener('submit', event => {
  event.preventDefault();
  observe(action('input', () => { requiredMethod(terminal(), 'input')(byId('input-text').value, true); refresh(); }));
});
const ready = (async () => {
  state.bindings = await import('../xterm.mjs');
  await state.bindings.initialize();
  await open();
  byId('status').textContent = 'Rust terminal ready';
})();
ready.catch(error => fail('WASM initialization', error));
window.demo = { ready, action, snapshot, write: writeText, open, close,
  get terminal() { return state.terminal; }, get addons() { return state.addons; },
  get outgoing() { return [...state.outgoing]; }, get errors() { return [...state.errors]; },
  get events() { return [...state.log]; } };
