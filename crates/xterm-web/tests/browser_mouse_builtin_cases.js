// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
// The same cases call the read-only source Terminal and actual Rust Terminal.
const protocols = ['NONE', 'X10', 'VT200', 'DRAG', 'ANY'];
const encodings = ['DEFAULT', 'SGR', 'SGR_PIXELS'];
const fields = ['action', 'button', 'ctrl', 'shift', 'alt', 'col', 'row', 'x', 'y'];
const event = () => ({action:1, button:0, ctrl:true, alt:true, shift:true,
  col:1, row:2, x:17, y:29});
const units = value => typeof value === 'string'
  ? Array.from({length:value.length}, (_, i) => value.charCodeAt(i)) : value;
const outcome = callback => {
  try { return ['ok', units(callback())]; }
  catch (error) { return ['throw', error?.constructor?.name ?? typeof error]; }
};

function builtinPolicies(term) {
  const mouse = term._core.mouseStateService;
  const restricted = [];
  const encoded = [];
  const values = [0, 1, 2, 3, 4, 8, 15, -1, 0.5, NaN, Infinity, '1', undefined];
  for (const protocol of protocols) {
    mouse.activeProtocol = protocol;
    for (const button of values) {
      for (const action of [0, 1, 32, '1']) {
        const e = {...event(), button, action};
        const extra = {};
        e.extra = extra;
        const result = outcome(() => mouse.restrictMouseEvent(e));
        restricted.push([protocol, result, e.ctrl, e.alt, e.shift, e.extra === extra]);
      }
    }
  }
  for (const encoding of encodings) {
    mouse.activeEncoding = encoding;
    for (const button of values) {
      for (const action of [0, 1, 32]) {
        encoded.push([encoding, outcome(() => mouse.encodeMouseEvent({...event(), button, action}))]);
      }
    }
    for (const col of [0, 223, 224, -32, -33, 0.5, NaN, Infinity, '2', undefined, '\ud800']) {
      encoded.push([encoding, outcome(() => mouse.encodeMouseEvent({...event(), col, x:col}))]);
    }
  }
  return {restricted, encoded};
}

function fieldOrdering(term) {
  const mouse = term._core.mouseStateService;
  const results = [];
  const traceEvent = trace => {
    const target = event();
    return new Proxy(target, {
      get(object, name) { trace.push(['get', name]); return object[name]; },
      set(object, name, value) { trace.push(['set', name, value]); object[name] = value; return true; },
    });
  };
  for (const protocol of protocols) {
    mouse.activeProtocol = protocol;
    const trace = [];
    results.push([protocol, outcome(() => mouse.restrictMouseEvent(traceEvent(trace))), trace]);
  }
  for (const encoding of encodings) {
    mouse.activeEncoding = encoding;
    const trace = [];
    const e = traceEvent(trace);
    e.button = {[Symbol.toPrimitive](hint) { trace.push(['button coercion', hint]); return 1; }};
    e.col = e.x = {[Symbol.toPrimitive](hint) { trace.push(['coordinate coercion', hint]); return 2; }};
    trace.length = 0;
    results.push([encoding, outcome(() => mouse.encodeMouseEvent(e)), trace]);
  }
  return results;
}

function failures(term) {
  const mouse = term._core.mouseStateService;
  const results = [];
  const thrown = [undefined, null, false, NaN, 3n, Symbol('thrown'), {}, new Error('sentinel')];
  const identity = (callback, sentinel) => {
    try { callback(); return [false, false]; }
    catch (error) { return [true, Object.is(error, sentinel)]; }
  };
  for (const encoding of encodings) {
    mouse.activeEncoding = encoding;
    for (const key of fields) {
      for (const sentinel of thrown) {
        const e = event();
        Object.defineProperty(e, key, {get() { throw sentinel; }});
        results.push([encoding, key, identity(() => mouse.encodeMouseEvent(e), sentinel)]);
      }
    }
    for (const key of ['button', 'col', 'x']) {
      for (const value of [2n, Symbol('input')]) {
        results.push([encoding, key, typeof value,
          outcome(() => mouse.encodeMouseEvent({...event(), [key]:value}))]);
      }
    }
    for (const value of [null, undefined, true, 3, 'event', Symbol('event')]) {
      results.push([encoding, typeof value, outcome(() => mouse.encodeMouseEvent(value))]);
    }
  }
  for (const protocol of protocols) {
    mouse.activeProtocol = protocol;
    for (const key of fields) {
      for (const sentinel of thrown) {
        const e = event();
        Object.defineProperty(e, key, {get() { throw sentinel; }});
        results.push([protocol, key, identity(() => mouse.restrictMouseEvent(e), sentinel)]);
      }
    }
  }
  mouse.activeProtocol = 'X10';
  for (const key of ['ctrl', 'alt', 'shift']) {
    for (const sentinel of thrown) {
      const e = event();
      Object.defineProperty(e, key, {set() { throw sentinel; }});
      results.push(['X10 setter', key, identity(() => mouse.restrictMouseEvent(e), sentinel)]);
    }
  }
  return results;
}

const run = (term, callback) => { try { return callback(term); } finally { term.dispose(); } };
export const liveBuiltinMousePolicies = term => run(term, builtinPolicies);
export const rawMouseFieldOrdering = term => run(term, fieldOrdering);
export const rawMouseFailures = term => run(term, failures);
export async function mousePolicyReentryAndParsedModes(term) {
  try { return await reentryAndParsedModes(term); } finally { term.dispose(); }
}

async function reentryAndParsedModes(term) {
  const mouse = term._core.mouseStateService;
  const results = [];
  mouse.activeEncoding = 'SGR';
  const e = event();
  Object.defineProperty(e, 'action', {get() {
    mouse.activeEncoding = 'DEFAULT';
    results.push(['nested encoding', units(mouse.encodeMouseEvent(event()))]);
    return 1;
  }});
  results.push(['outer encoding', units(mouse.encodeMouseEvent(e)), mouse.activeEncoding]);
  mouse.activeProtocol = 'X10';
  const p = event();
  Object.defineProperty(p, 'ctrl', {set(value) {
    mouse.activeProtocol = 'NONE';
    results.push(['nested restriction', mouse.restrictMouseEvent(event()), value]);
  }});
  results.push(['outer restriction', mouse.restrictMouseEvent(p), p.alt, p.shift, mouse.activeProtocol]);
  const write = value => new Promise(resolve => term.write(value, resolve));
  for (const sequence of ['\x1b[?9h', '\x1b[?1000h', '\x1b[?1002h', '\x1b[?1003h',
    '\x1b[?1006h', '\x1b[?1016h', '\x1b[?1016l']) {
    await write(sequence);
    const e = event();
    results.push(['parsed', mouse.activeProtocol, mouse.activeEncoding,
      mouse.restrictMouseEvent(e), units(mouse.encodeMouseEvent(e)), e.ctrl, e.alt, e.shift]);
  }
  mouse.reset();
  results.push(['reset', mouse.restrictMouseEvent(event()), units(mouse.encodeMouseEvent(event()))]);
  term.dispose();
  results.push(['disposed', mouse.restrictMouseEvent(event()), units(mouse.encodeMouseEvent(event()))]);
  return results;
}
