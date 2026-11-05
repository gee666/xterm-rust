// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
// Identical assertions run on unchanged source and actual Rust bindings.
async function checkSearchCallArgs(term, addon, write, equal) {
  const decorations = {matchOverviewRuler: '#ff0000', activeMatchColorOverviewRuler: '#00ff00'};
  const nativeSet = window.setTimeout, nativeClear = window.clearTimeout;
  const pending = new Map();
  let nextId = -1, cases = 0;
  // Drive only the addon's 200ms debounce. Other browser/application timers
  // retain their real scheduling, and no wall-clock wait is needed.
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
  async function refreshed(options, action, searchTerm = 'foo', resize = false) {
    const child = new addon.constructor();
    term.loadAddon(child); term.clearSelection();
    try {
      check('initial match', child.findNext(searchTerm, options), true);
      if (resize) term.resize(term.cols + 1, term.rows);
      else await write('\x1b[0m');
      check('one debounce callback', pending.size, 1);
      const [id, callback] = pending.entries().next().value;
      pending.delete(id);
      action(child, callback);
      cases++;
    } finally { child.dispose(); }
    check('child disposed all debounce callbacks', pending.size, 0);
  }
  try {
    await write('foo FOO foo');
    for (const resize of [false, true]) {
      const log = [], symbol = Symbol('extra');
      const options = Object.create({get inherited() { throw Error('inherited getter copied'); }});
      options.decorations = decorations;
      for (const key of ['2', '1', 'extra', 'incremental', symbol]) {
        Object.defineProperty(options, key, {enumerable: true, get() { log.push(String(key)); return false; }});
      }
      Object.defineProperty(options, 'hidden', {get() { throw Error('non-enumerable getter copied'); }});
      await refreshed(options, (child, fire) => {
        child.onBeforeSearch(() => log.push('before'));
        child.onAfterSearch(() => log.push('after'));
        fire();
        check('spread order before search events', log, ['1', '2', 'extra', 'incremental', 'Symbol(extra)', 'before', 'after']);
        check('selection survives refresh', term.getSelection().toLowerCase(), 'foo');
      }, 'foo', resize);
    }
    {
      const log = [];
      const options = {decorations, get early() {
        log.push('early'); delete this.late;
        Object.defineProperty(this, 'added', {enumerable: true, get() { throw Error('new key copied'); }});
        return true;
      }, get late() { throw Error('deleted key copied'); }};
      await refreshed(options, (_, fire) => { fire(); check('spread snapshots keys but reads live descriptors', log, ['early']); });
    }
    for (const inherited of [false, true]) {
      const options = inherited ? Object.create({caseSensitive: true}) : {};
      options.decorations = decorations;
      if (!inherited) Object.defineProperty(options, 'caseSensitive', {value: true});
      await refreshed(options, (child, fire) => {
        let count;
        child.onDidChangeResults(result => count = result.resultCount);
        fire();
        check('refresh drops inherited or non-enumerable flags', count, 3);
      }, 'FOO');
    }
    {
      const log = [], symbol = Symbol('key');
      let armed = false;
      const label = key => typeof key === 'symbol' ? 'symbol' : key;
      const target = {decorations, caseSensitive: false, extra: 1, [symbol]: 2};
      const options = new Proxy(target, {
        ownKeys(object) { if (armed) log.push('keys'); return Reflect.ownKeys(object); },
        getOwnPropertyDescriptor(object, key) { if (armed) log.push('descriptor:' + label(key)); return Reflect.getOwnPropertyDescriptor(object, key); },
        get(object, key, receiver) { if (armed) log.push('get:' + label(key)); return Reflect.get(object, key, receiver); }
      });
      await refreshed(options, (_, fire) => {
        armed = true; fire();
        check('proxy CopyDataProperties trap order', log, ['keys', 'descriptor:decorations', 'get:decorations',
          'descriptor:caseSensitive', 'get:caseSensitive', 'descriptor:extra', 'get:extra', 'descriptor:symbol', 'get:symbol']);
      });
    }
    for (const failure of ['getter', 'symbol', 'ownKeys', 'descriptor', 'proxyGet']) {
      const sentinel = {failure};
      const symbol = Symbol('throws');
      let armed = false;
      const target = {decorations};
      if (failure === 'getter' || failure === 'symbol') {
        Object.defineProperty(target, failure === 'getter' ? 'extra' : symbol, {
          enumerable: true, get() { if (armed) throw sentinel; return false; }
        });
      }
      const options = new Proxy(target, {
        ownKeys(object) { if (armed && failure === 'ownKeys') throw sentinel; return Reflect.ownKeys(object); },
        getOwnPropertyDescriptor(object, key) { if (armed && failure === 'descriptor') throw sentinel; return Reflect.getOwnPropertyDescriptor(object, key); },
        get(object, key, receiver) { if (armed && failure === 'proxyGet') throw sentinel; return Reflect.get(object, key, receiver); }
      });
      await refreshed(options, (child, fire) => {
        let before = 0, after = 0, caught = false;
        child.onBeforeSearch(() => before++); child.onAfterSearch(() => after++);
        armed = true;
        try { fire(); } catch (error) { if (error !== sentinel) throw Error('refresh exception identity changed'); caught = true; }
        armed = false;
        check('spread exception propagated ' + failure, caught, true);
        check('spread error precedes search events', [before, after], [0, 0]);
        check('search after failed refresh', child.findNext('foo'), true);
        check('cache cleared before failed spread', term.getSelectionPosition().start.x, 0);
      });
    }
    {
      let child, reads = 0;
      const options = {decorations, get extra() {
        reads++;
        check('reentrant search from spread getter', child.findNext('foo'), true);
        check('cache cleared before reentry', term.getSelectionPosition().start.x, 0);
        return 1;
      }};
      await refreshed(options, (value, fire) => { child = value; fire(); check('extra getter invoked once', reads, 1); });
    }
    {
      let reads = 0;
      const options = {decorations, get extra() { reads++; return 1; }};
      await refreshed(options, (child, fire) => {
        fire();
        // The refresh stores the copy. A later resize must not re-read the
        // original getter, even though it creates another refresh request.
        term.resize(term.cols + 1, term.rows);
        check('second refresh scheduled', pending.size, 1);
        const [id, second] = pending.entries().next().value;
        pending.delete(id); second();
        check('copied values replace original getters', reads, 1);
      });
    }
    {
      let child;
      const options = {decorations, get extra() { child.dispose(); return 1; }};
      await refreshed(options, (value, fire) => { child = value; fire(); });
    }
    check('all directed refresh cases', cases, 14);
    return cases;
  } finally {
    window.setTimeout = nativeSet; window.clearTimeout = nativeClear;
  }
}
