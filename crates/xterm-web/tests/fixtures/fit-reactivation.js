// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Shared source and Rust assertions. The source oracle uses DOM host fixtures;
// WASM tests supply two real Rust Terminals with measured browser elements.
function fitReactivationCases(createAddon, createPair, host) {
  const results = [];
  function assert(condition, message) { if (!condition) throw Error(message); }
  for (const mode of ['control', 'reactivate', 'reactivate-dispose', 'measurement-throws']) {
    const [first, second] = createPair();
    const addon = createAddon();
    const original = host.getComputedStyle;
    const firstBefore = [first.cols, first.rows];
    const secondBefore = [second.cols, second.rows];
    const thrown = { measurement: mode };
    let callbacks = 0;
    try {
      addon.activate(first);
      const expected = addon.proposeDimensions();
      assert(expected && Number.isFinite(expected.cols) && Number.isFinite(expected.rows), 'missing measurement');
      assert(expected.cols !== first.cols || expected.rows !== first.rows, 'first would not visibly resize');
      assert(expected.cols !== second.cols || expected.rows !== second.rows, 'second would not visibly resize');
      host.getComputedStyle = function(element, ...args) {
        if (element === first.element) {
          callbacks++;
          if (mode !== 'control') addon.activate(second);
          if (mode === 'reactivate-dispose') addon.dispose();
          if (mode === 'measurement-throws') throw thrown;
        }
        return Reflect.apply(original, this, [element, ...args]);
      };
      let caught;
      try { addon.fit(); } catch (error) { caught = error; }
      assert(callbacks === 1, 'element style callback count changed');
      if (mode === 'measurement-throws') {
        assert(caught === thrown, 'measurement exception identity changed');
        assert(first.cols === firstBefore[0] && first.rows === firstBefore[1], 'first resized after exception');
        assert(second.cols === secondBefore[0] && second.rows === secondBefore[1], 'second resized after exception');
      } else {
        assert(caught === undefined, 'unexpected fit exception');
        const target = mode === 'control' ? first : second;
        const untouched = mode === 'control' ? second : first;
        const before = mode === 'control' ? secondBefore : firstBefore;
        assert(target.cols === expected.cols && target.rows === expected.rows, `${mode}: latest terminal not resized`);
        assert(untouched.cols === before[0] && untouched.rows === before[1], `${mode}: old terminal resized`);
      }
      host.getComputedStyle = original;
      // Measurement errors retain the reactivation, and dispose is a no-op.
      const current = mode === 'control' ? first : second;
      const next = addon.proposeDimensions();
      addon.fit();
      assert(current.cols === next.cols && current.rows === next.rows, 'reactivation lost after fit');
      results.push({ mode, styleCallbacks: callbacks, resized: mode === 'measurement-throws' ? 'neither' : mode === 'control' ? 'first' : 'second', retainedActivation: true });
    } finally {
      host.getComputedStyle = original;
      addon.dispose();
      first.dispose();
      second.dispose();
    }
  }
  return results;
}
