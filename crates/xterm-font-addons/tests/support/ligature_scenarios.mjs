// Shared operation inputs for the source oracle and Rust/WASM addon tests.
// This is a terminal boundary probe, not an implementation of shaping or matching.
function terminal(family = '') {
  const trace = [], handlers = new Map();
  let next = 0, observe;
  const options = { logLevel: 'info', get fontFamily() {
    if (observe) { const done = observe; observe = undefined; done(); }
    return family;
  }, set fontFamily(value) { family = value; } };
  return { trace, handlers, options, rows: 5, element: { style: {} },
    registerCharacterJoiner(callback) { const id = next++; handlers.set(id, callback); trace.push(['register', id]); return id; },
    deregisterCharacterJoiner(id) { trace.push(['deregister', id]); return handlers.delete(id); },
    refresh(start, end) { trace.push(['refresh', start, end]); },
    join(text, id = next - 1) { return handlers.get(id)(text); },
    nextRead() { return new Promise(resolve => { observe = resolve; }); }
  };
}
function error(callback) {
  try { callback(); return null; } catch (e) { return [e.name, e.message]; }
}
function copy(value) { return JSON.parse(JSON.stringify(value)); }

export function lifecycleScenarios(create) {
  const result = {};
  const unopened = terminal(); unopened.element = undefined;
  const unopenedAddon = create();
  result.beforeOpen = error(() => unopenedAddon.activate(unopened));
  result.beforeOpenTrace = unopened.trace;
  unopenedAddon.dispose();
  const addon = create(), term = terminal();
  addon.activate(term);
  result.defaultStyle = term.element.style.fontFeatureSettings;
  result.defaults = term.join('😀 <----> === !=== </> ->> ::: <!---');
  addon.dispose(); addon.dispose();
  result.disposedStyle = term.element.style.fontFeatureSettings;
  result.lifecycle = term.trace;
  result.remaining = term.handlers.size;
  const patterns = ['=', '==', '===', '->'];
  const custom = create({ fallbackLigatures: patterns, fontFeatureSettings: '"liga" off' });
  const other = terminal(); custom.activate(other);
  result.sortedInput = patterns.slice();
  result.customStyle = other.element.style.fontFeatureSettings;
  result.customRanges = other.join('😀====->');
  patterns.unshift('😀');
  result.livePatterns = other.join('😀====->');
  custom.dispose();
  const disabled = create({ fallbackLigatures: [] }), empty = terminal();
  disabled.activate(empty); result.disabled = empty.join('===->'); disabled.dispose();
  // Reactivation intentionally preserves the first registration, as upstream.
  const repeated = create(), first = terminal(), second = terminal();
  repeated.activate(first); repeated.activate(second); repeated.dispose();
  result.reactivation = [first.trace, second.trace, first.handlers.size, second.handlers.size];
  first.handlers.clear();
  const retried = create(), throwing = terminal(); retried.activate(throwing);
  const deregister = throwing.deregisterCharacterJoiner;
  let fail = true;
  throwing.deregisterCharacterJoiner = function (id) { if (fail) throw new Error('deregister failed'); return deregister.call(this, id); };
  result.disposeError = error(() => retried.dispose());
  result.styleAfterError = throwing.element.style.fontFeatureSettings;
  fail = false; retried.dispose(); retried.dispose(); result.disposeRetry = throwing.trace;
  const setterFailure = create(), badStyle = terminal();
  Object.defineProperty(badStyle.element.style, 'fontFeatureSettings', { set() { throw new Error('style failed'); } });
  result.styleError = error(() => setterFailure.activate(badStyle));
  result.styleRegistration = badStyle.trace.slice();
  result.clearStyleError = error(() => setterFailure.dispose());
  result.styleDispose = badStyle.trace;
  const reentrant = create(), recursive = terminal(); reentrant.activate(recursive);
  let entered = false, nested, outer;
  Object.defineProperty(recursive.options, 'fontFamily', { get() {
    if (!entered) { entered = true; nested = recursive.join('==='); }
    return '';
  } });
  const recursionError = error(() => { outer = recursive.join('->'); });
  result.reentrant = { error: recursionError, nested, outer };
  reentrant.dispose();
  return result;
}

function deferred() { let resolve, reject; const promise = new Promise((a, b) => { resolve = a; reject = b; }); return { promise, resolve, reject }; }

export async function loadingScenarios(create, bytes) {
  const result = {}, controls = new Map();
  const names = ['Current', 'StaleSuccess', 'StaleFailure', 'Later', 'Broken', 'AfterDispose'];
  const metadata = names.map(family => ({ family, blob() {
    const control = controls.get(family); control.started.resolve(); return control.pending.promise;
  } }));
  for (const name of names) controls.set(name, { started: deferred(), pending: deferred() });
  const nav = Object.getOwnPropertyDescriptor(globalThis, 'navigator');
  const windowDescriptor = Object.getOwnPropertyDescriptor(globalThis, 'window');
  const owner = globalThis.window ?? {};
  const queryDescriptor = Object.getOwnPropertyDescriptor(owner, 'queryLocalFonts');
  Object.defineProperty(globalThis, 'navigator', { configurable: true, value: {} });
  if (!windowDescriptor) Object.defineProperty(globalThis, 'window', { configurable: true, value: owner });
  Object.defineProperty(owner, 'queryLocalFonts', { configurable: true, value: async () => metadata });
  const originalDebug = console.debug, logs = [];
  console.debug = (previous, message) => logs.push([previous === undefined ? 'undefined' : previous.message, message.message]);
  async function finish(term, family, fail = false) {
    const control = controls.get(family);
    await control.started.promise;
    const done = term.nextRead();
    if (fail) control.pending.reject(new Error(family + ' failed'));
    else control.pending.resolve({ arrayBuffer: async () => bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength) });
    await done;
  }
  try {
    const term = terminal('Current'), addon = create({ fallbackLigatures: ['ZZ'] }); addon.activate(term);
    result.loadingFallback = term.join('😀ZZ ==');
    await finish(term, 'Current');
    result.foundTrace = copy(term.trace);
    const ranges = term.join('😀=='); result.nativeRanges = copy(ranges);
    if (ranges.length) { ranges[0][0] = 99; ranges.push([80, 81]); }
    result.clonedRanges = term.join('😀==');
    // Each family has its own pending blob; out-of-order completions must not win.
    term.options.fontFamily = 'StaleSuccess'; term.join('ZZ==');
    await controls.get('StaleSuccess').started.promise;
    term.options.fontFamily = 'Later'; term.join('ZZ==');
    await controls.get('Later').started.promise;
    await finish(term, 'StaleSuccess');
    result.afterStaleSuccess = [copy(term.trace), term.join('ZZ==')];
    await finish(term, 'Later'); result.afterCurrentSuccess = term.join('ZZ==');
    term.options.fontFamily = 'StaleFailure'; term.join('ZZ==');
    await controls.get('StaleFailure').started.promise;
    term.options.fontFamily = 'Later';
    await finish(term, 'StaleFailure', true);
    // Re-selecting Later starts another read of its already fulfilled blob.
    term.join('ZZ=='); const laterRead = term.nextRead(); await laterRead;
    result.afterStaleFailure = [term.join('ZZ=='), logs.slice()];
    term.options.fontFamily = 'Broken'; term.options.logLevel = 'debug';
    result.beforeFailure = term.join('ZZ=='); await finish(term, 'Broken', true);
    result.afterFailure = [term.join('ZZ=='), logs.slice()];
    // Missing fonts resolve without refresh, and are not retried on every row.
    term.options.fontFamily = 'Missing'; term.join('ZZ=='); const missing = term.nextRead(); await missing;
    result.missing = [term.join('ZZ=='), copy(term.trace)];
    const noFallback = create({ fallbackLigatures: [] }), plain = terminal('Broken');
    noFallback.activate(plain);
    result.emptyWhileLoading = plain.join('a -> b www c');
    await plain.nextRead();
    result.emptyFailedFont = [plain.join('a -> b www c'), copy(plain.trace)];
    for (const family of ['notinstalled', 'monospace', {}]) {
      plain.options.fontFamily = family;
      const initial = plain.join('a -> b www c'); await plain.nextRead();
      const key = typeof family === 'string' ? family : 'malformed';
      result[key] = [initial, plain.join('a -> b www c'), copy(plain.trace)];
    }
    plain.options.fontFamily = '';
    result.noFamily = [plain.join('a -> b www c'), copy(plain.trace)];
    noFallback.dispose();
    term.options.fontFamily = 'AfterDispose'; term.join('ZZ==');
    await controls.get('AfterDispose').started.promise;
    addon.dispose(); await finish(term, 'AfterDispose');
    result.afterDispose = [copy(term.trace), term.element.style.fontFeatureSettings, term.handlers.size];
    return result;
  } finally {
    console.debug = originalDebug;
    if (nav) Object.defineProperty(globalThis, 'navigator', nav); else delete globalThis.navigator;
    if (!windowDescriptor) delete globalThis.window;
    if (queryDescriptor) Object.defineProperty(owner, 'queryLocalFonts', queryDescriptor); else delete owner.queryLocalFonts;
  }
}
