// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
import fs from 'node:fs';
import path from 'node:path';
import { sourceLoader } from './source-ts-loader.mjs';
const { source, load, hashes, typescript } = sourceLoader();
const { RenderDebouncer } = load(path.join(source, 'src/browser/RenderDebouncer.ts'));
const { TimeBasedDebouncer } = load(path.join(source, 'src/browser/TimeBasedDebouncer.ts'));
const initialSeed = 0x6deb0ace;
let seed = initialSeed;
const next = () => (seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0);
const number = value => Number.isNaN(value) ? 'NaN' : Object.is(value, -0) ? '-0' : String(value);
const optional = value => value === undefined ? '-' : number(value);
const value = () => [undefined, -0, 0, -10, 3.5, 40, NaN][next() % 7];
const records = [];
let now = 0;
let id = 0;
let callbacks = new Map();
let events = [];
const window = {
  requestAnimationFrame(callback) { const handle = ++id; callbacks.set(handle, callback); events.push(`frame:${handle}`); return handle; },
  cancelAnimationFrame(handle) { callbacks.delete(handle); events.push(`cancel:${handle}`); },
  setTimeout(callback, delay) { const handle = ++id; callbacks.set(handle, callback); events.push(`timeout:${number(delay)}:${handle}`); return handle; }
};
const originals = ['window', 'performance', 'clearTimeout'].map(key => [key, Object.getOwnPropertyDescriptor(globalThis, key)]);
Object.defineProperty(globalThis, 'window', { value: window, configurable: true });
Object.defineProperty(globalThis, 'performance', { value: { now: () => now }, configurable: true });
Object.defineProperty(globalThis, 'clearTimeout', { value: handle => { callbacks.delete(handle); events.push(`cancel:${handle}`); }, configurable: true });
try {
  for (const kind of ['frame', 'time']) {
    now = 0; id = 0; callbacks = new Map(); events = [];
    const render = (start, end) => events.push(`render:${number(start)}:${number(end)}`);
    const debouncer = kind === 'frame' ? new RenderDebouncer(render, { window }) : new TimeBasedDebouncer(render, 1000);
    for (let step = 0; step < 4096; step++) {
      now += next() % 1501;
      let op; let start; let end; let count; let tag = '-';
      const choice = next() % 10;
      if (choice < 6) {
        op = 'refresh'; start = value(); end = value(); count = 1 + next() % 80;
        debouncer.refresh(start, end, count);
      } else if (choice === 6 && kind === 'frame') {
        op = 'callback'; tag = String(step);
        debouncer.addRefreshCallback(time => events.push(`callback:${tag}:${number(time)}`));
      } else if (choice === 7) {
        op = 'dispose'; debouncer.dispose();
      } else {
        op = 'dispatch';
        const entry = callbacks.entries().next().value;
        if (entry) { callbacks.delete(entry[0]); entry[1](0); }
      }
      records.push([kind, op, now, optional(start), optional(end), optional(count), tag, events.join(',')].join('\t'));
      events = [];
    }
    debouncer.dispose();
  }
} finally {
  for (const [key, descriptor] of originals) {
    if (descriptor) Object.defineProperty(globalThis, key, descriptor); else delete globalThis[key];
  }
}
fs.mkdirSync('crates/xterm-web/tests/fixtures', { recursive: true });
fs.mkdirSync('results/links', { recursive: true });
fs.writeFileSync('crates/xterm-web/tests/fixtures/debounce.tsv', records.join('\n') + '\n');
const result = { command: 'node scripts/debounce-oracle.mjs', node: process.version, typescript, seed: initialSeed,
  operations: records.length, frameOperations: 4096, timeOperations: 4096,
  upstreamTestFiles: [], originalTestsDiscovered: 0,
  scope: 'unmodified source algorithms with a virtual event loop; not browser runtime evidence', sources: Object.fromEntries(hashes) };
fs.writeFileSync('results/links/debounce-oracle.json', JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify(result, null, 2));
