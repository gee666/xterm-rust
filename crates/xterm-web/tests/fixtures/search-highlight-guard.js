// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
async function checkSearchCallArgs(term, addon, write, equal) {
  await write('foo foo foo');
  let cases = 0;
  const sentinels = [{guard: true}, new Error('highlight guard'), undefined, null, 0, Symbol('guard')];
  const check = (label, actual, expected) => {
    try { equal(actual, expected); } catch (error) { throw Error(label + ': ' + error.message); }
  };
  async function scenario(method, kind, value, mode = 'plain') {
    const child = new addon.constructor(); term.loadAddon(child);
    const other = new term.constructor({cols: 20, rows: 3, allowProposedApi: true});
    const parent = document.createElement('div'); document.body.appendChild(parent);
    try {
      other.open(parent);
      await new Promise(resolve => other.write('foo foo foo', resolve));
      child.findNext('foo', {decorations: {}});
      term.select(0, 0, 3); other.select(4, 0, 3);
      let lengths = 0, decorations = 0, flags = 0, scrolls = 0, after = 0, coerces = 0;
      const results = [];
      child.onAfterSearch(() => after++);
      child.onDidChangeResults(event => results.push([event.resultIndex, event.resultCount]));
      const options = {
        get decorations() {
          decorations++;
          if (decorations === 2 && kind === 'empty') {
            check('captured highlight engine host cleared', term.hasSelection(), false);
            check('new terminal not cleared until addon invalid phase', other.hasSelection(), true);
          }
          if ((kind === 'late2' && decorations === 2) || (kind === 'late3' && decorations === 3)) throw value;
          return {};
        },
        get regex() { flags++; if (kind === 'strict' || kind === 'strictObject') throw value; return false; }
      };
      const internal = {get noScroll() { scrolls++; return true; }};
      const query = kind === 'primitive' ? value : {get length() {
        lengths++;
        if (lengths === 1) return 1;
        if (lengths === 3) return 0;
        check('highlight guard follows addon validity', lengths, 2);
        check('selection preserved before highlight engine guard', term.hasSelection(), true);
        if (mode === 'activate') child.activate(other);
        if (kind === 'throw') throw value;
        if (kind === 'strictObject') return {[Symbol.toPrimitive]() { coerces++; throw value; }};
        return kind.startsWith('late') ? 0 : value;
      }};
      let found, caught, failed = false;
      try { found = child[method](query, options, internal); }
      catch (error) { failed = true; caught = error; }
      const throws = ['throw', 'strict', 'strictObject', 'late2', 'late3'].includes(kind);
      check(kind + ' failure', failed, throws);
      if (throws) {
        check('exact thrown identity', Object.is(caught, value), true);
        check('after cutoff', after, 0);
        check('results cutoff', results, []);
      } else {
        check('found', found, kind === 'primitive' && value === 'foo');
        check('after completion', after, 1);
        if (kind === 'empty') check('empty results', results, [[-1, 0]]);
      }
      check('strict guard never coerces length', coerces, 0);
      if (kind === 'primitive') { cases++; return; }
      check('raw length reads', lengths, kind === 'empty' ? 3 : 2);
      check('matching flags cutoff', flags, kind === 'strict' || kind === 'strictObject' ? 1 : 0);
      check('scroll cutoff', scrolls, 0);
      if (kind === 'throw' || kind === 'strict' || kind === 'strictObject') {
        check('guard or matching error retains old selection', term.hasSelection(), true);
        check('guard or matching error retains other selection', other.hasSelection(), true);
        check('no late decorations after early error', decorations, 1);
      } else {
        check('empty highlight clears old selection', term.hasSelection(), false);
        check('other selection cleared only by later addon validation', other.hasSelection(), kind === 'empty' ? mode !== 'activate' : true);
        check('post-highlight decoration reads', decorations, kind === 'empty' ? 4 : Number(kind.slice(4)));
      }
      cases++;
    } finally { child.dispose(); other.dispose(); parent.remove(); }
  }
  for (const method of ['findNext', 'findPrevious']) {
    for (const zero of [0, -0]) for (const mode of ['plain', 'activate']) await scenario(method, 'empty', zero, mode);
    for (const sentinel of sentinels) {
      for (const mode of ['plain', 'activate']) await scenario(method, 'throw', sentinel, mode);
      for (const phase of ['late2', 'late3']) await scenario(method, phase, sentinel);
    }
    for (const length of [undefined, null, false, true, '0', 0n, NaN, -1, Infinity, Symbol('length'), {}]) await scenario(method, 'strict', length);
    await scenario(method, 'strictObject', sentinels[0]);
    for (const query of ['foo', 'absent']) await scenario(method, 'primitive', query);
  }
  check('highlight guard cases', cases, 84);
  return cases;
}
