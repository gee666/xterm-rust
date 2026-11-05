// Same host-boundary traces run against unchanged source and Rust exports.
function base64EncodeCases(createCodec) {
  const rows = [];
  const prototype = Uint8Array.prototype;
  const descriptor = Object.getOwnPropertyDescriptor(prototype, 'toBase64');
  const originalBtoa = Object.getOwnPropertyDescriptor(globalThis, 'btoa');
  function run(name, setup, input = 'A€\ud800') {
    const log = [], marker = {}, codec = createCodec();
    const property = definition => Object.defineProperty(prototype, 'toBase64', { configurable: true, ...definition });
    const fallback = value => Object.defineProperty(globalThis, 'btoa', { configurable: true, writable: true, value });
    try {
      setup({ log, marker, property, fallback });
      let result;
      try {
        const value = codec.encodeText(typeof input === 'function' ? input(log) : input);
        result = value === marker ? { marker: true } : { type: typeof value, value: value === undefined ? null : value };
      } catch (error) {
        result = { thrownMarker: error === marker, type: error === marker ? 'marker' : error.name };
      }
      rows.push({ name, log, result });
    } finally {
      if (descriptor) Object.defineProperty(prototype, 'toBase64', descriptor);
      else delete prototype.toBase64;
      if (originalBtoa) Object.defineProperty(globalThis, 'btoa', originalBtoa);
      else delete globalThis.btoa;
      codec.free?.();
    }
  }
  run('native byte encoding', ({ property }) => property({ value: undefined }));
  run('modern method receiver and two reads', ({ log, property }) => property({ get() {
    log.push('get');
    return function (...args) { log.push({ receiver: this instanceof Uint8Array, bytes: Array.from(this), args: args.length }); return 'modern'; };
  } }));
  run('modern raw object return', ({ marker, property }) => property({ value() { return marker; } }));
  run('modern undefined return', ({ property }) => property({ value() { return undefined; } }));
  run('modern null return', ({ property }) => property({ value() { return null; } }));
  run('modern numeric return', ({ property }) => property({ value() { return 42; } }));
  run('getter throws original value', ({ marker, property }) => property({ get() { throw marker; } }));
  run('method throws original value', ({ marker, property }) => property({ value() { throw marker; } }));
  run('null is present but noncallable', ({ property }) => property({ value: null }));
  run('second getter value is invoked', ({ log, property }) => {
    let reads = 0;
    property({ get() { log.push(++reads); return reads === 1 ? function () {} : null; } });
  });
  run('fallback raw return and binary argument', ({ log, marker, property, fallback }) => {
    property({ get() { log.push('get'); return undefined; } });
    fallback(function (...args) { log.push({ receiver: this === globalThis, args: args.length, units: Array.from(args[0], c => c.charCodeAt(0)) }); return marker; });
  });
  run('fallback sees byte mutation by capability getter', ({ log, property, fallback }) => {
    property({ get() { this[0] = 66; return undefined; } });
    fallback(value => { log.push(Array.from(value, c => c.charCodeAt(0))); return 'mutated'; });
  });
  run('fallback throws original value', ({ marker, property, fallback }) => {
    property({ value: undefined }); fallback(() => { throw marker; });
  });
  run('coercion precedes capability reads', ({ log, property }) => property({ get() {
    log.push('get'); return function () { log.push('call'); return 'done'; };
  } }), log => ({ toString() { log.push('string'); return 'A'; } }));
  run('undefined input encodes empty bytes', ({ log, property }) => property({ value() { log.push(Array.from(this)); return 'empty'; } }), () => undefined);
  run('Symbol conversion precedes capability reads', ({ log, property }) => property({ get() { log.push('unexpected'); return undefined; } }), () => Symbol('input'));
  return rows;
}
