// Same program runs with the original and the exported Rust Terminal.
return (async () => {
  const trace = [];
  const parent = document.createElement('div');
  document.body.append(parent);
  const calls = [];
  const errors = [];
  const capture = event => { errors.push(event.error); event.preventDefault(); };
  window.addEventListener('error', capture);
  const assert = (value, message) => { if (!value) throw new Error(message); };
  const point = (x, y) => ({ x, y });
  const range = (a, b, y = 1) => ({ start: point(a, y), end: point(b, y) });
  let screen;
  let cell;
  const mouse = (kind, x = 5, y = 1) => {
    const rect = screen.getBoundingClientRect();
    screen.dispatchEvent(new MouseEvent(kind, { bubbles: true, clientX: rect.left + (x - 0.5) * cell.width,
      clientY: rect.top + (y - 0.5) * cell.height }));
  };
  const pointer = () => screen.classList.contains('xterm-cursor-pointer');
  const record = (name, data) => trace.push({ name, ...data });
  let raw;
  const provider = {
    get provideLinks() {
      calls.push('get provideLinks');
      return function (y, cb) {
        calls.push(`provide ${y} ${this === provider}`);
        cb(y === 1 ? [raw] : undefined);
      };
    }
  };
  const token = terminal.registerLinkProvider(provider);
  record('before open', { calls: [...calls], disposable: typeof token.dispose });
  raw = {
    text: 'first', range: range(3, 7),
    activate(event, text) { calls.push(`activate ${text} ${this === raw} ${event.type}`); },
    hover(event, text) { calls.push(`hover ${text} ${this === raw} ${event.type}`); },
    leave(event, text) { calls.push(`leave ${text} ${this === raw} ${event.type}`); },
    dispose() { calls.push(`dispose ${this === raw}`); }
  };
  try {
    terminal.open(parent);
    terminal.focus();
    await new Promise(resolve => terminal.write('foo bar baz\r\nsecond line', resolve));
    await new Promise((resolve, reject) => {
      const timer = setTimeout(() => { subscription.dispose(); reject(new Error('render deadline')); }, 10000);
      const subscription = terminal.onRender(() => { clearTimeout(timer); subscription.dispose(); resolve(); });
      terminal.refresh(0, terminal.rows - 1);
    });
    screen = terminal.element.querySelector('.xterm-screen');
    cell = terminal.dimensions.css.cell;
    mouse('mousemove');
    const initial = raw.decorations;
    record('initial hover', { calls: [...calls], pointer: pointer(), underline: initial.underline,
      pointerValue: initial.pointerCursor, keys: Object.keys(initial),
      descriptors: ['underline', 'pointerCursor'].map(key => {
        const descriptor = Object.getOwnPropertyDescriptor(initial, key);
        return [key, descriptor.enumerable, descriptor.configurable, typeof descriptor.get, typeof descriptor.set];
      }) });
    assert(pointer(), 'initial link pointer');
    calls.length = 0;
    raw.text = 'x\ud800\udfff';
    const savedActivate = raw.activate;
    raw.activate = function (_, text) { calls.push({ receiver: this === raw, units: Array.from({ length: text.length }, (_, i) => text.charCodeAt(i)) }); };
    mouse('mousedown'); mouse('mouseup');
    record('lossless callback text', { calls: [...calls] });
    raw.activate = savedActivate;
    calls.length = 0;
    raw.text = 'changed';
    raw.activate = function (event, text) { calls.push(`new activate ${text} ${this === raw} ${event.type}`); };
    mouse('mousedown');
    raw.range = range(8, 10);
    raw.text = 'after down';
    mouse('mouseup', 9);
    record('live mouseup', { calls: [...calls], pointer: pointer() });
    calls.length = 0;
    // The source ignores the same cell even if the range changed under it.
    mouse('mousemove');
    record('same cell', { calls: [...calls], pointer: pointer() });
    mouse('mousemove', 6);
    record('mutated range leaves', { calls: [...calls], pointer: pointer(), oldUndefined: initial.underline === undefined });
    calls.length = 0;
    raw.hover = function (_, text) { calls.push(`new hover ${text} ${this === raw}`); };
    mouse('mousemove', 9);
    const decoration = raw.decorations;
    const flag = { marker: true };
    decoration.pointerCursor = flag;
    decoration.underline = false;
    record('raw decorations', { calls: [...calls], pointer: pointer(), sameFlag: decoration.pointerCursor === flag,
      underline: decoration.underline, oldNowFlag: initial.pointerCursor === flag });
    decoration.pointerCursor = undefined;
    decoration.underline = undefined;
    record('undefined decorations', { pointer: pointer(), hasPointer: decoration.pointerCursor !== undefined,
      hasUnderline: decoration.underline !== undefined });
    mouse('mouseleave');
    record('leave values', { pointer: pointer(), getterUndefined: decoration.underline === undefined });
    // Read only the expressions used by hit testing, in source order.
    const reads = [];
    const values = { start: point(3, 1), end: point(7, 1) };
    const liveRange = new Proxy(values, { get(target, key) {
      reads.push(String(key));
      return new Proxy(target[key], { get(point, axis) { reads.push(String(axis)); return point[axis]; } });
    } });
    raw.range = liveRange;
    Object.defineProperty(raw, 'range', { configurable: true, get() { reads.push('range'); return liveRange; } });
    mouse('mousemove', 4);
    record('getter reads', { reads: [...reads], pointer: pointer() });
    Object.defineProperty(raw, 'range', { configurable: true, writable: true, value: range(3, 7) });
    mouse('mouseleave');
    raw.decorations = { underline: false };
    calls.length = 0;
    raw.hover = function () { raw.decorations.pointerCursor = true; calls.push('hover wrote pre-accessor field'); };
    mouse('mousemove', 5);
    record('partial decorations timing', { calls: [...calls], pointer: pointer(), underline: raw.decorations.underline,
      pointerUndefined: raw.decorations.pointerCursor === undefined });
    mouse('mouseleave');
    calls.length = 0;
    token.dispose();
    // Repeated token disposal removes both registrations of one JS identity.
    let duplicates = 0;
    const duplicate = { provideLinks(_, cb) { duplicates++; cb(undefined); } };
    const first = terminal.registerLinkProvider(duplicate);
    terminal.registerLinkProvider(duplicate);
    first.dispose(); first.dispose();
    mouse('mousemove', 3, 2);
    record('duplicate disposal', { duplicates });
    // Priority waits for higher-priority async providers, without eager reads.
    let highReply;
    let lowReply;
    const high = terminal.registerLinkProvider({ provideLinks(_, cb) { highReply = cb; } });
    const low = terminal.registerLinkProvider({ provideLinks(_, cb) { lowReply = cb; } });
    mouse('mousemove', 5, 1);
    const asyncLink = { range: range(3, 7), text: 'async', activate() {}, hover() { calls.push('async hover'); } };
    lowReply([asyncLink]);
    record('priority pending', { calls: [...calls], pointer: pointer() });
    highReply(undefined);
    record('priority resolved', { calls: [...calls], pointer: pointer() });
    mouse('mouseleave');
    let ignoredMaps = 0;
    const ignored = { get map() { ignoredMaps++; throw new Error('ignored map read'); } };
    lowReply(ignored);
    record('ignored reply', { ignoredMaps, errors: errors.length });
    high.dispose(); low.dispose();
    // Provider getter and late callback exceptions keep the thrown object.
    const marker = { marker: 'provider' };
    const throwing = terminal.registerLinkProvider({ get provideLinks() { throw marker; } });
    mouse('mousemove', 6, 2);
    record('provider error', { errors: errors.length, identity: errors.at(-1) === marker });
    throwing.dispose();
    errors.length = 0;
    let pending;
    const pendingToken = terminal.registerLinkProvider({ provideLinks(_, cb) { pending = cb; } });
    mouse('mousemove', 4, 1);
    const getterMarker = { marker: 'range' };
    let caught;
    try { pending([{ get range() { throw getterMarker; } }]); } catch (error) { caught = error; }
    record('range error', { identity: caught === getterMarker, errors: errors.length });
    const decorationsMarker = { marker: 'decorations' };
    const leaveLog = [];
    let decorationsCaught;
    try { pending([{ range: range(3, 7), text: 'partial', activate() {},
      get decorations() { throw decorationsMarker; }, leave() { leaveLog.push('leave'); } }]); }
    catch (error) { decorationsCaught = error; }
    mouse('mouseleave');
    record('decoration getter partial state', { identity: decorationsCaught === decorationsMarker,
      leaveLog, errors: errors.length, pointer: pointer() });
    mouse('mousemove', 5, 2);
    const hoverMarker = { marker: 'hover' };
    const hoverThrow = { range: range(3, 7, 2), text: 'throw hover', activate() {}, hover() { throw hoverMarker; } };
    let hoverCaught;
    try { pending([hoverThrow]); } catch (error) { hoverCaught = error; }
    record('hover error partial state', { identity: hoverCaught === hoverMarker, pointer: pointer(),
      descriptorsInstalled: Object.hasOwn(hoverThrow, 'decorations') });
    pendingToken.dispose();
    terminal.dispose();
    let disposedMaps = 0;
    const disposedMarker = { marker: 'disposed map' };
    let disposedError;
    try { pending({ get map() { disposedMaps++; throw disposedMarker; } }); } catch (error) { disposedError = error; }
    record('disposed reply', { disposedMaps, identity: disposedError === disposedMarker });
    assert(trace.length === 20, `trace phase count ${trace.length}`);
    return trace;
  } finally {
    window.removeEventListener('error', capture);
    terminal.dispose();
    parent.remove();
  }
})();
