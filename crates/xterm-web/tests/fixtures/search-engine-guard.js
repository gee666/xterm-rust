// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
async function checkSearchCallArgs(term, addon, write, equal) {
  await write('foo foo foo');
  let cases = 0;
  const sentinels = [{guard: true}, new Error('guard'), undefined, null, 0, Symbol('guard')];
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
      child.onAfterSearch(() => after++);
      const options = {
        get decorations() {
          decorations++;
          if (kind === 'lateDecoration' && decorations === 2) throw value;
          return undefined;
        },
        get regex() { flags++; return false; }
      };
      const internal = {get noScroll() {
        scrolls++; if (kind === 'lateScroll') throw value; return true;
      }};
      const query = kind === 'primitive' ? value : {get length() {
        lengths++;
        if (lengths === 1) return kind === 'invalid' ? value : 1;
        check('engine guard follows addon check', lengths, 2);
        if (mode === 'activate') child.activate(other);
        if (mode === 'nested') child.findNext('foo', {decorations: {}});
        if (kind === 'throw') throw value;
        if (kind === 'strictObject') return {[Symbol.toPrimitive]() { coerces++; throw value; }};
        return kind.startsWith('late') ? 0 : value;
      }};
      let found, caught, failed = false;
      try { found = child[method](query, options, internal); }
      catch (error) { failed = true; caught = error; }
      const strict = kind === 'strict' || kind === 'strictObject';
      const throws = kind === 'throw' || strict || kind.startsWith('late');
      check(kind + ' failure', failed, throws);
      if (throws) {
        if (strict) check('nonzero strict length reaches matching', caught instanceof TypeError, true);
        else check('thrown identity', Object.is(caught, value), true);
        check('after cutoff', after, 0);
      } else {
        check('found', found, kind === 'primitive' && value === 'foo');
        check('after completion', after, mode === 'nested' ? 2 : 1);
      }
      if (kind !== 'primitive') check('raw length reads', lengths, kind === 'invalid' ? 1 : 2);
      check('strict engine guard never coerces length', coerces, 0);
      if (kind === 'throw') {
        check('old selection before guard failure', term.getSelectionPosition().start.x, 0);
        check('other selection before guard failure', other.getSelectionPosition().start.x, 4);
        check('no engine options after guard throw', flags, 0);
        check('no selection arguments after guard throw', decorations, 1);
        check('no scroll arguments after guard throw', scrolls, 0);
      } else if (kind !== 'primitive') {
        check('old engine host selection cleared', term.hasSelection(), false);
        check('new host cleared only by later selection phase', other.hasSelection(), mode !== 'activate');
        if (kind === 'invalid') {
          check('addon invalid skips selection arguments', decorations, 2);
          check('addon invalid skips scroll', scrolls, 0);
        } else if (strict) {
          check('strict nonzero enters options', flags > 0, true);
          check('matching failure skips selection arguments', decorations, 1);
          check('matching failure skips scroll', scrolls, 0);
        } else {
          check('empty engine skips matching flags', flags, 0);
          check('empty engine selection arguments', decorations, kind === 'lateDecoration' || kind === 'lateScroll' ? 2 : 3);
          check('empty engine scroll argument', scrolls, kind === 'lateDecoration' ? 0 : 1);
        }
      }
      cases++;
    } finally { child.dispose(); other.dispose(); parent.remove(); }
  }
  for (const method of ['findNext', 'findPrevious']) {
    for (const zero of [0, -0]) for (const mode of ['plain', 'activate', 'nested']) {
      await scenario(method, 'empty', zero, mode);
    }
    for (const sentinel of sentinels) {
      for (const mode of ['plain', 'activate']) await scenario(method, 'throw', sentinel, mode);
      await scenario(method, 'lateDecoration', sentinel);
      await scenario(method, 'lateScroll', sentinel);
    }
    for (const length of [undefined, null, false, true, '0', 0n, NaN, -1, Infinity, Symbol('length'), {}]) {
      await scenario(method, 'strict', length);
    }
    await scenario(method, 'strictObject', sentinels[0]);
    for (const length of [0, -1]) await scenario(method, 'invalid', length);
    for (const query of ['foo', 'absent']) await scenario(method, 'primitive', query);
  }
  check('engine guard cases', cases, 92);
  return cases;
}
