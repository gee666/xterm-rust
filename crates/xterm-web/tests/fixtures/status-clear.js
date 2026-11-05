// Shared operation trace for the original addon and Rust's real browser addon.
async function statusClearCases(create, feed) {
  const cases = [
    ['omitted', () => []],
    ['undefined', () => [undefined]],
    ['empty', () => ['']],
    ['string', () => ['a']],
    ['null', () => [null]],
    ['number', () => [1]],
    ['bigint', () => [1n]],
    ['boolean', () => [false]],
    ['boxed-string', () => [new String('a')]],
    ['array', () => [['a']]],
    ['object', log => [{ [Symbol.toPrimitive](hint) { log.push(hint); return 'a'; } }]],
    ['throw', (log, addon, thrown) => [{ [Symbol.toPrimitive](hint) { log.push(hint); throw thrown; } }]],
    ['symbol', () => [Symbol('a')]],
    ['bad-primitive', log => [{ [Symbol.toPrimitive](hint) { log.push(hint); return {}; } }]],
    ['reentry', (log, addon) => [{ [Symbol.toPrimitive](hint) { log.push(hint); addon.clear('a'); return 'ab'; } }]],
    ['dispose-in-coercion', (log, addon) => [{ [Symbol.toPrimitive](hint) { log.push(hint); addon.dispose(); return 'a'; } }]],
    ['empty-store-coercion', (log, addon, thrown) => [{ [Symbol.toPrimitive](hint) { log.push(hint); throw thrown; } }]]
  ];
  const rows = [];
  for (const [name, argument] of cases) {
    const { addon, close } = create();
    try {
      if (name !== 'empty-store-coercion') {
        for (const id of ['', 'a', 'a/b', 'ab', 'ab/c', 'null', 'null/x', '1', '1/x', 'false', 'false/x']) {
          await feed(addon, `state=working${id ? ':id=' + id : ''}`);
        }
      }
      let events = 0;
      addon.onChange(() => events++);
      const before = addon.records, log = [], thrown = { marker: name };
      let error = null;
      try { addon.clear(...argument(log, addon, thrown)); }
      catch (caught) { error = caught === thrown ? 'identity' : caught instanceof TypeError ? 'TypeError' : String(caught); }
      rows.push({ name, ids: addon.records.map(record => record.id), events, sameSnapshot: addon.records === before, log, error });
    } finally { close(); }
  }
  return rows;
}
