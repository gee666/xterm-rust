// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
// Runs unchanged against the original classes and actual Rust browser exports.
async function checkSearchCallArgs(term, addon, write, equal) {
  await write('a a');
  const flags = ['caseSensitive', 'regex', 'wholeWord'];
  const sentinel = {invalidTermFailure: true};
  const invalid = [undefined, null, false, 0, -0, NaN, '', 0n, 1, -1, true, 1n,
    Symbol('term'), {}, [], function () {}, new Date(0), /x/, Object.create(null),
    new String(''), new Number(1), new Boolean(true)];
  const lengths = [undefined, null, false, 0, -1, NaN, -Infinity, '0', 'no', 0n];
  let cases = 0;
  const caught = callback => {
    try { callback(); } catch (error) {
      if (error !== sentinel) throw error;
      return true;
    }
    return false;
  };
  for (const direction of ['findNext', 'findPrevious']) {
    const log = [];
    const before = addon.onBeforeSearch(() => log.push('before'));
    const after = addon.onAfterSearch(() => log.push('after'));
    const changed = addon.onDidChangeResults(() => log.push('results'));
    const options = (decorations, poison = true) => {
      const value = {get decorations() { log.push('decorations'); return decorations; }};
      for (const key of flags) Object.defineProperty(value, key, {get() {
        if (poison) throw sentinel;
        log.push(key);
        return false;
      }});
      return value;
    };
    try {
      for (const value of invalid) {
        for (const decorations of [false, {}]) {
          addon.clearDecorations(); term.select(0, 0, 1); log.length = 0;
          equal(addon[direction](value, options(decorations)), false);
          equal(term.hasSelection(), false);
          equal(log, ['before', 'decorations', 'decorations', ...(decorations ? ['results'] : []), 'after']);
          cases++;
        }
        // Cache equality is raw ===, not text conversion or invalid/empty collapse.
        addon.clearDecorations(); addon[direction](value); log.length = 0;
        equal(addon[direction](value, options({}, false)), false);
        const compared = value !== undefined && value === value;
        equal(log, ['before', 'decorations', ...(compared ? flags.flatMap(key => [key, key]) : []), 'decorations', 'results', 'after']);
        cases++;
      }
      for (const length of lengths) {
        for (const decorations of [false, {}]) {
          addon.clearDecorations(); log.length = 0;
          const value = {get length() { log.push('length'); return length; }};
          equal(addon[direction](value, options(decorations)), false);
          equal(log, ['before', 'decorations', ...(decorations ? ['length'] : []), 'length', 'decorations', ...(decorations ? ['results'] : []), 'after']);
          cases++;
        }
      }
      for (const [decorations, throwAt] of [[false, 1], [{}, 1], [{}, 2]]) {
        addon.clearDecorations(); term.select(0, 0, 1); log.length = 0;
        let reads = 0;
        const value = {get length() { log.push('length'); if (++reads === throwAt) throw sentinel; return 0; }};
        equal(caught(() => addon[direction](value, options(decorations))), true);
        equal(term.hasSelection(), true);
        equal(log, ['before', 'decorations', ...Array(throwAt).fill('length')]);
        cases++;
      }
      addon.clearDecorations(); log.length = 0;
      let symbolError = false;
      try { addon[direction]({get length() { log.push('length'); return Symbol('length'); }}, options({})); }
      catch (error) { if (!(error instanceof TypeError)) throw error; symbolError = true; }
      equal(symbolError, true); equal(log, ['before', 'decorations', 'length']); cases++;

      addon.clearDecorations(); log.length = 0;
      const poisoned = {get length() { throw Error('length read before decorations'); }};
      equal(caught(() => addon[direction](poisoned, {get decorations() { log.push('decorations'); throw sentinel; }})), true);
      equal(log, ['before', 'decorations']); cases++;

      addon.clearDecorations(); log.length = 0;
      let nested = false;
      const reentrant = {get length() {
        log.push('length');
        if (!nested) { nested = true; equal(addon.findNext('a'), true); }
        return 0;
      }};
      equal(addon[direction](reentrant, options({})), false);
      equal(term.hasSelection(), false);
      equal(log, ['before', 'decorations', 'length', 'before', 'after', 'length', 'decorations', 'results', 'after']); cases++;

      addon.clearDecorations(); addon[direction]({length: 0}); log.length = 0;
      equal(addon[direction]({length: 0}, options({})), false);
      equal(log, ['before', 'decorations', 'decorations', 'results', 'after']); cases++;

      for (const decorations of [false, {}]) {
        addon.clearDecorations(); log.length = 0;
        const value = {get length() { log.push('length'); return {valueOf() { log.push('valueOf'); throw sentinel; }}; }};
        equal(caught(() => addon[direction](value, options(decorations))), true);
        equal(log, ['before', 'decorations', 'length', 'valueOf']); cases++;
      }
      // Write invalidation tests raw term truthiness, not term validity/UTF16 length.
      for (const value of invalid) {
        addon.clearDecorations(); let reads = 0;
        const pending = {get decorations() { reads++; return false; }};
        for (const key of flags) Object.defineProperty(pending, key, {get() { throw sentinel; }});
        addon[direction](value, pending); equal(reads, 2);
        await write('b'); equal(reads, 2 + (value ? 1 : 0)); cases++;
      }
      const child = new addon.constructor();
      let loadedError = false;
      try { child[direction](poisoned); }
      catch (error) { if (error.message !== 'Cannot use addon until it has been loaded') throw error; loadedError = true; }
      equal(loadedError, true); child.dispose(); cases++;

      const failure = addon.onBeforeSearch(() => { throw sentinel; });
      equal(caught(() => addon[direction](poisoned)), true); failure.dispose(); cases++;
      let length = 1;
      const edit = addon.onBeforeSearch(() => { length = 0; });
      equal(addon[direction]({get length() { return length; }}), false); edit.dispose(); cases++;
    } finally { before.dispose(); after.dispose(); changed.dispose(); }
  }
  return cases;
}
