// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
// Shared by unchanged source classes and actual Rust/WASM public bindings.
async function checkSearchCallArgs(term, addon, write, equal) {
  await write('a a a');
  let cases = 0;
  const position = col => ({start: {x: col, y: 0}, end: {x: col + 1, y: 0}});
  for (const reverse of [false, true]) {
    const method = reverse ? 'findPrevious' : 'findNext';
    for (const cached of [undefined, 'a', 'b']) {
      for (const nested of ['a', 'b', 'clear', 'throw']) {
        for (const key of ['regex', 'caseSensitive', 'wholeWord']) {
          addon.clearDecorations();
          term.clearSelection();
          if (cached !== undefined) addon.findNext(cached);
          term.select(reverse ? 2 : 0, 0, 1);
          let armed = true;
          const sentinel = Object.freeze({reverse, cached, nested, key});
          const request = {};
          Object.defineProperty(request, key, {get() {
            if (armed) {
              armed = false;
              if (nested === 'clear') addon.clearDecorations();
              else addon.findNext(nested === 'throw' ? 'a' : nested);
              if (nested === 'throw') throw sentinel;
            }
            return false;
          }});
          const events = [];
          const before = addon.onBeforeSearch(() => events.push('before'));
          const after = addon.onAfterSearch(() => events.push('after'));
          try {
            if (nested === 'throw') {
              let thrown;
              try { addon[method]('a', request); } catch (error) { thrown = error; }
              if (thrown !== sentinel) throw Error('engine callback exception identity changed');
              equal(term.getSelectionPosition(), position(0));
              equal(events, ['before', 'before', 'after']);
              equal(addon.findNext('a'), true);
              equal(term.getSelectionPosition(), position(2));
            } else {
              equal(addon[method]('a', request), true);
              const col = reverse !== (cached === 'a') ? 2 : 0;
              equal(term.getSelectionPosition(), position(col));
              equal(events, nested === 'clear' ? ['before', 'after'] : ['before', 'before', 'after', 'after']);
              // Success writes the outer request back after nested completion.
              equal(addon.findNext('a'), true);
              equal(term.getSelectionPosition(), position(col + 2));
            }
            equal(armed, false);
          } finally { before.dispose(); after.dispose(); }
          cases++;
        }
      }
    }
  }
  return cases;
}
