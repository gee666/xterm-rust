// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
// Shared unchanged-source/actual-WASM error-phase controls. This does not assert
// orphan listener/timer liveness after successful registration, tracked in 527.
async function checkSearchCallArgs(term, addon, write, equal) {
  await write('a a a');
  let cases = 0;
  for (const reverse of [false, true]) {
    for (const warm of [false, true]) {
      for (const dispose of [false, true]) {
        for (const mode of ['throw', 'return', 'empty']) {
          const sentinels = mode === 'empty' ? [null] : [Object.freeze({timer: true}), null, Symbol('timer')];
          for (const sentinel of sentinels) {
            const parent = document.createElement('div'); document.body.appendChild(parent);
            const other = new term.constructor({cols: 20, rows: 3, allowProposedApi: true});
            const child = new addon.constructor(); term.loadAddon(child);
            const original = window.setTimeout, clear = window.clearTimeout;
            const ids = [], trace = [];
            let reads = 0, caught, failed = false;
            const matching = Object.freeze({matching: true});
            try {
              other.open(parent);
              await new Promise(resolve => other.write('    a a', resolve));
              if (warm) child.findNext('a');
              term.select(0, 0, 1); other.select(4, 0, 1);
              const query = {get length() {
                if (++reads === 2) {
                  trace.push('guard');
                  if (dispose) child.dispose(); else child.activate(other);
                  if (mode === 'empty') return 0;
                }
                return 1;
              }};
              const request = {get regex() { trace.push('matching'); throw matching; }};
              window.setTimeout = function(callback, delay, ...args) {
                if (delay === 15000) {
                  trace.push('timeout');
                  if (mode === 'throw') throw sentinel;
                  const id = Reflect.apply(original, this, [callback, delay, ...args]);
                  ids.push(id);
                  return id;
                }
                return Reflect.apply(original, this, [callback, delay, ...args]);
              };
              let result;
              try { result = child[reverse ? 'findPrevious' : 'findNext'](query, request); }
              catch (error) { failed = true; caught = error; }
              equal(reads, 2);
              equal(failed, mode !== 'empty');
              if (mode === 'empty') {
                equal(result, false); equal(trace, ['guard']);
              } else {
                equal(Object.is(caught, mode === 'throw' ? sentinel : matching), true);
                equal(trace, mode === 'throw' ? ['guard', 'timeout'] : ['guard', 'timeout', 'matching']);
                equal(term.hasSelection(), false);
                if (!dispose) equal(other.getSelectionPosition(), {start: {x: 4, y: 0}, end: {x: 5, y: 0}});
              }
              cases++;
            } catch (error) {
              throw Error(JSON.stringify({reverse, warm, dispose, mode}) + ': ' + error.message);
            } finally {
              window.setTimeout = original;
              for (const id of ids) Reflect.apply(clear, window, [id]);
              child.dispose(); other.dispose(); parent.remove();
            }
          }
        }
      }
    }
  }
  equal(cases, 56);
  return cases;
}
