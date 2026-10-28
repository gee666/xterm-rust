// Browser-side test driver. Reference and Rust run in independent pages.
import { sourceSnapshot } from './source-snapshot.mjs';
const parameters = new URL(location.href).searchParams;
const implementation = parameters.get('implementation');
const events = [];
const inspectionErrors = [];
const handles = [];
const required = (object, name) => {
  if (typeof object?.[name] !== 'function') throw new Error(`Missing real ${name} binding`);
  return object[name].bind(object);
};
function encoded(value) {
  if (value === undefined) return { jsType: 'undefined' };
  if (typeof value === 'number' && !Number.isFinite(value)) return { jsType: String(value) };
  if (Object.is(value, -0)) return { jsType: 'negative-zero' };
  if (value === null || ['string', 'number', 'boolean'].includes(typeof value)) return value;
  if (Array.isArray(value)) return value.map(encoded);
  if (typeof value === 'object') return Object.fromEntries(Object.entries(value).map(([key, entry]) => [key, encoded(entry)]));
  throw new Error('Unsupported event value requires an explicit capture representation');
}
let terminal;
function snapshot() {
  return implementation === 'reference'
    ? sourceSnapshot(terminal, globalThis.XtermReference.CHARSETS)
    : required(terminal, '_acceptanceSnapshot')();
}
function properties() {
  const root = terminal.element;
  const screen = root.querySelector('.xterm-screen');
  const textarea = terminal.textarea;
  const bounds = element => {
    const box = element.getBoundingClientRect();
    return { x: box.x, y: box.y, width: box.width, height: box.height };
  };
  const style = getComputedStyle(screen);
  return { apiModes: encoded(terminal.modes),
    disableStdin: encoded(terminal.options.disableStdin), convertEol: encoded(terminal.options.convertEol),
    selection: required(terminal, 'getSelection')(),
    selectionPosition: encoded(required(terminal, 'getSelectionPosition')()),
    root: bounds(root), screen: bounds(screen), textarea: bounds(textarea),
    focused: document.activeElement === textarea,
    fontFamily: style.fontFamily, fontSize: style.fontSize, lineHeight: style.lineHeight,
    rootCount: document.querySelectorAll('.xterm').length,
    cursorCount: root.querySelectorAll('.xterm-cursor').length };
}
function record(type, value) {
  try { events.push({ type, value: encoded(value), state: snapshot(), properties: properties() }); }
  catch (error) { inspectionErrors.push({ type, error: error.stack ?? String(error) }); }
}
async function write(text, render = true) {
  const deadlineMs = 15000;
  await new Promise((resolve, reject) => {
    let parsed = false;
    let rendered = !render;
    let handle;
    const timeout = setTimeout(() => { handle?.dispose(); reject(new Error('Actual write/render completion timed out')); }, deadlineMs);
    function complete() {
      if (!parsed || !rendered) return;
      clearTimeout(timeout); handle?.dispose(); resolve();
    }
    try {
      if (render) handle = required(terminal, 'onRender')(() => {
        // Ignore an earlier startup frame. The source render service publishes
        // this write's viewport update after its parse/completion callback.
        if (parsed) { rendered = true; complete(); }
      });
      required(terminal, 'write')(text, () => { parsed = true; complete(); });
    } catch (error) { clearTimeout(timeout); handle?.dispose(); reject(error); }
  });
  if (inspectionErrors.length) throw new Error(JSON.stringify(inspectionErrors));
}
function renderAction(action) {
  return new Promise((resolve, reject) => {
    let handle;
    const timeout = setTimeout(() => {
      handle?.dispose(); reject(new Error('Actual render action timed out'));
    }, 15000);
    try {
      handle = required(terminal, 'onRender')(() => {
        clearTimeout(timeout); handle.dispose(); resolve();
      });
      action();
    } catch (error) { clearTimeout(timeout); handle?.dispose(); reject(error); }
  });
}
function capture(label) {
  if (inspectionErrors.length) throw new Error(JSON.stringify(inspectionErrors));
  return { label, state: snapshot(), properties: properties(), events: structuredClone(events) };
}
const ready = (async () => {
  const options = { cols: 24, rows: 6, scrollback: 20, allowProposedApi: true,
    fontFamily: 'monospace', fontSize: 14, cursorBlink: false };
  if (implementation === 'reference') terminal = new globalThis.XtermReference.Terminal(options);
  else if (implementation === 'rust') {
    const bindings = await import('/xterm.mjs'); await bindings.initialize();
    terminal = new bindings.Terminal(options);
  } else throw new Error('Unknown capture implementation');
  required(terminal, 'open')(document.getElementById('terminal'));
  await document.fonts.ready;
  for (const [name, type] of [['onData', 'data'], ['onBinary', 'binary'], ['onRender', 'render'],
    ['onResize', 'resize'], ['onScroll', 'scroll'], ['onCursorMove', 'cursor'],
    ['onLineFeed', 'linefeed'], ['onTitleChange', 'title'], ['onSelectionChange', 'selection']]) {
    handles.push(required(terminal, name)(value => record(type, value)));
  }
  handles.push(required(terminal.buffer, 'onBufferChange')(buffer => record('buffer', buffer.type)));
  if (implementation === 'reference') {
    const handler = terminal._core._inputHandler;
    for (const [method, name] of [['onRequestSyncScrollBar', 'syncScrollBar'], ['onRequestSendFocus', 'sendFocus'], ['onRequestReset', 'requestReset']]) {
      handles.push(required(handler, method)(() => record('internal', { name })));
    }
    handles.push(required(handler, 'onRequestRefreshRows')(range => record('internal', {
      name: 'refresh', start: range?.start ?? 0, end: range?.end ?? terminal.rows - 1
    })));
    handles.push(required(handler, 'onScroll')(position => record('internal', { name: 'inputScroll', position })));
  } else handles.push(required(terminal, '_acceptanceOnInternalEvent')(event => record('internal', event)));
  await write('\x1b[2J\x1b[H');
  required(terminal, 'focus')();
  await renderAction(() => required(terminal, 'refresh')(0, terminal.rows - 1));
  events.length = 0;
})();
window.captureSession = { ready, write, capture, renderAction, recordBusiness: value => record('business', value),
  get terminal() { return terminal; },
  clearEvents() { events.length = 0; inspectionErrors.length = 0; },
  async dispose() {
    for (const handle of handles) handle.dispose();
    handles.length = 0;
    required(terminal, 'dispose')();
    return { roots: document.querySelectorAll('.xterm').length };
  } };
ready.catch(error => { window.captureInitializationError = error.stack ?? String(error); });
