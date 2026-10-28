// Test-only raw host cases shared by the unchanged source and Rust WASM tests.
export function scenarios() {
  const cases = [];
  for (const protocol of ['NONE', 'X10', 'VT200', 'DRAG', 'ANY']) {
    for (const encoding of ['DEFAULT', 'SGR', 'SGR_PIXELS']) {
      for (const button of ['number', 'string', 'null', 'undefined', 'false', 'true', 'nan',
        'symbol', 'bigint', 'object', 'changing', 'throwCoerce']) {
        for (const action of [0, 1, 32]) cases.push({ protocol, encoding, button, action });
      }
      for (const actionRaw of ['number', 'string', 'null', 'undefined', 'false', 'true', 'nan',
        'symbol', 'bigint', 'object', 'changing', 'throwCoerce']) {
        for (const action of [-2147483648, 2147483647]) {
          cases.push({ protocol, encoding, button: 'number', wheel: true, action, actionRaw });
        }
      }
      for (const field of ['ctrl', 'shift', 'alt', 'button', 'action', 'col', 'row', 'x', 'y']) {
        cases.push({ protocol, encoding, button: 'number', action: 1, throwGet: field });
      }
      for (const coordinate of ['string', 'object', 'symbol', 'bigint', 'changing', 'throwCoerce']) {
        cases.push({ protocol, encoding, button: 'number', action: 0, coordinate });
      }
    }
  }
  return cases;
}

export function observe(restrict, encode, spec) {
  const log = [], sentinel = {};
  let coercions = 0;
  function raw(kind, number) {
    if (kind === 'number') return number;
    if (kind === 'string') return String(number);
    if (kind === 'null') return null;
    if (kind === 'undefined') return undefined;
    if (kind === 'false') return false;
    if (kind === 'true') return true;
    if (kind === 'nan') return NaN;
    if (kind === 'symbol') return Symbol('raw');
    if (kind === 'bigint') return BigInt(number);
    return { [Symbol.toPrimitive](hint) {
      log.push('coerce:' + hint);
      if (kind === 'throwCoerce') throw sentinel;
      return kind === 'changing' ? ++coercions : number;
    } };
  }
  const values = { col: raw(spec.coordinate ?? 'number', 1), row: raw(spec.coordinate ?? 'number', 2),
    x: raw(spec.coordinate ?? 'number', 10), y: raw(spec.coordinate ?? 'number', 20),
    button: raw(spec.button, spec.wheel ? 4 : 0), action: raw(spec.actionRaw ?? 'number', spec.action), ctrl: false, alt: true, shift: false };
  const event = {};
  for (const key of Object.keys(values)) Object.defineProperty(event, key, {
    get() { log.push('get:' + key); if (key === spec.throwGet) throw sentinel; return values[key]; },
    set(value) { log.push('set:' + key + ':' + value); values[key] = value; }, configurable: true
  });
  function call(fn) {
    try {
      const result = fn(event);
      return typeof result === 'string' ? { units: Array.from({ length: result.length }, (_, i) => result.charCodeAt(i)) } : { result };
    } catch (error) {
      if (error === sentinel) return { thrownSentinel: true };
      if (error?.name === 'TypeError') return { thrownType: 'TypeError' };
      throw error;
    }
  }
  const restricted = call(restrict);
  const encoded = call(encode);
  return { restricted, encoded, log };
}

export function compareAll(protocols, encodings, fixtureText) {
  const fixtures = JSON.parse(fixtureText);
  if (fixtures.length !== 1125) throw new Error('Unexpected mouse field corpus count');
  for (const { spec, expected } of fixtures) {
    const actual = observe(protocols[spec.protocol], encodings[spec.encoding], spec);
    if (JSON.stringify(actual) !== JSON.stringify(expected)) {
      throw new Error(JSON.stringify({ spec, actual, expected }));
    }
  }
  return fixtures.length;
}
