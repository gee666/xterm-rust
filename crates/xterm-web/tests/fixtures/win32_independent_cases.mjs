// Licensed under MIT. Independent task185 probes use own-property descriptors
// rather than the retained suite's Object.prototype injection.
const values = ['undefined', 'null', 'false', 'true', 'zero', 'negative-zero', 'nan',
  'infinity', 'negative-infinity', 'empty', 'text', 'high-surrogate', 'low-surrogate',
  'nul', 'bigint-zero', 'bigint-one', 'bigint-large', 'bigint-negative', 'symbol',
  'boxed-false', 'boxed-zero', 'array'];
const operations = ['proxy-hint', 'proxy-throw', 'lookup-throw', 'toString-throw',
  'valueOf-fallback', 'reentrant', 'late-table-mutation'];

function primitive(name) {
  switch (name) {
    case 'undefined': return undefined;
    case 'null': return null;
    case 'false': return false;
    case 'true': return true;
    case 'zero': return 0;
    case 'negative-zero': return -0;
    case 'nan': return NaN;
    case 'infinity': return Infinity;
    case 'negative-infinity': return -Infinity;
    case 'empty': return '';
    case 'text': return 'independent;field';
    case 'high-surrogate': return '\ud800';
    case 'low-surrogate': return '\udfff';
    case 'nul': return '\0';
    case 'bigint-zero': return 0n;
    case 'bigint-one': return 1n;
    case 'bigint-large': return 123456789012345678901234567890n;
    case 'bigint-negative': return -987654321987654321n;
    case 'symbol': return Symbol('independent');
    case 'boxed-false': return Object(false);
    case 'boxed-zero': return Object(0);
    case 'array': return ['a', '\ud800', 'b'];
    default: throw new Error('Unknown independent primitive');
  }
}
const units = value => Array.from({ length: value.length }, (_, index) => value.charCodeAt(index));
function record(value) {
  return { type: value.type, cancel: value.cancel, key: units(value.key) };
}

function capture(Encoder, spec) {
  const encoder = new Encoder();
  const maps = [encoder._codeToVk, encoder._codeToScancode, encoder._keyToControlChar];
  const trace = [];
  const marker = {};
  const probe = '__independent_win32__';
  let phase = 'before-modifiers';
  const value = () => {
    if (values.includes(spec.operation)) return primitive(spec.operation);
    const object = {
      toString() {
        trace.push(['toString', phase, this === object]);
        if (spec.operation === 'toString-throw') throw marker;
        if (spec.operation === 'valueOf-fallback') return {};
        if (spec.operation === 'reentrant') {
          // Reentry uses the same table owners while the outer raw value lives.
          const event = { code: 'KeyA', key: 'a', keyCode: 65,
            shiftKey: false, ctrlKey: false, altKey: false, metaKey: false };
          trace.push(['nested', record(encoder.evaluateKeyboardEvent(event, false))]);
        }
        return `own-${phase}`;
      },
      valueOf() { trace.push(['valueOf', this === object]); return 905; }
    };
    if (spec.operation.startsWith('proxy-')) return new Proxy(object, {
      get(target, name, receiver) {
        trace.push(['proxy-get', typeof name === 'symbol' ? 'Symbol.toPrimitive' : name,
          receiver === proxy]);
        if (spec.operation === 'proxy-throw' && name === Symbol.toPrimitive) throw marker;
        return Reflect.get(target, name, receiver);
      }
    });
    return object;
  };
  // Proxy receivers are checked without changing the receiver's identity.
  let proxy;
  const raw = value();
  proxy = raw;
  for (const [index, table] of maps.entries()) Object.defineProperty(table, probe, {
    configurable: true, get() {
      trace.push(['lookup', index, this === table]);
      if (index === spec.table && spec.operation === 'lookup-throw') throw marker;
      return index === spec.table ? raw : 11 + index;
    }
  });
  const event = {};
  for (const [name, result] of Object.entries({ code: probe, key: probe, keyCode: 211,
    shiftKey: !!(spec.mask & 1), altKey: !!(spec.mask & 2),
    ctrlKey: !!(spec.mask & 4), metaKey: !!(spec.mask & 8) })) {
    Object.defineProperty(event, name, { get() {
      trace.push(['field', name]);
      if (name === 'shiftKey') {
        phase = 'after-modifiers';
        if (spec.operation === 'late-table-mutation') {
          for (const table of maps) Object.defineProperty(table, probe, { value: 999, configurable: true });
        }
      }
      return result;
    } });
  }
  try {
    return { returned: record(encoder.evaluateKeyboardEvent(event, spec.down)), trace };
  } catch (error) {
    return { threw: error === marker ? { marker: true }
      : { name: error.name, message: units(error.message) }, trace };
  }
}

export function compareIndependentWin32(Source, Port) {
  const failures = [];
  let cases = 0;
  for (const operation of [...values, ...operations]) for (const table of [0, 1, 2]) {
    for (const down of [false, true]) for (const mask of [0, 15]) {
      const spec = { operation, table, down, mask };
      const id = `${operation}-${table}-${Number(down)}-${mask}`;
      cases++;
      const expected = capture(Source, spec);
      const actual = capture(Port, spec);
      if (JSON.stringify(actual) !== JSON.stringify(expected)) failures.push({ id, expected, actual });
    }
  }
  return { cases, failures };
}
