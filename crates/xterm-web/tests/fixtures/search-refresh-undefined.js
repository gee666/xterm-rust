// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
// The same assertions run against unchanged source and actual Rust bindings.
async function checkSearchCallArgs(term, addon, write, equal) {
  const decorations = {matchOverviewRuler: '#ff0000', activeMatchColorOverviewRuler: '#00ff00'};
  const nativeSet = window.setTimeout, nativeClear = window.clearTimeout;
  const pending = new Map();
  let nextId = -1, cases = 0;
  window.setTimeout = function(callback, delay, ...args) {
    if (delay !== 200) return nativeSet.call(this, callback, delay, ...args);
    const id = nextId--; pending.set(id, () => callback(...args)); return id;
  };
  window.clearTimeout = function(id) {
    if (!pending.delete(id)) nativeClear.call(this, id);
  };
  const check = (label, actual, expected) => {
    try { equal(actual, expected); } catch (error) { throw Error(label + ': ' + error.message); }
  };
  function fire() {
    check('one pending refresh', pending.size, 1);
    const [id, callback] = pending.entries().next().value;
    pending.delete(id); callback();
  }
  async function scheduled(resize, action) {
    const child = new addon.constructor();
    term.loadAddon(child); term.clearSelection();
    try {
      let copies = 0, nested;
      const options = {decorations, get extra() { copies++; if (nested) nested(child); return 1; }};
      check('initial search', child.findNext('foo', options), true);
      if (resize) term.resize(term.cols + 1, term.rows);
      else await write('\x1b[0m');
      check('refresh is queued', pending.size, 1);
      action(child, callback => { nested = callback; }, () => copies);
      cases++;
    } finally { child.dispose(); }
    check('disposed child has no pending refresh', pending.size, 0);
  }
  const probe = (child, direction, expectedReads, failure) => {
    const reads = [];
    const options = {decorations};
    for (const key of ['caseSensitive', 'regex', 'wholeWord']) {
      Object.defineProperty(options, key, {get() { reads.push(key); if (failure) throw failure; return false; }});
    }
    let caught;
    try { check('empty search result', child[direction]('', options), false); }
    catch (error) { caught = error; }
    if (failure && expectedReads) check('comparison preserves thrown identity', caught === failure, true);
    else if (caught) throw caught;
    check('cached-term flag comparison', reads, expectedReads
      ? (failure ? ['caseSensitive'] : ['caseSensitive', 'caseSensitive', 'regex', 'regex', 'wholeWord', 'wholeWord']) : []);
  };
  try {
    await write('foo foo');
    const actions = [
      ['clear', child => child.clearDecorations(), false, false],
      ['retain', child => child.clearDecorations(true), true, false],
      ['next undefined', child => child.findNext(undefined), false, false],
      ['previous undefined', child => child.findPrevious(undefined), false, false],
      ['empty', child => child.findNext(''), false, true],
      ['null', child => child.findNext(null), false, false],
      ['false', child => child.findNext(false), false, false],
      ['zero', child => child.findNext(0), false, false]
    ];
    for (const resize of [false, true]) {
      for (const direction of ['findNext', 'findPrevious']) {
        for (const [label, action, selected, flags] of actions) {
          await scheduled(resize, (child, _, copies) => {
            action(child);
            // Cache clearing alone does not clear selection. The refresh must.
            term.select(0, 0, 3);
            const events = [];
            child.onBeforeSearch(() => events.push('before'));
            child.onAfterSearch(() => events.push('after'));
            fire();
            check(label + ' option copy', copies(), label === 'clear' || label === 'retain' ? 1 : 0);
            check(label + ' refresh events', events, ['before', 'after']);
            check(label + ' refreshed selection', term.getSelection(), selected ? 'foo' : '');
            probe(child, direction, flags);
          });
        }
      }
      for (const nested of [false, true]) {
        await scheduled(resize, (child, setNested, copies) => {
          child.clearDecorations();
          if (nested) setNested(value => check('copy reentry', value.findNext('', {decorations}), false));
          fire();
          check('copy after clearing cache', copies(), 1);
          probe(child, 'findNext', false, {mustNotThrow: true});
        });
      }
      await scheduled(resize, child => {
        child.findNext(''); fire();
        probe(child, 'findPrevious', true, {mustThrow: true});
      });
    }
    check('all delayed absence cases', cases, 38);
    return cases;
  } finally {
    window.setTimeout = nativeSet; window.clearTimeout = nativeClear;
  }
}
