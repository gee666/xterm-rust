// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
// Identical assertions on unchanged source classes and the actual Rust bindings.
async function checkSearchCallArgs(term, addon, write, equal) {
  const ttl = 15000;
  const nativeSet = window.setTimeout, nativeClear = window.clearTimeout;
  const nativeDateNow = Date.now;
  const performanceDescriptor = Object.getOwnPropertyDescriptor(performance, 'now');
  const pending = new Map();
  let now = 0, nextId = -1, cases = 0;
  window.setTimeout = function(callback, delay, ...args) {
    // Only the known cache TTL and its one-millisecond-short extension are
    // virtual. Parsing, rendering and unrelated browser work remain native.
    if (delay !== ttl && delay !== ttl - 1) return nativeSet.call(this, callback, delay, ...args);
    const id = nextId--;
    pending.set(id, {due: now + delay, callback: () => callback(...args)});
    return id;
  };
  window.clearTimeout = function(id) {
    if (!pending.delete(id)) nativeClear.call(this, id);
  };
  Date.now = () => now;
  Object.defineProperty(performance, 'now', {configurable: true, value: () => now});
  const fire = () => {
    const job = [...pending].filter(([, value]) => value.due <= now)
      .sort((a, b) => a[1].due - b[1].due)[0];
    if (!job) return false;
    pending.delete(job[0]); job[1].callback(); return true;
  };
  try {
    for (const method of ['findNext', 'findPrevious']) {
      for (const delayed of [ttl - 1, ttl, ttl * 3]) {
        now = 0;
        await write('\x1bcfoo\r');
        const child = new addon.constructor();
        term.loadAddon(child);
        const find = () => { term.clearSelection(); return child[method]('foo'); };
        try {
          equal(find(), true);
          equal(pending.size, 1);
          await write('\x1b[3X');
          equal(pending.size, 1);
          equal(term.buffer.active.getLine(0).translateToString(true), '');
          now = delayed;
          equal(find(), true);
          equal(pending.size, 1);
          now = Math.max(delayed, ttl);
          equal(fire(), true);
          equal(pending.size, 1);
          equal([...pending.values()].map(job => job.due), [delayed + ttl]);
          now = delayed + ttl - 1;
          equal(fire(), false);
          now = delayed + ttl;
          equal(fire(), true);
          equal(pending.size, 0);
          equal(find(), false);
          equal(pending.size, 1);
          cases++;
        } catch (error) {
          throw Error(method + ' delayed=' + delayed + ': ' + error.message);
        } finally { child.dispose(); }
        equal(pending.size, 0);
      }
    }
    return cases;
  } finally {
    for (const id of pending.keys()) window.clearTimeout(id);
    window.setTimeout = nativeSet;
    window.clearTimeout = nativeClear;
    Date.now = nativeDateNow;
    if (performanceDescriptor) Object.defineProperty(performance, 'now', performanceDescriptor);
    else delete performance.now;
  }
}
