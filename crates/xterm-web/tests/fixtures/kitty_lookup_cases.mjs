// Licensed under MIT. Realm-local source/port probes, never production logic.
const rawNames = ['undefined', 'null', 'false', 'true', 'zero', 'negative-zero', 'nan',
  'infinity', 'empty', 'text', 'high-surrogate', 'low-surrogate', 'bigint-zero',
  'bigint-large', 'symbol', 'boxed-false', 'boxed-zero', 'array', 'enter', 'tab',
  'backspace', 'boxed-enter', 'native-function', 'native-object'];
const operations = ['primitive-hint', 'valueOf-first', 'toString-fallback', 'conversion-throw',
  'lookup-throw', 'proxy', 'proxy-throw', 'reentry', 'persistent-mutation', 'cached-functional'];
const fields = ['_functionalKeyCodes', '_csiTildeKeys', '_csiLetterKeys', '_ss3FunctionKeys'];
const units = text => text === undefined ? null :
  Array.from({ length: text.length }, (_, i) => text.charCodeAt(i));
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
    case 'empty': return '';
    case 'text': return 'raw;value';
    case 'high-surrogate': return '\ud800';
    case 'low-surrogate': return '\udfff';
    case 'bigint-zero': return 0n;
    case 'bigint-large': return 123456789012345678901234567890n;
    case 'symbol': return Symbol('kitty');
    case 'boxed-false': return Object(false);
    case 'boxed-zero': return Object(0);
    case 'array': return ['a', '\ud800', 'b'];
    case 'enter': return 13;
    case 'tab': return 9;
    case 'backspace': return 127;
    case 'boxed-enter': return Object(13);
    case 'native-function': return Object.prototype.toString;
    case 'native-object': return Object.prototype;
    default: throw new Error('Unknown raw value');
  }
}
function record(value) { return { type: value.type, cancel: value.cancel, key: units(value.key) }; }

export function captureKittyLookup(Encoder, spec) {
  const encoder = new Encoder();
  const other = new Encoder();
  const maps = fields.map(field => encoder[field]);
  const trace = [];
  const marker = {};
  const probe = spec.intrinsicKey ?? '__kitty_raw_lookup__';
  let phase = 'before-lookups';
  let functionalReads = 0;
  const base = { key: probe, code: '', keyCode: 0, shiftKey: !!(spec.mask & 1),
    altKey: !!(spec.mask & 2), ctrlKey: !!(spec.mask & 4), metaKey: !!(spec.mask & 8) };
  const rawObject = {
    valueOf() {
      trace.push(['valueOf', this === raw, phase]);
      if (spec.operation === 'conversion-throw') throw marker;
      if (spec.operation === 'valueOf-first') return 77;
      if (spec.operation === 'persistent-mutation') {
        Object.defineProperty(maps[spec.table], probe, { value: 'persisted', configurable: true });
      }
      if (spec.operation === 'reentry') {
        trace.push(['nested', record(encoder.evaluate({ ...base, key: 'a' }, 8, 1, false))]);
      }
      return {};
    },
    toString() { trace.push(['toString', this === raw, phase]); return 'raw-\ud800'; }
  };
  if (spec.operation === 'primitive-hint') Object.defineProperty(rawObject, Symbol.toPrimitive, {
    value(hint) { trace.push(['toPrimitive', hint, this === rawObject, phase]); return 'hint-\udfff'; }
  });
  let raw = rawNames.includes(spec.operation) ? primitive(spec.operation) : rawObject;
  if (spec.operation.startsWith('proxy')) raw = new Proxy(rawObject, {
    get(target, name, receiver) {
      trace.push(['proxy-get', typeof name === 'symbol' ? 'Symbol.toPrimitive' : name, receiver === raw]);
      if (spec.operation === 'proxy-throw' && name === Symbol.toPrimitive) throw marker;
      return Reflect.get(target, name, receiver);
    }
  });
  function getValue(index, receiver) {
    trace.push(['lookup', index, receiver === maps[index]]);
    if (index === spec.table && spec.operation === 'lookup-throw') throw marker;
    phase = 'lookup-' + index;
    if (index === 0 && ++functionalReads === 2) {
      phase = 'second-functional';
      if (spec.operation === 'cached-functional') return undefined;
    }
    return index === spec.table ? raw : undefined;
  }
  const original = Object.getOwnPropertyDescriptor(Object.prototype, probe);
  if (spec.intrinsicKey === undefined && spec.inherited) Object.defineProperty(Object.prototype, probe, {
    configurable: true, get() {
      const index = maps.indexOf(this);
      // The second encoder must not share the first encoder's table identity.
      if (index < 0) return undefined;
      return getValue(index, this);
    }
  });
  else if (spec.intrinsicKey === undefined) maps.forEach((map, index) => Object.defineProperty(map, probe, {
    configurable: true, get() { return getValue(index, this); }
  }));
  const event = {};
  for (const [name, value] of Object.entries(base)) Object.defineProperty(event, name, {
    get() { trace.push(['field', name]); return value; }
  });
  try {
    const result = record(encoder.evaluate(event, spec.flags, spec.type, false));
    if (spec.operation === 'persistent-mutation') {
      trace.push(['again', record(encoder.evaluate(event, spec.flags, spec.type, false))]);
    }
    if (spec.operation === 'persistent-mutation' || spec.operation === 'reentry') {
      trace.push(['isolated', record(other.evaluate(base, 8, 1, false))]);
    }
    return { result, trace };
  } catch (error) {
    return { threw: error === marker ? { marker: true }
      : { name: error.name, message: units(error.message) }, trace };
  } finally {
    if (spec.intrinsicKey === undefined && spec.inherited) {
      if (original) Object.defineProperty(Object.prototype, probe, original);
      else delete Object.prototype[probe];
    }
  }
}

export const kittyLookupSpecs = [];
for (const operation of [...rawNames, ...operations]) for (const table of [0, 1, 2, 3]) {
  for (const mask of [0, 1, 15]) for (const flags of [0, 2, 8, 31]) {
    for (const type of [1, 2, 3]) for (const inherited of [false, true]) {
      kittyLookupSpecs.push({ operation, table, mask, flags, type, inherited });
    }
  }
}
for (const intrinsicKey of ['constructor', '__defineGetter__', '__defineSetter__', 'hasOwnProperty',
  '__lookupGetter__', '__lookupSetter__', 'isPrototypeOf', 'propertyIsEnumerable',
  'toString', 'valueOf', '__proto__', 'toLocaleString']) {
  for (const mask of [0, 1, 15]) for (const flags of [0, 2, 8, 31]) for (const type of [1, 2, 3]) {
    kittyLookupSpecs.push({ operation: 'intrinsic', intrinsicKey, mask, flags, type });
  }
}
export function compareKittyLookups(Source, Port) {
  const failures = [];
  for (const spec of kittyLookupSpecs) {
    const expected = captureKittyLookup(Source, spec);
    const actual = captureKittyLookup(Port, spec);
    if (JSON.stringify(actual) !== JSON.stringify(expected)) failures.push({ spec, expected, actual });
  }
  return { cases: kittyLookupSpecs.length, failures };
}
