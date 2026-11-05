// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
// Shared source/Rust public-binding assertions. Matching is not implemented here.
async function checkSearchCallArgs(term, addon, write, equal) {
  await write('foo FOO');
  let cases = 0;
  const check = (label, actual, expected) => {
    try { equal(actual, expected); } catch (error) { throw Error(label + ': ' + error.message); }
  };
  const thrown = [{sentinel: true}, null, undefined, Symbol('sentinel'), 'sentinel', NaN];
  const colors = {activeMatchBackground: '#334455', activeMatchColorOverviewRuler: '#00ff00'};
  for (const method of ['findNext', 'findPrevious']) {
    for (const query of ['foo', 'absent']) {
      for (const selected of [false, true]) {
        for (const active of [false, true]) {
          for (const action of ['return', 'nested', 'throw', 'nestedThrow', 'scrollThrow']) {
            for (const sentinel of action.includes('Throw') || action === 'throw' ? thrown : [undefined]) {
              const child = new addon.constructor(); term.loadAddon(child);
              try {
                term.clearSelection(); if (selected) term.select(0, 0, 3);
                const log = []; let reads = 0, matching = 0, failed = false, caught, found;
                child.onBeforeSearch(() => log.push('before'));
                child.onAfterSearch(() => log.push('after'));
                child.onDidChangeResults(() => log.push('results'));
                const flags = key => {
                  check('matching must precede selection decoration read ' + key, reads, 1);
                  check('matching sees cleared selection', term.hasSelection(), false);
                  matching++;
                  return key === 'caseSensitive';
                };
                const request = {
                  get regex() { return flags('regex'); },
                  get caseSensitive() { return flags('caseSensitive'); },
                  get wholeWord() { flags('wholeWord'); return false; },
                  get decorations() {
                    log.push('d' + ++reads);
                    if (reads !== 2) return undefined;
                    check('selection decoration read follows matching', matching > 0, true);
                    check('selection decoration getter sees cleared selection', term.hasSelection(), false);
                    if (action === 'nested' || action === 'nestedThrow') {
                      check('selection getter can reenter search', child.findNext('FOO', {caseSensitive: true}), true);
                      check('nested getter selects real cells', term.getSelection(), 'FOO');
                    }
                    if (action === 'throw' || action === 'nestedThrow') throw sentinel;
                    return active ? colors : undefined;
                  }
                };
                try {
                  found = child[method](query, request, {get noScroll() {
                    log.push('scroll');
                    check('noScroll follows selection decoration getter', reads, 2);
                    check('nested selection survives until noScroll', term.getSelection(), action === 'nested' ? 'FOO' : '');
                    if (action === 'scrollThrow') throw sentinel;
                    return true;
                  }});
                } catch (error) { failed = true; caught = error; }
                const nested = action === 'nested' || action === 'nestedThrow';
                const expected = ['before', 'd1', 'd2', ...(nested ? ['before', 'after'] : [])];
                if (action !== 'throw' && action !== 'nestedThrow') expected.push('scroll');
                const throws = action === 'throw' || action === 'nestedThrow' || action === 'scrollThrow';
                if (!throws) expected.push('d3', 'after');
                check('observable source phase order', log, expected);
                check('throw outcome', failed, throws);
                if (throws) {
                  check('custom thrown value identity', Object.is(caught, sentinel), true);
                  check('failure retains nested selection only', term.getSelection(), nested ? 'FOO' : '');
                } else {
                  check('search result', found, query === 'foo');
                  check('real selection', term.getSelection(), query === 'foo' ? 'foo' : '');
                  check('matching has completed before selected result', matching > 0, true);
                }
                cases++;
              } finally { child.dispose(); }
            }
          }
        }
      }
    }
    for (const key of ['regex', 'caseSensitive']) {
      for (const sentinel of thrown) {
        const child = new addon.constructor(); term.loadAddon(child);
        try {
          term.select(0, 0, 3);
          let reads = 0, failed = false, caught;
          const log = [];
          child.onBeforeSearch(() => log.push('before'));
          child.onAfterSearch(() => log.push('after'));
          child.onDidChangeResults(() => log.push('results'));
          const request = {
            get decorations() {
              reads++; check('matching failure skips late decoration getter', reads, 1); return undefined;
            },
            get [key]() { throw sentinel; }
          };
          try { child[method]('foo', request, {get noScroll() { throw Error('early noScroll'); }}); }
          catch (error) { failed = true; caught = error; }
          check('matching failed', failed, true);
          check('matching failure identity', Object.is(caught, sentinel), true);
          check('matching failure clears selection', term.hasSelection(), false);
          check('matching failure event cutoff', log, ['before']);
          check('only initial decoration read before engine failure', reads, 1);
          cases++;
        } finally { child.dispose(); }
      }
    }
    for (const query of ['', undefined, null, 0, false, {}, [], Symbol('invalid')]) {
      const child = new addon.constructor(); term.loadAddon(child);
      try {
        term.select(0, 0, 3); let reads = 0;
        const log = [];
        child.onBeforeSearch(() => log.push('before'));
        child.onAfterSearch(() => log.push('after'));
        const request = {
          get decorations() { reads++; return undefined; },
          get regex() { throw Error('invalid term reached engine'); }
        };
        check('invalid term short circuit', child[method](query, request,
          {get noScroll() { throw Error('invalid term reached noScroll'); }}), false);
        check('invalid term reads only eligibility and result decorations', reads, 2);
        check('invalid term clears selection', term.hasSelection(), false);
        check('invalid term events', log, ['before', 'after']);
        cases++;
      } finally { child.dispose(); }
    }
  }
  check('directed selection option cases', cases, 360);
  return cases;
}
