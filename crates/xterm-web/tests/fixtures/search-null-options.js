// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
// Shared unchanged-source and Rust-export assertions, not a JS search engine.
async function checkSearchCallArgs(term, addon, write, equal) {
  await write('foo FOO foo');
  let cases = 0;
  const check = (label, actual, expected) => {
    try { equal(actual, expected); } catch (error) { throw Error(label + ': ' + error.message); }
  };
  for (const method of ['findNext', 'findPrevious']) {
    for (const selected of [false, true]) {
      for (const query of ['foo', 'absent']) {
        const child = new addon.constructor(); term.loadAddon(child);
        try {
          term.clearSelection(); if (selected) term.select(0, 0, 3);
          const log = [];
          child.onBeforeSearch(() => log.push('before'));
          child.onAfterSearch(() => log.push('after'));
          child.onDidChangeResults(() => log.push('results'));
          let caught;
          try { child[method](query, null, {get noScroll() { throw Error('noScroll before matching'); }}); }
          catch (error) { caught = error; }
          check('null request throws TypeError', caught instanceof TypeError, true);
          check('null fails on first engine regex read', caught.message.includes('regex'), true);
          check('matching error clears selection', term.hasSelection(), false);
          check('matching error stops after before event', log, ['before']);
          check('search recovers after null request', child.findNext('foo'), true);
          cases++;
        } finally { child.dispose(); }
      }
    }
    for (const query of ['', undefined, null, 0, false, {}, [], Symbol('invalid')]) {
      const child = new addon.constructor(); term.loadAddon(child);
      try {
        term.select(0, 0, 3);
        const log = [];
        child.onBeforeSearch(() => log.push('before'));
        child.onAfterSearch(() => log.push('after'));
        check('invalid term short circuits null request', child[method](query, null,
          {get noScroll() { throw Error('invalid term read noScroll'); }}), false);
        check('invalid term clears selection', term.hasSelection(), false);
        check('invalid term completes search events', log, ['before', 'after']);
        cases++;
      } finally { child.dispose(); }
    }
    for (const options of [undefined, false, 0, '', true, 1, Symbol('options')]) {
      term.clearSelection();
      check('non-null primitive request remains supported', addon[method]('foo', options), true);
      check('primitive request selects real cells', term.getSelection().toLowerCase(), 'foo');
      cases++;
    }
    for (const key of ['regex', 'caseSensitive']) {
      const child = new addon.constructor(); term.loadAddon(child);
      try {
        term.select(0, 0, 3);
        const sentinel = {key}; let caught;
        const options = {get [key]() {
          check('matching getter sees cleared selection', term.hasSelection(), false);
          // Nested matching and real selection must not encounter a held borrow.
          check('nested search from matching getter', child.findNext('FOO', {caseSensitive: true}), true);
          throw sentinel;
        }};
        try { child[method]('foo', options); } catch (error) { caught = error; }
        check('matching getter preserves thrown identity', caught === sentinel, true);
        check('nested search selection survives outer error', term.getSelection(), 'FOO');
        cases++;
      } finally { child.dispose(); }
    }
    {
      const child = new addon.constructor(); term.loadAddon(child);
      try {
        term.select(0, 0, 3); const sentinel = {}; let caught;
        child.onBeforeSearch(() => { throw sentinel; });
        try { child[method]('foo', null); } catch (error) { caught = error; }
        check('before callback precedes null error', caught === sentinel, true);
        check('before callback failure retains selection', term.getSelection(), 'foo');
        cases++;
      } finally { child.dispose(); }
    }
    {
      const child = new addon.constructor(); let before = 0, caught;
      child.onBeforeSearch(() => before++);
      try { child[method]('foo', null); } catch (error) { caught = error; }
      check('activation error precedes null request', caught?.message, 'Cannot use addon until it has been loaded');
      check('unloaded search has no before event', before, 0);
      child.dispose(); cases++;
    }
  }
  check('all directed null-option cases', cases, 46);
  return cases;
}
