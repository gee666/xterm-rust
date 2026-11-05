// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
// One fixture exercises the unchanged source and actual Rust public binding.
async function checkSearchCallArgs(term, addon, write, equal) {
  await write('foo FOO');
  let cases = 0;
  const check = (label, actual, expected) => {
    try { equal(actual, expected); } catch (error) { throw Error(label + ': ' + error.message); }
  };
  const sentinels = [{sentinel: true}, null, undefined, Symbol('sentinel'), 'sentinel', NaN];
  const eligibility = {get matchBackground() { throw Error('eligibility object used as argument'); }};
  const actions = ['return', 'skip', 'throwEligibility', 'throwArgument', 'nestedEligibility',
    'nestedArgument', 'nestedEligibilityThrow', 'nestedArgumentThrow'];
  for (const method of ['findNext', 'findPrevious']) {
    for (const query of ['foo', 'absent']) {
      for (const action of actions) {
        const throws = action.toLowerCase().includes('throw');
        for (const sentinel of throws ? sentinels : [undefined]) {
          let limitReads = 0;
          const child = new addon.constructor({highlightLimit: {valueOf() { limitReads++; return 100; }}});
          term.loadAddon(child);
          try {
            term.select(0, 0, 3);
            let reads = 0, backgrounds = 0, caught, failed = false, found, report;
            const log = [];
            child.onBeforeSearch(() => log.push('before'));
            child.onAfterSearch(() => log.push('after'));
            child.onDidChangeResults(value => { report = value; });
            const argument = {get matchBackground() { backgrounds++; return '#112233'; }, matchOverviewRuler: '#ff0000'};
            const request = {caseSensitive: false, get decorations() {
              log.push('d' + ++reads);
              if (reads === 1) return eligibility;
              if (reads === 2 || reads === 3 && action !== 'skip') {
                check('highlight results sliced before late getter', limitReads > 0, true);
                check('highlight getter precedes selection clear', term.getSelection(),
                  reads === 3 && action.startsWith('nestedEligibility') ? 'FOO' : 'foo');
                const phase = reads === 2 ? 'Eligibility' : 'Argument';
                if (action.toLowerCase().includes(phase.toLowerCase())) {
                  if (action.startsWith('nested')) {
                    check('late getter reentry', child.findNext('FOO', {caseSensitive: true}), true);
                    check('nested real selection', term.getSelection(), 'FOO');
                  }
                  if (throws) throw sentinel;
                }
                if (reads === 2) return action === 'skip' ? undefined : eligibility;
                return argument;
              }
              // Suppress active decorations but emit the outer result count.
              return reads === (action === 'skip' ? 4 : 5) ? {} : undefined;
            }};
            try { found = child[method](query, request); }
            catch (error) { failed = true; caught = error; }
            check('throw cutoff', failed, throws);
            if (throws) {
              check('exact thrown value', Object.is(caught, sentinel), true);
              check('failed read count', reads, action.toLowerCase().includes('eligibility') ? 2 : 3);
              check('failure retains selection', term.getSelection(), action.startsWith('nested') ? 'FOO' : 'foo');
              check('no color before failed argument', backgrounds, 0);
            } else {
              check('search result', found, query === 'foo');
              // Nested search replaces the cached term and selected position;
              // the outer search then expands from that nested selection.
              check('final selection', term.getSelection(), query === 'foo' ? action.startsWith('nested') ? 'FOO' : 'foo' : '');
              check('distinct argument object colors', backgrounds, query === 'foo' && action !== 'skip' ? 2 : 0);
              check('decoration phase reads', reads, action === 'skip' ? 4 : 5);
              check('stored result count survives late getter', report, {resultIndex: -1, resultCount: query === 'foo' ? 2 : 0});
            }
            check('event starts before decoration access', log.slice(0, 2), ['before', 'd1']);
            check('after event cutoff', log.filter(value => value === 'after').length,
              Number(action.startsWith('nested')) + Number(!throws));
            cases++;
          } finally { child.dispose(); }
        }
      }
      for (const value of [undefined, null, false, 0, '', NaN, Symbol('decoration')]) {
        const child = new addon.constructor(); term.loadAddon(child);
        try {
          let reads = 0, failed = false, caught;
          const request = {get decorations() { return ++reads === 3 ? value : reads < 3 ? {} : undefined; }};
          try { child[method](query, request); } catch (error) { failed = true; caught = error; }
          const invalid = value == null && query === 'foo';
          check('raw falsy argument failure', failed, invalid);
          if (invalid) {
            check('nullish colors are TypeError', caught instanceof TypeError, true);
            check('nullish color error identifies matchBackground', caught.message.includes('matchBackground'), true);
          }
          check('raw falsy argument read count', reads, invalid ? 3 : 5);
          cases++;
        } finally { child.dispose(); }
      }
    }
    for (const activateAt of [2, 3]) {
      const other = new term.constructor({cols: 20, rows: 3, allowProposedApi: true});
      const parent = document.createElement('div'); document.body.appendChild(parent);
      const child = new addon.constructor(); term.loadAddon(child);
      try {
        other.open(parent);
        await new Promise(resolve => other.write('foo FOO', resolve));
        const originalMarkers = term.markers.length, otherMarkers = other.markers.length;
        const seen = []; let reads = 0;
        const colors = {get matchBackground() {
          seen.push([term.markers.length - originalMarkers, other.markers.length - otherMarkers]);
          return '#112233';
        }, matchOverviewRuler: '#ff0000'};
        const request = {get decorations() {
          if (++reads === activateAt) child.activate(other);
          return reads <= 3 ? colors : undefined;
        }};
        check('reactivation search result', child[method]('foo', request), true);
        check('manager capture between late getters', seen,
          activateAt === 2 ? [[0, 1], [0, 2]] : [[1, 0], [2, 0]]);
        cases++;
      } finally { child.dispose(); other.dispose(); parent.remove(); }
    }
    for (const key of ['regex', 'caseSensitive']) {
      for (const sentinel of sentinels) {
        const child = new addon.constructor(); term.loadAddon(child);
        try {
          term.select(0, 0, 3); let reads = 0, failed = false, caught;
          const request = {get decorations() { reads++; return {}; }, get [key]() { throw sentinel; }};
          try { child[method]('foo', request); } catch (error) { failed = true; caught = error; }
          check('highlight matching fails', failed, true);
          check('matching throw identity', Object.is(caught, sentinel), true);
          check('matching failure skips late highlight reads', reads, 1);
          check('highlight failure precedes selection clear', term.getSelection(), 'foo');
          cases++;
        } finally { child.dispose(); }
      }
    }
    for (const query of ['', undefined, null, 0, false, {}, [], Symbol('invalid')]) {
      const child = new addon.constructor(); term.loadAddon(child);
      try {
        let reads = 0;
        const request = {get decorations() { reads++; return {}; }, get regex() { throw Error('invalid matching'); }};
        check('invalid term short circuit', child[method](query, request), false);
        check('invalid term skips both late highlight reads', reads, 2);
        cases++;
      } finally { child.dispose(); }
    }
  }
  check('highlight directed cases', cases, 184);
  return cases;
}
