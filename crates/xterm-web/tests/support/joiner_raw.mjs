// Shared operation inputs. Production source is compiled unchanged by the oracle.
export function liveJoinerFixture() {
  const first = [[1, '3']];
  const marker = {};
  const logged = [];
  const previous = console.error;
  return {
    first, marker, logged,
    handler() { return first; },
    failingConversion() { return [{get 0() { throw marker; }, 1: 3}]; },
    failingHandler() { throw marker; },
    installLogger() { console.error = error => logged.push(error); },
    restoreLogger() { console.error = previous; }
  };
}
export function joinerRawScenarios(create) {
  const result = {};
  const save = (name, body) => {
    const service = create();
    const errors = [];
    const previous = console.error;
    console.error = value => errors.push(value);
    try { result[name] = body(service, errors); }
    finally { console.error = previous; service.free?.(); }
  };
  save('identity', service => {
    const first = [[0, 2], [3, 5]];
    const added = [2, 3];
    const trace = [];
    service.register(function(text) { trace.push([this === undefined, text]); return first; });
    service.register(() => [added]);
    const out = service.get();
    return { first, added, out, newOuter: out !== first, sameInner: out[0] === first[0],
      insertedIdentity: out[1] === added, trace };
  });
  save('arrayLike', service => {
    const first = {0: {0: '1', 1: '3'}, length: 1};
    service.register(() => first);
    const out = service.get();
    return { first, out, sameInner: out[0] === first[0] };
  });
  save('emptyArrayLike', service => {
    service.register(() => ({}));
    return service.get();
  });
  save('falseHead', service => {
    const first = [false, [0, 2]];
    service.register(() => first);
    const out = service.get();
    return { first, out };
  });
  save('thrownHandler', (service, errors) => {
    const marker = {};
    service.register(() => { throw marker; });
    service.register(() => [[1, 3]]);
    const out = service.get();
    return { out, errors: errors.length, sameError: errors[0] === marker };
  });
  save('thrownLogger', service => {
    const marker = {}, logger = {};
    service.register(() => { throw marker; });
    const old = console.error;
    console.error = () => { throw logger; };
    try { service.get(); return { threw: false }; }
    catch (error) { return { threw: true, identity: error === logger }; }
    finally { console.error = old; }
  });
  save('secondaryError', (service, errors) => {
    const marker = {};
    service.register(() => [[0, 2]]);
    service.register(() => ({get length() { throw marker; }}));
    service.register(() => [[3, 5]]);
    const out = service.get();
    return { out, sameError: errors[0] === marker, errors: errors.length };
  });
  save('nullSecondary', (service, errors) => {
    service.register(() => []);
    service.register(() => null);
    service.register(() => [[1, 3]]);
    return { out: service.get(), errors: errors.map(error => error.name) };
  });
  save('conversionError', (service, errors) => {
    const marker = {};
    service.register(() => [{ get 0() { throw marker; }, 1: 3 }]);
    try { service.get(); return { threw: false }; }
    catch (error) { return { threw: true, identity: error === marker, logged: errors.length }; }
  });
  save('nullFirst', (service, errors) => {
    service.register(() => null);
    try { service.get(); return { threw: false }; }
    catch (error) { return { threw: true, name: error.name, logged: errors.length }; }
  });
  save('frozen', (service, errors) => {
    service.register(() => [Object.freeze([0, 2])]);
    try { service.get(); return { threw: false }; }
    catch (error) { return { threw: true, name: error.name, logged: errors.length }; }
  });
  save('frozenMerge', (service, errors) => {
    const first = Object.freeze([[0, 2]]);
    service.register(() => first);
    service.register(() => [[3, 5]]);
    return { out: service.get(), errors: errors.map(error => error.name) };
  });
  save('coercion', service => {
    const trace = [];
    const value = (name, n) => ({ [Symbol.toPrimitive](hint) { trace.push([name, hint]); return n; } });
    const first = [[value('start', 0.5), value('end', 2.5)]];
    service.register(() => first);
    const out = service.get();
    return { out, trace };
  });
  save('proxyOrder', service => {
    const trace = [];
    const range = new Proxy([0, 2], {
      get(target, key) { trace.push(['get', String(key)]); return Reflect.get(target, key); },
      set(target, key, value) { trace.push(['set', String(key), value]); return Reflect.set(target, key, value); }
    });
    const first = new Proxy([range], {
      get(target, key) { trace.push(['outer', String(key)]); return Reflect.get(target, key); }
    });
    service.register(() => first);
    const out = service.get();
    // Snapshot before JSON serialization reads the proxied inner range again.
    return { out: out.map(range => [range[0], range[1]]), trace: trace.slice() };
  });
  save('receiverMutation', service => {
    const trace = [];
    const handler = function(text) {
      trace.push([this.id, this.handler === handler, text]);
      this.id = 73;
      this.handler = function() { trace.push([this.id, 'replacement']); return [[1, 3]]; };
      return [];
    };
    const id = service.register(handler);
    const first = service.get();
    const originalRemoved = service.deregister(id);
    const second = service.get();
    const changedRemoved = service.deregister(73);
    return {first, second, originalRemoved, changedRemoved, trace};
  });
  save('liveHandlers', service => {
    const trace = [];
    let second;
    service.register(text => {
      trace.push('first:' + text);
      service.deregister(second);
      service.register(() => { trace.push('third'); return [[1, 3]]; });
      return [];
    });
    second = service.register(() => { trace.push('second'); return []; });
    const out = service.get();
    return { out, trace };
  });
  for (const [name, start, end] of [
    ['negative', -3, -1], ['fractional', 0.5, 2.5], ['nan', NaN, NaN],
    ['infinity', Infinity, Infinity], ['strings', '1', '4'], ['bigint', 1n, 4n],
    ['undefined', undefined, undefined], ['boolean', false, true]
  ]) save(name, service => {
    const first = [[start, end]];
    service.register(() => first);
    return service.get();
  });
  save('symbol', (service, errors) => {
    service.register(() => [[Symbol('start'), 2]]);
    try { service.get(); return { threw: false }; }
    catch (error) { return { threw: true, name: error.name, logged: errors.length }; }
  });
  save('mathLookupOrder', service => {
    const previous = Math.min;
    const trace = [];
    const added = { get 0() { Math.min = () => { trace.push('replacement'); return 99; }; return 1; }, 1: 2 };
    service.register(() => [[0, 3]]);
    service.register(() => [added]);
    try { return {out:service.get(), trace}; }
    finally { Math.min = previous; }
  });
  save('mergeGetterError', (service, errors) => {
    const marker = {};
    const range = [0, 2];
    service.register(() => [range]);
    service.register(() => [{get 1() { throw marker; }}]);
    const out = service.get();
    return { out, sameError: errors[0] === marker, count: errors.length };
  });
  // Exercise all nine source merge branches, retaining new inner identities.
  for (const [name, first, added] of [
    ['before', [[2, 3]], [0, 1]],
    ['inside', [[0, 4]], [1, 2]],
    ['front', [[2, 4]], [1, 3]],
    ['after', [[0, 1]], [2, 3]],
    ['bridgeGap', [[0, 3], [7, 9]], [2, 6]],
    ['bridgeNext', [[0, 3], [5, 9]], [2, 6]],
    ['pastAll', [[0, 3], [4, 5], [6, 7]], [2, 9]],
    ['extendLast', [[0, 3]], [2, 7]],
    ['adjacent', [[0, 2], [4, 5]], [2, 4]]
  ]) save('merge:' + name, service => {
    service.register(() => first);
    service.register(() => [added]);
    const out = service.get();
    return { out, first, added, identities: out.map(range => range === added ? 'added' : first.includes(range) ? 'first' : 'other') };
  });
  let seed = 0x335C0DE;
  const random = () => { seed ^= seed << 13; seed ^= seed >>> 17; seed ^= seed << 5; return seed >>> 0; };
  const endpoints = [undefined, null, false, true, -3, -1, 0, 0.5, 1, 2, 2.5, 4, 7, NaN, Infinity, -Infinity, '0', '2', 'x', 1n, 4n];
  for (let i = 0; i < 256; i++) {
    const groups = Array.from({length: 1 + random() % 3}, () =>
      Array.from({length: random() % 5}, () => [endpoints[random() % endpoints.length], endpoints[random() % endpoints.length]]));
    save('seed:' + i, (service, errors) => {
      for (const ranges of groups) service.register(() => ranges);
      try { return {out:service.get(), groups, errors:errors.map(error => error.name)}; }
      catch (error) { return {error:error.name, groups, errors:errors.map(error => error.name)}; }
    });
  }
  return JSON.stringify(result, (_key, value) => {
    if (typeof value === 'bigint') return 'bigint:' + value;
    if (typeof value === 'number' && !Number.isFinite(value)) return String(value);
    if (value === undefined) return 'undefined';
    return value;
  });
}
