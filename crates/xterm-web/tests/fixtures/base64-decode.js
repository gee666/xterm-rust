// Shared unchanged-source and Rust-export traces. All host overrides are restored.
function base64DecodeCases(createCodec) {
  const rows = [];
  const originalModern = Object.getOwnPropertyDescriptor(Uint8Array, 'fromBase64');
  const originalDecoder = Object.getOwnPropertyDescriptor(globalThis, 'TextDecoder');
  const originalAtob = Object.getOwnPropertyDescriptor(globalThis, 'atob');
  function restore(target, key, descriptor) {
    if (descriptor) Object.defineProperty(target, key, descriptor);
    else delete target[key];
  }
  function run(name, setup, input = 'QQ==') {
    const log = [], marker = {}, codec = createCodec();
    const modern = definition => Object.defineProperty(Uint8Array, 'fromBase64', { configurable: true, ...definition });
    const decoder = value => Object.defineProperty(globalThis, 'TextDecoder', { configurable: true, writable: true, value });
    const fallback = value => Object.defineProperty(globalThis, 'atob', { configurable: true, writable: true, value });
    try {
      setup({ log, marker, modern, decoder, fallback });
      let result;
      try {
        const value = codec.decodeText(typeof input === 'function' ? input(log) : input);
        result = value === marker ? { marker: true } : { type: typeof value, value: value === undefined ? null : value };
      } catch (error) {
        result = { thrownMarker: error === marker, type: error === marker ? 'marker' : error.name };
      }
      rows.push({ name, log, result });
    } finally {
      restore(Uint8Array, 'fromBase64', originalModern);
      restore(globalThis, 'TextDecoder', originalDecoder);
      restore(globalThis, 'atob', originalAtob);
      codec.free?.();
    }
  }
  for (const [name, make] of [
    ['Uint8Array', () => new Uint8Array([239,187,191,65,226,130,172,255])],
    ['ArrayBuffer', () => new Uint8Array([65,226,130,172]).buffer],
    ['DataView offset', () => new DataView(new Uint8Array([88,65,226,130,172,89]).buffer,1,4)],
    ['Uint16Array bytes', () => new Uint16Array(new Uint8Array([65,66,67,68]).buffer)],
    ['empty undefined', () => undefined],
    ['invalid array-like', () => ({0:65,length:1})],
    ['invalid null', () => null]
  ]) run('modern BufferSource '+name, ({ modern }) => modern({ value: make }));
  run('modern host order and receiver', ({ log, modern, decoder }) => {
    modern({ get() { log.push('fromBase64 get'); return function (...args) {
      log.push({ fromReceiver:this===Uint8Array,args }); return new Uint8Array([65]);
    }; } });
    decoder(function (...args) {
      log.push({ construct:!!new.target,args:args.length });
      const instance = { get decode() {
        log.push('decode get');
        return function (...args) {
          log.push({ decodeReceiver:this===instance,args:args.length,bytes:Array.from(args[0]) });
          return 'host';
        };
      } };
      return instance;
    });
  });
  for (const [name, value] of [['object', 'marker'],['undefined',undefined],['null',null],['number',42]]) {
    run('modern raw decoder '+name, ({ marker, modern, decoder }) => {
      modern({ value: () => new Uint8Array([65]) });
      decoder(function () { return { decode() { return value==='marker'?marker:value; } }; });
    });
  }
  run('outer capability getter propagates', ({ marker, modern, decoder }) => {
    modern({ get() { throw marker; } }); decoder(function () { throw Error('not reached'); });
  });
  run('inner capability getter caught after decoder getter', ({ marker, log, modern, decoder }) => {
    let reads=0;
    modern({ get() { log.push(++reads); if(reads===2)throw marker; return () => new Uint8Array(); } });
    decoder(function () { log.push('construct'); return { get decode() {log.push('decode'); return () => ''; } }; });
  });
  run('constructor exception caught before modern invocation', ({ marker, log, modern, decoder }) => {
    modern({ value() { log.push('unexpected'); return new Uint8Array(); } });
    decoder(function () { log.push('construct'); throw marker; });
  });
  run('decode getter exception prevents modern invocation', ({ marker, log, modern, decoder }) => {
    modern({ value() { log.push('unexpected'); return new Uint8Array(); } });
    decoder(function () { return { get decode() { log.push('decode'); throw marker; } }; });
  });
  run('noncallable decoder still evaluates argument', ({ log, modern, decoder }) => {
    modern({ value() { log.push('argument'); return new Uint8Array(); } });
    decoder(function () { return { decode:null }; });
  });
  run('modern exception caught', ({ marker, modern }) => modern({ value() { throw marker; } }));
  run('modern decoder exception caught', ({ marker, modern, decoder }) => {
    modern({ value:() => new Uint8Array() }); decoder(function () { return { decode() { throw marker; } }; });
  });
  run('modern input remains raw', ({ log, modern }) => modern({ value(value) { log.push(typeof value); return new Uint8Array([65]); } }), () => Symbol('input'));
  run('fallback native replacement and BOM', ({ modern, fallback }) => {
    modern({ value:undefined }); fallback(() => String.fromCharCode(239,187,191,65,255));
  });
  run('fallback decode receiver raw return and order', ({ log, marker, modern, fallback, decoder }) => {
    modern({ get() {log.push('capability'); return undefined;} });
    fallback(value => {log.push(['atob',value]);return 'A';});
    decoder(function () {log.push('construct'); const instance={get decode() {log.push('get');return function (bytes) {
      log.push({receiver:this===instance,bytes:Array.from(bytes)});return marker;
    };}};return instance;});
  });
  run('fallback constructor exception caught', ({ marker, modern, decoder }) => {
    modern({ value:undefined }); decoder(function () {throw marker;});
  });
  run('fallback decoder exception caught', ({ marker, modern, decoder }) => {
    modern({ value:undefined }); decoder(function () {return {decode() {throw marker;}};});
  });
  run('fallback standard ASCII', ({ modern }) => modern({ value:undefined }));
  return rows;
}
