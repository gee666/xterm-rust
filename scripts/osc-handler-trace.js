// Shared probe of the original OSC provider and the production Rust registry.
return (async () => {
  const trace = [];
  const marker = { marker: 'exact thrown identity' };
  const savedConfirm = Object.getOwnPropertyDescriptor(window, 'confirm');
  let log;
  Object.defineProperty(window, 'confirm', { configurable: true, value(text) {
    log.push(['confirm', text]);
    return false;
  } });
  const label = value => typeof value === 'number' && Object.is(value, -0) ? '-0' : String(value);
  async function run(name, handler, uri = 'custom:resource', actions = ['hover', 'activate', 'leave']) {
    log = [];
    try {
      const count = await probe(handler, uri, actions);
      trace.push({ name, count, log });
    } catch (error) {
      trace.push({ name, error: error === marker ? 'marker' : error.name, log });
    }
  }
  async function primitive(value, prototype) {
    const saved = new Map(['allowNonHttpProtocols', 'activate', 'hover', 'leave'].map(key =>
      [key, Object.getOwnPropertyDescriptor(prototype, key)]));
    let range;
    try {
      Object.defineProperty(prototype, 'allowNonHttpProtocols', { configurable: true, get() {
        'use strict';
        log.push(['policy', typeof this, Object.is(this, value)]);
        return true;
      } });
      for (const name of ['activate', 'hover', 'leave']) {
        Object.defineProperty(prototype, name, { configurable: true, get() {
          'use strict';
          log.push(['get', name, typeof this, Object.is(this, value)]);
          return function(event, text, currentRange) {
            'use strict';
            log.push(['call', name, typeof this, Object.is(this, value), event.type, text,
              range === undefined || range === currentRange, currentRange.start.x]);
            range = currentRange;
            currentRange.start.x = 7;
          };
        } });
      }
      await run(`primitive:${typeof value}:${label(value)}`, value);
      Object.defineProperty(prototype, 'allowNonHttpProtocols', { configurable: true, get() {
        throw marker;
      } });
      await run(`throw-policy:${typeof value}:${label(value)}`, value);
    } finally {
      for (const [key, descriptor] of saved) {
        if (descriptor) Object.defineProperty(prototype, key, descriptor);
        else delete prototype[key];
      }
    }
  }
  try {
    for (const value of [false, true]) await primitive(value, Boolean.prototype);
    for (const value of [0, -0, NaN, 2]) await primitive(value, Number.prototype);
    for (const value of ['', 'text']) await primitive(value, String.prototype);
    for (const value of [0n, 2n]) await primitive(value, BigInt.prototype);
    await primitive(Symbol('handler'), Symbol.prototype);
    await run('null', null, 'https://example.com/');
    await run('undefined', undefined, 'https://example.com/');
    for (const name of ['activate', 'hover', 'leave']) {
      const handler = { allowNonHttpProtocols: true };
      Object.defineProperty(handler, name, { get() { throw marker; } });
      await run(`throw-member:${name}`, handler, 'custom:resource', [name]);
    }
    for (const value of [null, undefined]) {
      await run(`nullish-method:${String(value)}`, { allowNonHttpProtocols: true, hover: value,
        leave: value }, 'custom:resource', ['hover', 'leave']);
    }
    await run('noncallable-hover', { allowNonHttpProtocols: true, hover: 0 }, 'custom:resource', ['hover']);
    const handler = { allowNonHttpProtocols: true, hover(event, text, range) {
      log.push(['object', this === handler, event.type, text, range.end.x]);
    } };
    await run('object-hover', handler, 'custom:resource', ['hover']);
    return trace;
  } finally {
    if (savedConfirm) Object.defineProperty(window, 'confirm', savedConfirm);
    else delete window.confirm;
  }
})();
