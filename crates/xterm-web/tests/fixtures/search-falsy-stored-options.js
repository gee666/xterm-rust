// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
async function checkSearchCallArgs(term, addon, write, equal) {
  await write('foo FOO foo');
  const decorations = {matchOverviewRuler:'#ff0000',activeMatchColorOverviewRuler:'#00ff00'};
  const falsy = [undefined, null, false, 0, -0, NaN, 0n, ''];
  const truthy = [{}, true, 1, 1n, 'x', Object(false)];
  let cases = 0;
  for (const method of ['findNext', 'findPrevious']) {
    for (const nested of ['findNext', 'findPrevious']) {
      for (const replacement of [...falsy, ...truthy]) {
        const child = new addon.constructor(); term.loadAddon(child);
        try {
          equal(child.findNext('foo'), true);
          const events = [], flags = [];
          child.onBeforeSearch(() => events.push('before'));
          child.onAfterSearch(() => events.push('after'));
          const comparison = {comparison:true}, matching = {matching:true};
          let entered = false, caught;
          const request = {
            get decorations() {
              if (!entered) {
                entered = true;
                let nestedError;
                try { equal(child[nested]('foo', replacement), true); }
                catch (error) { nestedError = error; }
                equal(replacement === null ? nestedError instanceof TypeError : nestedError === undefined, true);
              }
              return decorations;
            },
            get caseSensitive() { flags.push('caseSensitive'); throw comparison; },
            get regex() { flags.push('regex'); throw matching; }
          };
          try { child[method]('foo', request, {get noScroll() { throw Error('early noScroll'); }}); }
          catch (error) { caught = error; }
          equal(caught === (replacement ? comparison : matching), true);
          equal(flags, [replacement ? 'caseSensitive' : 'regex']);
          equal(events, replacement === null ? ['before','before'] : ['before','before','after']);
          // The failing outer call must not fire after-search or disturb the
          // nested selection. Null matching fails after clearing selection.
          equal(term.hasSelection(), replacement !== null);
          if (replacement !== null) equal(term.getSelection().toLowerCase(), 'foo');
          cases++;
        } finally { child.dispose(); }
      }
    }
    for (const mode of ['changed', 'absent']) {
      const child = new addon.constructor(); term.loadAddon(child);
      try {
        child.findNext('foo');
        if (mode === 'absent') child.clearDecorations();
        const matching = {}, flags = []; let caught;
        try { child[method](mode === 'changed' ? 'FOO' : 'foo', {
          decorations,
          get caseSensitive() { flags.push('caseSensitive'); throw Error('cache short circuit lost'); },
          get regex() { flags.push('regex'); throw matching; }
        }); } catch (error) { caught = error; }
        equal(caught === matching, true); equal(flags, ['regex']); cases++;
      } finally { child.dispose(); }
    }
    // Primitive requests can inherit decoration eligibility even when the raw
    // stored request is falsy. Restore every descriptor, including on failure.
    for (const replaced of [false, true]) {
    for (const request of [false, 0, -0, NaN, 0n, '']) {
      const prototype = Object.getPrototypeOf(Object(request));
      const keys = ['decorations','caseSensitive','regex'];
      const descriptors = keys.map(key => Object.getOwnPropertyDescriptor(prototype, key));
      const child = new addon.constructor(); term.loadAddon(child);
      const matching = {}, flags = []; let caught, entered = false;
      try {
        child.findNext('foo');
        Object.defineProperties(prototype, {
          decorations: {configurable:true, get() {
            if (replaced && !entered) { entered = true; equal(child.findNext('foo', {}), true); }
            return decorations;
          }},
          caseSensitive: {configurable:true, get() { flags.push('caseSensitive'); throw Error('falsy request compared flags'); }},
          regex: {configurable:true, get() { flags.push('regex'); throw matching; }}
        });
        try { child[method]('foo', request); } catch (error) { caught = error; }
      } finally {
        keys.forEach((key,index) => {
          if (descriptors[index]) Object.defineProperty(prototype,key,descriptors[index]);
          else delete prototype[key];
        });
        child.dispose();
      }
      equal(caught === matching, true); equal(flags, ['regex']);
      equal(term.hasSelection(), !replaced); cases++;
    }
    }
  }
  equal(cases, 84);
  return cases;
}
