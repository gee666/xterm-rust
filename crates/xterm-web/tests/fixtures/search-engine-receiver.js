// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
// Shared unchanged-source and real Rust/WASM assertions. Getter counts and
// successful raw-object matching are deliberately outside this scoped repair.
async function checkSearchCallArgs(term, addon, write, equal) {
  await write('a a a');
  let cases = 0;
  const position = col => ({start: {x: col, y: 0}, end: {x: col + 1, y: 0}});
  async function scenario(reverse, field, warm, nested, query, throwing, sentinel) {
    const parent = document.createElement('div'); document.body.appendChild(parent);
    const other = new term.constructor({cols: 20, rows: 3, allowProposedApi: true});
    const child = new addon.constructor(); term.loadAddon(child);
    let before, after;
    const check = (actual, expected) => {
      try { equal(actual, expected); } catch (error) {
        throw Error(JSON.stringify({reverse, field, warm, nested, query, throwing}) + ': ' + error.message);
      }
    };
    try {
      other.open(parent);
      await new Promise(resolve => other.write('    a a', resolve));
      if (warm) child.findNext('a');
      term.select(0, 0, 1); other.select(4, 0, 1);
      const events = [];
      before = child.onBeforeSearch(() => events.push('before'));
      after = child.onAfterSearch(() => events.push('after'));
      let armed = true;
      const request = {get [field]() {
        if (armed) {
          armed = false;
          child.activate(other);
          if (nested) check(child.findNext('a'), true);
          if (throwing) throw sentinel;
        }
        return false;
      }};
      let result, caught, failed = false;
      try { result = child[reverse ? 'findPrevious' : 'findNext'](query, request); }
      catch (error) { failed = true; caught = error; }
      check(armed, false);
      check(failed, throwing);
      check(term.hasSelection(), false);
      if (throwing) {
        check(Object.is(caught, sentinel), true);
        check(other.getSelectionPosition(), position(warm && nested ? 6 : 4));
        check(events, nested ? ['before', 'before', 'after'] : ['before']);
        check(child.findNext('a'), true);
        check(other.getSelectionPosition(), position(warm && nested ? 4 : warm || nested ? 6 : 4));
      } else {
        check(result, query === 'a');
        check(events, nested ? ['before', 'before', 'after', 'after'] : ['before', 'after']);
        if (query === 'a') {
          const col = warm ? reverse ? 4 : 2 : 0;
          check(other.getSelectionPosition(), position(col));
          check(child.findNext('a'), true);
          check(other.getSelectionPosition(), position(col === 4 ? 6 : 4));
        } else {
          check(other.hasSelection(), false);
          check(child.findNext('a'), true);
          check(other.getSelectionPosition(), position(4));
        }
      }
      // Neither the retired engine's text nor an old-host cursor event may
      // populate or expire the newly active engine's independently owned cache.
      await new Promise(resolve => term.write('\x1b[H', resolve));
      other.clearSelection();
      check(child.findNext('a'), true);
      check(other.getSelectionPosition(), position(4));
      check(child.findNext('a'), true);
      check(other.getSelectionPosition(), position(6));
      cases++;
    } finally {
      before?.dispose(); after?.dispose(); child.dispose(); other.dispose(); parent.remove();
    }
  }
  for (const reverse of [false, true]) {
    for (const field of ['regex', 'caseSensitive', 'wholeWord']) {
      for (const warm of [false, true]) {
        for (const nested of [false, true]) {
          await scenario(reverse, field, warm, nested, 'a', false);
          for (const sentinel of [Object.freeze({engineReceiver: true}), null, Symbol('engineReceiver')]) {
            await scenario(reverse, field, warm, nested, 'a', true, sentinel);
          }
          if (field !== 'wholeWord') await scenario(reverse, field, warm, nested, 'z', false);
        }
      }
    }
  }
  for (const reverse of [false, true]) {
    for (const field of ['regex', 'caseSensitive', 'wholeWord']) {
      const parent = document.createElement('div'); document.body.appendChild(parent);
      const other = new term.constructor({cols: 20, rows: 3, allowProposedApi: true});
      const child = new addon.constructor(); term.loadAddon(child);
      try {
        other.open(parent);
        await new Promise(resolve => other.write('    a a', resolve));
        term.select(0, 0, 1); other.select(4, 0, 1);
        let armed = true, changed;
        child.onDidChangeResults(value => changed = value);
        const request = {decorations: {}, get [field]() {
          if (armed) { armed = false; child.activate(other); }
          return false;
        }};
        equal(child[reverse ? 'findPrevious' : 'findNext']('a', request), true);
        equal(armed, false);
        equal(term.getSelectionPosition(), position(0));
        equal(other.getSelectionPosition(), position(4));
        // The first highlight uses A. Later engine calls and final selection
        // use B, so the result list is [A:0, B:4, B:6], not just B's two hits.
        equal(changed, {resultIndex: 1, resultCount: 3});
        equal(child.findNext('a'), true);
        equal(other.getSelectionPosition(), position(6));
        cases++;
      } finally { child.dispose(); other.dispose(); parent.remove(); }
    }
  }
  equal(cases, 118);
  return cases;
}
