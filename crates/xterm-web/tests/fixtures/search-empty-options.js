// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
// Shared assertions run unchanged against source and actual Rust Terminal.
async function checkSearchCallArgs(term, addon, write, equal) {
  await write('a a');
  let cases = 0;
  const flags = ['caseSensitive', 'regex', 'wholeWord'];
  const sentinel = {emptySearchGetterFailure: true};
  const caught = action => {
    try { action(); } catch (error) { if (error !== sentinel) throw error; return true; }
    return false;
  };
  for (const direction of ['findNext', 'findPrevious']) {
    const log = [];
    const before = addon.onBeforeSearch(() => log.push('before'));
    const after = addon.onAfterSearch(() => log.push('after'));
    const changed = addon.onDidChangeResults(event => {
      equal(event, {resultIndex: -1, resultCount: 0});
      log.push('results');
    });
    try {
      for (const decorations of [undefined, null, false, 0, NaN, '', 0n, true, {}, 1, [], Symbol('decoration')]) {
        addon.clearDecorations();
        term.select(0, 0, 1);
        log.length = 0;
        const options = {get decorations() { log.push('decorations'); return decorations; }};
        for (const key of flags) Object.defineProperty(options, key, {get() { throw sentinel; }});
        equal(addon[direction]('', options), false);
        equal(term.hasSelection(), false);
        equal(log, ['before', 'decorations', 'decorations', ...(decorations ? ['results'] : []), 'after']);
        cases++;
      }
      // An empty cached term does not defeat the decoration short circuit.
      log.length = 0;
      const undecorated = {get decorations() { log.push('decorations'); return false; }};
      for (const key of flags) Object.defineProperty(undecorated, key, {get() { throw sentinel; }});
      equal(addon[direction]('', undecorated), false);
      equal(log, ['before', 'decorations', 'decorations', 'after']);
      cases++;
      // Cached empty + truthy decorations does compare the request to itself.
      log.length = 0;
      const same = {get decorations() { log.push('decorations'); return {}; }};
      for (const key of flags) Object.defineProperty(same, key, {get() { log.push(key); return false; }});
      equal(addon[direction]('', same), false);
      equal(log, ['before', 'decorations', ...flags.flatMap(key => [key, key]), 'decorations', 'results', 'after']);
      cases++;
      for (const target of flags) {
        for (const mode of ['sameObject', 'changed', 'throw']) {
          addon.findNext('');
          log.length = 0;
          const options = {get decorations() { log.push('decorations'); return {}; }};
          const identity = {};
          let reads = 0;
          for (const key of flags) Object.defineProperty(options, key, {get() {
            log.push(key);
            if (key !== target) return false;
            reads++;
            if (mode === 'throw') throw sentinel;
            if (mode === 'sameObject') return identity;
            return reads === 1 ? NaN : false;
          }});
          equal(caught(() => equal(addon[direction]('', options), false)), mode === 'throw');
          const compared = mode === 'sameObject' ? flags : flags.slice(0, flags.indexOf(target) + 1);
          const expected = ['before', 'decorations'];
          for (const key of compared) expected.push(key, ...(mode === 'throw' && key === target ? [] : [key]));
          if (mode !== 'throw') expected.push('decorations', 'results', 'after');
          equal(log, expected);
          cases++;
        }
      }
      for (const throwAt of [1, 2]) {
        addon.clearDecorations();
        term.select(0, 0, 1);
        log.length = 0;
        let reads = 0;
        const options = {get decorations() {
          log.push('decorations');
          if (++reads === throwAt) throw sentinel;
          return false;
        }};
        equal(caught(() => addon[direction]('', options)), true);
        equal(term.hasSelection(), throwAt === 1);
        equal(log, ['before', ...Array(throwAt).fill('decorations')]);
        cases++;
      }
      addon.clearDecorations();
      log.length = 0;
      let nested = false;
      const reentrant = {get decorations() {
        log.push('decorations');
        if (!nested) { nested = true; addon.findNext('a'); }
        return false;
      }};
      equal(addon[direction]('', reentrant), false);
      equal(term.hasSelection(), false);
      equal(log, ['before', 'decorations', 'before', 'after', 'decorations', 'after']);
      cases++;
      addon.clearDecorations();
      let reads = 0;
      const pending = {get decorations() { reads++; return false; }};
      for (const key of flags) Object.defineProperty(pending, key, {get() { throw sentinel; }});
      addon[direction]('', pending);
      equal(reads, 2);
      await write('b');
      equal(reads, 2);
      cases++;
    } finally { before.dispose(); after.dispose(); changed.dispose(); }
  }
  return cases;
}
