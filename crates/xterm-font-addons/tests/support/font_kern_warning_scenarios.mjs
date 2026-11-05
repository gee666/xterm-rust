// Shared source/Rust host inputs. Font parsing and addon decisions are not mocked.
import {withKern} from './font_metadata_scenarios.mjs';
export async function kernWarningScenarios(create, bytes) {
  const warning = 'Only the first kern subtable is supported.';
  const payload = new Uint8Array(18), view = new DataView(payload.buffer);
  view.setUint16(0, 1); view.setUint32(4, 2);
  const valid = withKern(bytes, payload);
  const truncated = withKern(bytes, payload.slice(0, 8));
  const owner = globalThis.window ?? {};
  const descriptors = [
    [globalThis, 'window'], [globalThis, 'navigator'], [owner, 'queryLocalFonts'], [console, 'warn'], [console, 'debug']
  ].map(([object, key]) => [object, key, Object.getOwnPropertyDescriptor(object, key)]);
  if (!globalThis.window) Object.defineProperty(globalThis, 'window', {configurable: true, value: owner});
  Object.defineProperty(globalThis, 'navigator', {configurable: true, value: {}});
  const families = ['Valid', 'Truncated', 'Throw', 'GetterThrow', 'PrimitiveThrow', 'SymbolThrow', 'NullThrow', 'UndefinedThrow', 'Reenter', 'Change', 'Nested', 'AfterThrow'];
  Object.defineProperty(owner, 'queryLocalFonts', {configurable: true, value: async () =>
    families.map(family => ({family, async blob() { return {async arrayBuffer() {
      const data = family === 'Truncated' ? truncated : valid;
      return data.buffer.slice(data.byteOffset, data.byteOffset + data.byteLength);
    }}; }})) });
  const results = {};
  try {
    for (const mode of ['Valid', 'Truncated', 'Throw', 'GetterThrow', 'PrimitiveThrow', 'SymbolThrow', 'NullThrow', 'UndefinedThrow', 'Reenter', 'Change']) {
      const trace = [], marker = {};
      const throwing = mode.endsWith('Throw');
      const thrown = mode === 'PrimitiveThrow' ? 17 : mode === 'SymbolThrow' ? Symbol(mode) :
        mode === 'NullThrow' ? null : mode === 'UndefinedThrow' ? undefined : {mode};
      let join, family = mode, complete, inWarning = false, pending = mode === 'Change' ? 2 : 1;
      let refreshes = 0, previousIdentity = false;
      const term = {rows: 5, element: {style: {}}, options: {
        logLevel: 'debug',
        get fontFamily() {
          if (!inWarning && complete && --pending === 0) { const done = complete; complete = undefined; done(); }
          return family;
        }
      }, registerCharacterJoiner(callback) {join = callback; return 0;},
      deregisterCharacterJoiner() {}, refresh() { refreshes++; trace.push('refresh'); }};
      const addon = create({fallbackLigatures: ['===']});
      Object.defineProperty(console, 'debug', {configurable: true, value(previous) {
        previousIdentity = previous === thrown; trace.push(previousIdentity ? 'debug:thrown' : 'debug:other');
      }});
      Object.defineProperty(console, 'warn', {configurable: true, get() {
        trace.push('warn:get');
        if (mode === 'GetterThrow') throw thrown;
        return function(message) {
          trace.push(['warn:call', this === console, message === warning]);
          if (throwing) throw thrown;
          if ((mode === 'Reenter' || mode === 'Change') && family !== 'Nested') {
            inWarning = true;
            try {
              if (mode === 'Change') family = 'Nested';
              trace.push(['reenter', join('===')]);
            } finally { inWarning = false; }
          }
          return marker;
        };
      }});
      try {
        addon.activate(term);
        const loading = join('===');
        await new Promise(resolve => {complete = resolve;});
        const first = {loading, ranges: join('==='), refreshes, trace: trace.slice()};
        // A second rejection exposes the previous error through the source's
        // debug callback, proving exact thrown-value identity was retained.
        if (throwing) {
          family = 'AfterThrow'; pending = 1;
          join('===');
          await new Promise(resolve => {complete = resolve;});
        }
        results[mode] = {first, previousIdentity, cached: join('==='), refreshes};
      } finally {addon.dispose();}
    }
    return results;
  } finally {
    for (const [object, key, descriptor] of descriptors.reverse()) {
      if (descriptor) Object.defineProperty(object, key, descriptor); else delete object[key];
    }
  }
}
