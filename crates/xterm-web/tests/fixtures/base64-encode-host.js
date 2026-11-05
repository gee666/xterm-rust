// Shared unchanged-source and Rust-export traces for encoder host operations.
function base64EncodeHostCases(createCodec) {
  const rows = [];
  const encoderDescriptor = Object.getOwnPropertyDescriptor(globalThis, 'TextEncoder');
  const btoaDescriptor = Object.getOwnPropertyDescriptor(globalThis, 'btoa');
  const charDescriptor = Object.getOwnPropertyDescriptor(String, 'fromCharCode');
  const capability = Object.getOwnPropertyDescriptor(Uint8Array.prototype, 'toBase64');
  const install = (object, key, value) => Object.defineProperty(object, key, { configurable: true, writable: true, value });
  const restore = (object, key, descriptor) => descriptor ? Object.defineProperty(object, key, descriptor) : delete object[key];
  function run(name, setup, input) {
    if (arguments.length < 3) input = 'input';
    const codec = createCodec(), log = [], marker = {};
    try {
      setup({ log, marker, encoder: value => install(globalThis, 'TextEncoder', value),
        fallback: value => install(globalThis, 'btoa', value), chars: value => install(String, 'fromCharCode', value) });
      let result;
      try {
        const value = codec.encodeText(input);
        result = value === marker ? { marker: true } : { type: typeof value, value: value === undefined ? null : value };
      } catch (error) {
        result = { marker: error === marker, type: error === marker ? 'marker' : error?.name };
      }
      rows.push({ name, log, result });
    } finally {
      restore(globalThis, 'TextEncoder', encoderDescriptor);
      restore(globalThis, 'btoa', btoaDescriptor);
      restore(String, 'fromCharCode', charDescriptor);
      restore(Uint8Array.prototype, 'toBase64', capability);
      codec.free?.();
    }
  }
  run('constructor and encode receiver precede capability', ({ log, encoder, marker }) => {
    const bytes = { get toBase64() { log.push('capability'); return function (...args) { log.push({ receiver: this === bytes, args: args.length }); return marker; }; } };
    encoder(function (...args) {
      log.push({ constructor: new.target !== undefined, args: args.length });
      const instance = this;
      Object.defineProperty(this, 'encode', { get() { log.push('encode getter'); return function (data) { log.push({ receiver: this === instance, data }); return bytes; }; } });
    });
  });
  run('constructor throws original value', ({ encoder, marker }) => encoder(function () { throw marker; }));
  run('nonconstructible encoder', ({ encoder }) => encoder(() => ({})));
  run('null encoder constructor', ({ encoder }) => encoder(null));
  run('encode getter throws original value', ({ encoder, marker }) => encoder(function () { Object.defineProperty(this, 'encode', { get() { throw marker; } }); }));
  run('noncallable encode method', ({ encoder }) => encoder(function () { this.encode = null; }));
  run('encode method throws original value', ({ encoder, marker }) => encoder(function () { this.encode = () => { throw marker; }; }));
  run('encode sees unconverted Symbol input', ({ log, encoder }) => encoder(function () { this.encode = value => { log.push(typeof value); return { toBase64() { return 'symbol accepted by custom encoder'; } }; }; }), Symbol('input'));
  run('encode sees undefined input', ({ log, encoder }) => encoder(function () { this.encode = (...args) => { log.push({ args: args.length, type: typeof args[0] }); return { toBase64() { return 'undefined'; } }; }; }), undefined);
  run('null byte result throws', ({ encoder }) => encoder(function () { this.encode = () => null; }));
  run('raw numeric byte result boxes for fallback', ({ encoder, fallback }) => { encoder(function () { this.encode = () => 7; }); fallback(value => value); });
  run('changing capability getter on raw result', ({ log, encoder }) => {
    let reads = 0;
    encoder(function () { this.encode = () => ({ get toBase64() { log.push(++reads); return reads === 1 ? function () {} : null; } }); });
  });
  run('fallback mutable length and index order', ({ log, encoder, fallback }) => {
    let reads = 0;
    const bytes = { get toBase64() { log.push('capability'); return undefined; }, get length() { log.push('length'); return ++reads < 3 ? 2 : 0; }, get 0() { log.push('index0'); return 65; }, get 1() { log.push('index1'); return 8364; } };
    encoder(function () { this.encode = () => bytes; });
    fallback(value => { log.push(Array.from(value, c => c.charCodeAt(0))); return 'fallback'; });
  });
  run('fallback calls mutable String.fromCharCode per byte', ({ log, encoder, chars, fallback }) => {
    encoder(function () { this.encode = () => ({ length: 2, 0: 65, 1: 66 }); });
    chars(function (...args) { log.push({ receiver: this === String, args }); return { [Symbol.toPrimitive](hint) { log.push(hint); return 'X'; } }; });
    fallback(value => value);
  });
  run('fallback char method changes between bytes', ({ log, encoder, chars, fallback }) => {
    encoder(function () { this.encode = () => ({ length: 2, 0: 65, 1: 66 }); });
    chars(function (value) { log.push(value); chars(function (next) { log.push(next); return 'Y'; }); return 'X'; });
    fallback(value => value);
  });
  run('fallback char conversion throws original value', ({ encoder, chars, marker }) => {
    encoder(function () { this.encode = () => ({ length: 1, 0: 65 }); }); chars(() => { throw marker; });
  });
  run('fallback byte coercion uses standard host conversion', ({ log, encoder, fallback }) => {
    encoder(function () { this.encode = () => ({ length: 1, 0: { valueOf() { log.push('number'); return 65601; } } }); }); fallback(value => value);
  });
  run('fallback length coercion throws original value', ({ encoder, marker }) => encoder(function () { this.encode = () => ({ length: { valueOf() { throw marker; } } }); }));
  run('fallback bigint length accepts numeric index comparison', ({ encoder, fallback }) => { encoder(function () { this.encode = () => ({ length: 1n, 0: 65 }); }); fallback(value => value); });
  run('fallback zero length never reads char method', ({ encoder, chars, fallback }) => { encoder(function () { this.encode = () => ({ length: 0 }); }); chars(null); fallback(value => value); });
  return rows;
}
