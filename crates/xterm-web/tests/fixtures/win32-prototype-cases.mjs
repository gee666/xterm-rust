// Licensed under MIT. Original-protocol reference cases; no Rust result is fabricated.
export const prototypeCases = (() => {
  const cases = [];
  const names = ['constructor', 'toString', 'valueOf', 'hasOwnProperty', 'isPrototypeOf',
    'propertyIsEnumerable', 'toLocaleString', '__defineGetter__', '__defineSetter__',
    '__lookupGetter__', '__lookupSetter__', '__proto__'];
  for (const name of names) for (const down of [false, true]) for (const mask of [0, 15]) {
    for (const location of ['code', 'key', 'both']) cases.push({ id: `builtin-${name}-${location}-${Number(down)}-${mask}`,
      kind: 'builtin', name, location, down, mask });
  }
  for (const value of ['undefined', 'null', 'false', 'true', 'zero', 'negative-zero', 'nan',
    'infinity', 'negative-infinity', 'empty', 'text', 'surrogate']) {
    for (const down of [false, true]) for (const mask of [0, 15]) {
      cases.push({ id: `primitive-${value}-${Number(down)}-${mask}`, kind: 'primitive', value, down, mask });
    }
  }
  for (const mask of [0, 15]) for (const kind of ['late-object', 'late-prototype-text', 'string-hint',
    'receiver-mutation', 'second-encoder-isolation', 'symbol-result', 'invalid-result']) {
    cases.push({ id: `${kind}-${mask}`, kind, mask, down: true });
  }
  for (const table of ['vk', 'sc', 'uc']) for (const kind of ['lookup-throw', 'coercion-throw']) {
    cases.push({ id: `${kind}-${table}`, kind, table, mask: 0, down: true });
  }
  return cases;
})();

// Self-contained so Node vm and actual browser realms run identical cases.
export function capturePrototypeCase(Encoder, spec) {
  const trace = [];
  const results = [];
  const identities = new Map();
  let nextIdentity = 1;
  const id = value => {
    if (!identities.has(value)) identities.set(value, `value-${nextIdentity++}`);
    return identities.get(value);
  };
  const units = value => Array.from({ length: value.length }, (_, index) => value.charCodeAt(index));
  const describe = value => {
    if (value === undefined) return { kind: 'undefined' };
    if (value === null) return { kind: 'null' };
    if (typeof value === 'number') return { kind: 'number', value: Number.isNaN(value) ? 'NaN'
      : value === Infinity ? 'Infinity' : value === -Infinity ? '-Infinity' : Object.is(value, -0) ? '-0' : value };
    if (typeof value === 'boolean') return { kind: 'boolean', value };
    if (typeof value === 'string') return { kind: 'string', units: units(value) };
    return { kind: typeof value, identity: id(value) };
  };
  const marker = {};
  const encoder = new Encoder();
  const maps = new Map([[encoder._codeToVk, 'vk'], [encoder._codeToScancode, 'sc'], [encoder._keyToControlChar, 'uc']]);
  for (const [map, name] of maps) identities.set(map, `table-${name}`);
  const prototype = Object.getPrototypeOf(encoder._codeToVk);
  const probe = '__xterm_prototype_probe__';
  const savedProbe = Object.getOwnPropertyDescriptor(prototype, probe);
  const savedText = Object.getOwnPropertyDescriptor(prototype, 'toString');
  let phase = 'early';
  const primitive = name => ({ undefined: undefined, null: null, false: false, true: true,
    zero: 0, 'negative-zero': -0, nan: NaN, infinity: Infinity, 'negative-infinity': -Infinity,
    empty: '', text: 'field text', surrogate: '\ud800\udfff' })[name];
  const objects = new Map();
  const rawObject = table => {
    if (objects.has(table)) return objects.get(table);
    const value = spec.kind === 'string-hint' ? {
      toString() { trace.push({ op: 'toString', table }); return `string-${table}`; },
      valueOf() { trace.push({ op: 'valueOf', table }); return `default-${table}`; }
    } : { [Symbol.toPrimitive](hint) {
      trace.push({ op: 'coerce', table, hint, phase, receiver: this === value });
      if (spec.kind === 'coercion-throw' && spec.table === table) throw marker;
      if (spec.kind === 'symbol-result') return Symbol('source symbol');
      if (spec.kind === 'invalid-result') return {};
      return `${phase}-${table}`;
    } };
    objects.set(table, value);
    return value;
  };
  if (spec.kind !== 'builtin' && spec.kind !== 'late-prototype-text') {
    Object.defineProperty(prototype, probe, { configurable: true, get() {
      const table = maps.get(this) ?? 'foreign-table';
      trace.push({ op: 'lookup', table, receiver: id(this) });
      if (spec.kind === 'lookup-throw' && spec.table === table) throw marker;
      if (spec.kind === 'primitive') return primitive(spec.value);
      if (spec.kind === 'receiver-mutation' || spec.kind === 'second-encoder-isolation') {
        if (table === 'vk') { this.KeyA = 991; encoder._codeToScancode.KeyA = 123; }
        return table === 'vk' ? 1 : table === 'sc' ? 2 : 3;
      }
      return rawObject(table);
    } });
  }
  function execute(selected, number) {
    let codeReads = 0;
    const event = {};
    const mask = spec.mask ?? 0;
    const mutation = spec.kind === 'receiver-mutation' || spec.kind === 'second-encoder-isolation';
    const code = spec.kind === 'builtin' ? (spec.location === 'key' ? 'KeyA' : spec.name)
      : spec.kind === 'late-prototype-text' ? '__proto__' : probe;
    const key = spec.kind === 'builtin' ? (spec.location === 'code' ? 'a' : spec.name)
      : spec.kind === 'late-prototype-text' ? '__proto__' : mutation ? 'a' : probe;
    const field = (name, read) => Object.defineProperty(event, name, { get() {
      const value = read(); trace.push({ op: 'field', event: number, name, value: describe(value) }); return value;
    } });
    field('keyCode', () => 65);
    field('key', () => key);
    field('code', () => {
      codeReads++;
      const last = 3 + Number(!!(mask & 4)) + Number(!!(mask & 2));
      if (codeReads === last) {
        phase = 'late';
        if (spec.kind === 'late-prototype-text') Object.defineProperty(prototype, 'toString', {
          configurable: true, value() { trace.push({ op: 'late-prototype-toString' }); return '[late prototype]'; }
        });
      }
      if (mutation && (number > 0 || codeReads > 1)) return 'KeyA';
      return code;
    });
    for (const [name, bit] of [['shiftKey', 1], ['altKey', 2], ['ctrlKey', 4], ['metaKey', 8]]) field(name, () => !!(mask & bit));
    try {
      const output = selected.evaluateKeyboardEvent(event, spec.down);
      results.push({ returned: { type: output.type, cancel: output.cancel,
        key: output.key === undefined ? { kind: 'undefined' } : { kind: 'string', units: units(output.key) } } });
    } catch (error) {
      results.push({ threw: Object.is(error, marker) ? { marker: true }
        : { marker: false, name: String(error.name), message: units(String(error.message)) } });
    }
  }
  try {
    execute(encoder, 0);
    if (spec.kind === 'receiver-mutation') execute(encoder, 1);
    if (spec.kind === 'second-encoder-isolation') {
      const second = new Encoder();
      for (const [map, name] of [[second._codeToVk, 'vk-2'], [second._codeToScancode, 'sc-2'], [second._keyToControlChar, 'uc-2']]) {
        maps.set(map, name); identities.set(map, `table-${name}`);
      }
      execute(second, 1);
    }
    return { caseId: spec.id, results, trace, ownValues: {
      vkKeyA: describe(encoder._codeToVk.KeyA), scKeyA: describe(encoder._codeToScancode.KeyA) } };
  } finally {
    if (savedProbe) Object.defineProperty(prototype, probe, savedProbe); else delete prototype[probe];
    Object.defineProperty(prototype, 'toString', savedText);
  }
}
