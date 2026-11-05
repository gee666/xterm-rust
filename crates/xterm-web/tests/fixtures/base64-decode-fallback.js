// Shared unchanged-source and Rust-export traces. Restore every host override.
function base64DecodeFallbackCases(createCodec) {
  const rows = [];
  const NativeBytes = Uint8Array;
  const originals = Object.fromEntries(['Uint8Array', 'atob', 'TextDecoder'].map(key =>
    [key, Object.getOwnPropertyDescriptor(globalThis, key)]));
  const originalModern = Object.getOwnPropertyDescriptor(NativeBytes, 'fromBase64');
  const define = (key, value) => Object.defineProperty(globalThis, key,
    { configurable: true, writable: true, value });
  function run(name, setup, input = 'QQ==') {
    const log = [], marker = {}, codec = createCodec();
    try {
      Object.defineProperty(NativeBytes, 'fromBase64', { configurable: true, value: undefined });
      const hooks = {
        log, marker, NativeBytes,
        atob: value => define('atob', value),
        bytes: value => define('Uint8Array', value),
        decoder: value => define('TextDecoder', value)
      };
      setup(hooks);
      let result;
      try {
        const value = codec.decodeText(input);
        result = value === marker ? { marker: true } :
          { type: typeof value, value: value === undefined ? null : value };
      } catch (error) {
        result = { thrownMarker: error === marker, type: error === marker ? 'marker' : error.name };
      }
      rows.push({ name, log, result });
    } finally {
      for (const [key, descriptor] of Object.entries(originals)) Object.defineProperty(globalThis, key, descriptor);
      if (originalModern) Object.defineProperty(NativeBytes, 'fromBase64', originalModern);
      else delete NativeBytes.fromBase64;
      try { codec.free?.(); } catch (error) {
        // Baseline uncaught JS imports can leave a borrowed WASM receiver. Keep
        // this as an asserted mismatch instead of hiding it behind cleanup.
        rows[rows.length - 1].cleanupError = error.name + ': ' + error.message;
      }
    }
  }
  for (const [name, value] of [['number', 42], ['boolean', true], ['symbol', Symbol('bin')],
    ['null', null], ['undefined', undefined], ['boxed string', new String('A')]]) {
    run('atob raw '+name, ({ atob }) => atob(() => value));
  }
  run('raw object property call and byte coercion', ({ log, atob }) => {
    const bin = { get length() { log.push('length'); return 2; },
      get charCodeAt() { log.push('charCodeAt get'); return function (...args) {
        log.push({ receiver: this === bin, args });
        return { valueOf() { log.push('byte valueOf'); return args[0] ? 0x142 : 0x141; } };
      }; } };
    atob(function (...args) { log.push({ receiver: this === undefined, args }); return bin; });
  });
  run('length reentry changes loop bound', ({ log, atob }) => {
    let reads = 0;
    atob(() => ({ get length() { log.push(++reads); return reads < 3 ? 3 : 1; },
      charCodeAt(i) { log.push(['charCodeAt', i]); return 65+i; } }));
  });
  run('length value conversion order', ({ log, atob }) => {
    atob(() => ({ get length() { log.push('length'); return {
      valueOf() { log.push('length valueOf'); return 1; }
    }; }, charCodeAt(i) { log.push(i); return 65; } }));
  });
  run('length getter exception caught', ({ log, marker, atob }) => {
    atob(() => ({ get length() { log.push('length'); throw marker; } }));
  });
  run('charCodeAt getter exception caught', ({ log, marker, atob }) => {
    atob(() => ({ length: 1, get charCodeAt() { log.push('charCodeAt'); throw marker; } }));
  });
  run('charCodeAt noncallable caught', ({ atob }) => atob(() => ({ length: 1, charCodeAt: null })));
  run('charCodeAt raw Symbol conversion caught', ({ atob }) => atob(() => ({ length: 1, charCodeAt() { return Symbol('byte'); } })));
  run('charCodeAt raw BigInt conversion caught', ({ atob }) => atob(() => ({ length: 1, charCodeAt() { return 65n; } })));
  run('atob exception caught', ({ marker, atob }) => atob(() => { throw marker; }));
  run('replaced bytes constructor length and decoder order', ({ log, atob, bytes, decoder }) => {
    const bin = { get length() { log.push('length'); return 1; }, charCodeAt(i) { log.push(['charCodeAt', i]); return 321; } };
    atob(value => { log.push(['atob', value]); return bin; });
    let buffer;
    bytes(function (...args) {
      log.push({ construct: !!new.target, args });
      buffer = { set 0(value) { log.push(['set 0', value]); } };
      return buffer;
    });
    decoder(function (...args) { log.push({ decoder: !!new.target, args }); return {
      get decode() { log.push('decode get'); return function (...args) {
        log.push({ bytesIdentity: args[0] === buffer, args: args.length }); return 'host';
      }; }
    }; });
  });
  run('constructor receives raw length before loop coercion', ({ log, atob, bytes, decoder }) => {
    const length = { valueOf() { log.push('valueOf'); return 0; } };
    atob(() => ({ get length() { log.push('length'); return length; } }));
    bytes(function (value) { log.push({ rawLength: value === length }); return {}; });
    decoder(function () { return { decode() { return 'host'; } }; });
  });
  run('constructor changed inside length getter uses selected reference', ({ log, atob, bytes, NativeBytes }) => {
    bytes(function (size) { log.push(['selected', size]); return new NativeBytes(size); });
    atob(() => ({ get length() {
      log.push('length'); bytes(function () { log.push('unexpected constructor'); throw Error('wrong constructor'); }); return 1;
    }, charCodeAt() { return 65; } }));
  });
  run('constructor selected after atob mutation', ({ log, atob, bytes, NativeBytes }) => {
    atob(() => { bytes(function (size) { log.push(['new constructor', size]); return new NativeBytes(size); }); return 'A'; });
  });
  run('constructor exception caught', ({ log, marker, bytes }) => {
    bytes(function () { log.push('construct'); throw marker; });
  });
  run('index setter observes raw charCodeAt return', ({ log, marker, atob, bytes, decoder }) => {
    atob(() => ({ length: 1, charCodeAt() { return marker; } }));
    bytes(function () { return { set 0(value) { log.push({ raw: value === marker }); } }; });
    decoder(function () { return { decode() { return 'host'; } }; });
  });
  run('index setter exception prevents decoder', ({ log, marker, bytes, decoder }) => {
    bytes(function () { return { set 0(value) { log.push(['set', value]); throw marker; } }; });
    decoder(function () { log.push('unexpected decoder'); return { decode() { return ''; } }; });
  });
  run('frozen constructor return assignment caught in strict mode', ({ bytes }) => {
    bytes(function () { return Object.freeze({}); });
  });
  run('global constructor getter read timing', ({ log, NativeBytes }) => {
    Object.defineProperty(globalThis, 'Uint8Array', { configurable: true, get() { log.push('Uint8Array get'); return NativeBytes; } });
  });
  run('constructor capability getter exception remains outside catch', ({ marker, bytes }) => {
    bytes(Object.defineProperty(function () {}, 'fromBase64', { get() { throw marker; } }));
  });
  return rows;
}
