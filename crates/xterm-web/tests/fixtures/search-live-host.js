// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
async function checkSearchCallArgs(term, addon, write, equal) {
  await write('foo foo foo');
  let cases = 0;
  const check = (label, actual, expected) => {
    try { equal(actual, expected); } catch (error) { throw Error(label + ': ' + error.message); }
  };
  async function scenario(method, phase, query, sentinel) {
    const other = new term.constructor({cols: 20, rows: 3, allowProposedApi: true});
    const parent = document.createElement('div'); document.body.appendChild(parent);
    const child = new addon.constructor(); term.loadAddon(child);
    try {
      other.open(parent);
      await new Promise(resolve => other.write('foo foo foo', resolve));
      // Seed strict cached-term equality, then give the two hosts different starts.
      child.findNext('foo');
      if (phase.startsWith('highlight')) child.clearDecorations();
      term.select(0, 0, 3); other.select(4, 0, 3);
      let reads = 0, lengths = 0, activated = false, after = 0;
      child.onAfterSearch(() => after++);
      const activate = () => { activated = true; child.activate(other); };
      const listener = child.onBeforeSearch(() => { if (phase === 'before') activate(); });
      const request = {get decorations() {
        reads++;
        const at = {eligibility: 1, highlightEligibility: 2, highlightArgument: 3, selection: 2, throwing: 1}[phase];
        if (reads === at) { activate(); if (phase === 'throwing') throw sentinel; }
        return phase.startsWith('highlight') || phase.startsWith('length') ? {} : undefined;
      }};
      const raw = phase.startsWith('length') ? {get length() {
        if (++lengths === Number(phase.slice(-1))) activate();
        return 0;
      }} : query;
      let found, caught, failed = false;
      try { found = child[method](raw, request); } catch (error) { failed = true; caught = error; }
      check('reactivation reached ' + phase, activated, true);
      check('failure cutoff ' + phase, failed, phase === 'throwing');
      if (failed) {
        check('thrown identity', Object.is(caught, sentinel), true);
        check('old selection before failure', term.getSelectionPosition().start.x, 0);
        check('new selection before failure', other.getSelectionPosition().start.x, 4);
        check('no after-search on failure', after, 0);
      } else {
        const invalid = raw !== query || query === '';
        check('found ' + phase, found, !invalid && query === 'foo');
        if (phase === 'selection') {
          check('already captured engine host is cleared', term.hasSelection(), false);
          check('late selected host', other.hasSelection(), query === 'foo');
          if (query === 'foo') check('late selection keeps old engine position', other.getSelectionPosition().start.x,
            method === 'findNext' ? 4 : 8);
        } else {
          check('old host selection is untouched', term.getSelectionPosition().start.x, 0);
          check('current host selected', other.hasSelection(), !invalid && query === 'foo');
          if (!invalid && query === 'foo') check('current selection determines match', other.getSelectionPosition().start.x,
            phase.startsWith('highlight') ? 4 : method === 'findNext' ? 8 : 0);
        }
        if (phase.startsWith('length')) check('invalid validation phases', lengths, 2);
        check('after-search fires once', after, 1);
      }
      listener.dispose(); cases++;
    } finally { child.dispose(); other.dispose(); parent.remove(); }
  }
  for (const method of ['findNext', 'findPrevious']) {
    for (const phase of ['before', 'eligibility', 'highlightEligibility', 'highlightArgument']) {
      for (const query of ['foo', 'absent']) await scenario(method, phase, query);
    }
    for (const phase of ['before', 'eligibility']) await scenario(method, phase, '');
    for (const phase of ['length1', 'length2']) await scenario(method, phase, '');
    for (const sentinel of [{host: true}, new Error('host'), undefined, null, 0, Symbol('host')]) {
      await scenario(method, 'throwing', 'foo', sentinel);
    }
    for (const query of ['foo', 'absent']) await scenario(method, 'selection', query);
  }
  check('live host cases', cases, 40);
  return cases;
}
