// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
async function checkSearchCallArgs(term, addon, write, equal) {
  let cases = 0;
  for (const method of ['findNext', 'findPrevious']) {
    for (const reactivate of [false, true]) {
      for (const guardThrows of [false, true]) {
        term.resize(20, 3);
        await write('\x1bcfoo foo');
        const child = new addon.constructor();
        term.loadAddon(child);
        const other = new term.constructor({cols: 20, rows: 3, allowProposedApi: true});
        const parent = document.createElement('div'); document.body.appendChild(parent);
        let lengths = 0, flags = 0, cutoff, after = 0, results = 0;
        const sentinel = {boundsGuard: true};
        try {
          other.open(parent);
          child.onAfterSearch(() => after++);
          child.onDidChangeResults(() => results++);
          // Source reads length for addon validation, engine validation and
          // first match sizing. The next read is the second engine guard.
          const query = {
            get length() {
              lengths++;
              if (lengths === 4) {
                if (reactivate) child.activate(other);
                term.resize(2, 3);
                cutoff = flags;
                if (guardThrows) throw sentinel;
              }
              return 3;
            },
            toLowerCase() { return 'foo'; }
          };
          const options = {
            decorations: {},
            get regex() {
              flags++;
              if (cutoff !== undefined) throw Error('flags read after invalid column');
              return false;
            }
          };
          let caught, failed = false;
          try { child[method](query, options); }
          catch (error) { caught = error; failed = true; }
          equal(failed, true);
          try { equal(lengths, 4); }
          catch (error) { throw Error(error.message + '; earlier error: ' + String(caught)); }
          equal(flags, cutoff);
          equal(after, 0);
          equal(results, 0);
          if (guardThrows) equal(Object.is(caught, sentinel), true);
          else equal(caught.message, 'Invalid col: 3 to search in terminal of 2 cols');
          cases++;
        } finally { child.dispose(); other.dispose(); parent.remove(); }
      }
    }
  }
  equal(cases, 8);
  return cases;
}
