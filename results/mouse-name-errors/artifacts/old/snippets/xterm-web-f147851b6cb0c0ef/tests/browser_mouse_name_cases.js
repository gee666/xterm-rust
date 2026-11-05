// Identical cases exercise the unchanged source Terminal and Rust browser ABI.
const units = value => Array.from({length:value.length}, (_, index) => value.charCodeAt(index));
const keys = [['activeProtocol', 'protocol'], ['activeEncoding', 'encoding']];
const state = mouse => [mouse.activeProtocol, mouse.activeEncoding,
  mouse.areMouseEventsActive, mouse.isDefaultEncoding, mouse.isPixelEncoding];

function reject(mouse, key, kind, name) {
  let error;
  try { mouse[key] = name; } catch (caught) { error = caught; }
  const expected = `unknown ${kind} "${name}"`;
  if (!(error instanceof Error) || error.constructor !== Error || error.message !== expected) {
    throw new Error(`${key} error changed UTF16 units for ${JSON.stringify(units(name))}`);
  }
  return error;
}

export function unknownMouseStringNames(term) {
  try {
    const mouse = term._core.mouseStateService;
    const events = [];
    const listener = mouse.onProtocolChange(value => events.push(value));
    mouse.activeProtocol = 'VT200';
    mouse.activeEncoding = 'SGR_PIXELS';
    const before = state(mouse);
    const names = ['', '\0', '"', 'a\nb\r\tc', '\ud800', '\udfff', 'x\udbffy',
      '\udc00\ud800', '\ud800\ud800', '\udfff\udfff', '\ud800\udc00',
      '\udbff\udfff', 'NONE\0', 'SGR\ud800', '\ufffd', 'é中'];
    const errors = keys.map(([key, kind]) => names.map(name => {
      const error = reject(mouse, key, kind, name);
      return [units(name), error.name, units(error.message),
        units(Error.prototype.toString.call(error)), Object.keys(error)];
    }));
    const after = state(mouse);
    listener.dispose();
    return {before, after, events, errors};
  } finally { term.dispose(); }
}

export function everyLoneMouseNameSurrogate(term) {
  try {
    const mouse = term._core.mouseStateService;
    const events = [];
    const listener = mouse.onProtocolChange(value => events.push(value));
    mouse.activeProtocol = 'ANY';
    mouse.activeEncoding = 'SGR';
    let checked = 0;
    for (let unit = 0xd800; unit <= 0xdfff; unit++) {
      for (const [key, kind] of keys) {
        reject(mouse, key, kind, String.fromCharCode(unit));
        checked++;
      }
    }
    const result = {checked, events, final:state(mouse)};
    listener.dispose();
    return result;
  } finally { term.dispose(); }
}

export function seededAndLongMouseNames(term) {
  try {
    const mouse = term._core.mouseStateService;
    let seed = 0x6d6f7573;
    const next = () => (seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0);
    let checked = 0;
    const lengths = [];
    for (let index = 0; index < 128; index++) {
      const length = 1 + next() % 48;
      const name = 'unknown:' + String.fromCharCode(...Array.from({length}, next));
      lengths.push(name.length);
      for (const [key, kind] of keys) { reject(mouse, key, kind, name); checked++; }
    }
    // Cross the existing UTF16 transport chunk size, including each surrogate
    // class and a genuine pair at both ends. No generated name is a builtin.
    const longName = '\ud800\udc00' + '\udfff'.repeat(8193) + '\udbff\udfff';
    for (const [key, kind] of keys) { reject(mouse, key, kind, longName); checked++; }
    return {initialSeed:0x6d6f7573, finalSeed:seed, checked, lengths,
      longLength:longName.length, final:state(mouse)};
  } finally { term.dispose(); }
}

export function reentrantMouseNameFailure(term) {
  try {
    const mouse = term._core.mouseStateService;
    const seen = [];
    let nestedError;
    const first = mouse.onProtocolChange(mask => {
      seen.push(['first', mask, state(mouse)]);
      nestedError = reject(mouse, 'activeEncoding', 'encoding', 'SGR\ud800');
      mouse.activeEncoding = 'SGR_PIXELS';
      throw nestedError;
    });
    const second = mouse.onProtocolChange(mask => seen.push(['second', mask]));
    let sameError = false;
    try { mouse.activeProtocol = 'X10'; } catch (error) { sameError = error === nestedError; }
    const committed = state(mouse);
    first.dispose(); second.dispose();
    // A direct failed protocol assignment neither dispatches nor alters modes.
    const error = reject(mouse, 'activeProtocol', 'protocol', '\ud800NONE');
    mouse.reset();
    return {sameError, seen, committed, message:units(error.message), reset:state(mouse)};
  } finally { term.dispose(); }
}

export async function parsedModesSurviveMouseNameErrors(term) {
  try {
    const mouse = term._core.mouseStateService;
    const write = value => new Promise(resolve => term.write(value, resolve));
    const states = [];
    for (const sequence of ['\x1b[?1000h', '\x1b[?1006h', '\x1b[?1016h',
      '\x1b[?1003h', '\x1b[?1003l']) {
      await write(sequence);
      const before = state(mouse);
      for (const [key, kind] of keys) reject(mouse, key, kind, 'missing\udc00');
      states.push([before, state(mouse), mouse === term._core.mouseStateService]);
    }
    term.reset();
    return {states, reset:state(mouse)};
  } finally { term.dispose(); }
}
