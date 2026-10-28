// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
// Execute the unchanged Gesture class with controlled DOM, clock and RAF hosts.
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
import vm from 'node:vm';
const require = createRequire(import.meta.url);
const esbuild = require('../source_repo/node_modules/esbuild');
const build = esbuild.buildSync({ entryPoints: ['source_repo/src/browser/scrollable/touch.ts'],
  bundle: true, write: false, format: 'cjs', platform: 'node', metafile: true,
  tsconfigRaw: { compilerOptions: { experimentalDecorators: true, useDefineForClassFields: false } } });
let clock = 0;
let clockReads = [];
let raf = [];
const events = [];
const hostErrors = [];
const hostWarnings = [];
class Clock {
  static now() { return clockReads.length ? clockReads.shift() : clock; }
  getTime() { return clock; }
}
class HostNode {
  constructor(parent = null) { this.parentElement = parent; this.listeners = new Map(); }
  contains(node) { for (; node; node = node.parentElement) if (node === this) return true; return false; }
  dispatchEvent(event) {
    events.push({ ...event });
    for (const callback of [...(this.listeners.get(event.type) ?? [])]) callback(event);
    return true;
  }
  addEventListener(type, callback) {
    const entries = this.listeners.get(type) ?? [];
    if (!entries.includes(callback)) entries.push(callback);
    this.listeners.set(type, entries);
  }
  removeEventListener(type, callback) {
    const entries = this.listeners.get(type);
    const index = entries?.indexOf(callback) ?? -1;
    if (index >= 0) entries.splice(index, 1);
  }
}
const document = new HostNode();
document.createEvent = () => ({ initEvent(type, bubbles, cancelable) { Object.assign(this, { type, bubbles, cancelable }); } });
const window = { document, ontouchstart: null,
  requestAnimationFrame(callback) { raf.push(callback); return raf.length; } };
const exports = {};
const context = vm.createContext({ exports, module: { exports }, window, document,
  navigator: { maxTouchPoints: 1 }, Node: HostNode, Date: Clock,
  console: { warn(...args) { hostWarnings.push(args.map(String).join(' ')); }, error(error) { hostErrors.push(String(error)); } } });
vm.runInContext(build.outputFiles[0].text, context, { filename: 'unchanged-touch-bundle.cjs' });
const { Gesture } = context.module.exports;
assert.equal(typeof Gesture, 'function');
const n = value => Number.isNaN(value) ? 'NaN' : value === Infinity ? 'Infinity'
  : value === -Infinity ? '-Infinity' : Object.is(value, -0) ? '-0' : String(value);
const numbers = values => values.map(n).join(',');
const list = touch => ({ length: 1, item() { return touch; } });
const event = touch => ({ targetTouches: list(touch), changedTouches: list(touch), preventDefault() {}, stopPropagation() {} });
const traces = [];
let frames = 0;
let moves = 0;
function trace(start, operations) {
  clock = start[0]; clockReads = []; events.length = 0;
  const root = new HostNode();
  const target = new HostNode(root);
  const gesture = new Gesture();
  gesture._targets.push(root);
  const touch = { identifier: 1, target, pageX: start[1], pageY: start[2], clientX: start[1], clientY: start[2] };
  gesture._handleTouchStart(event(touch));
  const record = gesture._activeTouches[1];
  const snapshots = [];
  const encoded = [];
  const last = () => [record.rollingPageX.at(-1), record.rollingPageY.at(-1)];
  for (const operation of operations) {
    events.length = 0;
    const [kind, ...args] = operation;
    encoded.push(kind + (args.length ? ':' + numbers(args) : ''));
    switch (kind) {
      case 'm': {
        clock = args[0];
        Object.assign(touch, { pageX: args[1], pageY: args[2], clientX: args[1], clientY: args[2] });
        gesture._handleTouchMove(event(touch));
        const change = events.find(event => event.type === '-xterm-gesturechange');
        assert(change);
        snapshots.push('M:' + numbers([change.translationX, change.translationY, ...last()]));
        moves++;
        break;
      }
      case 'e': {
        clock = args[0]; clockReads = [args[1], args[0]];
        gesture._handleTouchEnd(window, event(touch));
        assert.equal(clockReads.length, 0);
        const tap = events.find(event => event.type === '-xterm-gesturetap');
        const menu = events.find(event => event.type === '-xterm-gesturecontextmenu');
        const classification = tap ? 'T' : menu ? 'C' : 'I';
        snapshots.push('E:' + classification + ':' + numbers(last()) + ':' + (tap?.tapCount ?? 0));
        break;
      }
      case 'f': {
        clock = args[0];
        const callback = raf.shift();
        if (callback) callback(clock);
        const change = events.find(event => event.type === '-xterm-gesturechange');
        snapshots.push(change ? 'F:' + numbers([change.translationX, change.translationY]) + ':' + Number(raf.length > 0) : 'F:none');
        frames++;
        break;
      }
      case 't': {
        clock = args[0];
        const tap = gesture._newGestureEvent('-xterm-gesturetap', target);
        gesture._dispatchEvent(tap);
        snapshots.push('T:' + tap.tapCount);
        break;
      }
      case 'r': {
        gesture._dispatchEvent(gesture._newGestureEvent('-xterm-gesturechange', target));
        snapshots.push('R');
        break;
      }
      default: throw new Error(`unrecognized operation ${kind}`);
    }
  }
  gesture.dispose();
  // Source cancellation leaves a host RAF registration but marks its work item
  // cancelled. Flush only that cancelled host turn, never spin an animation.
  const cancelled = raf; raf = [];
  for (const callback of cancelled) callback(clock);
  assert.equal(raf.length, 0, 'disposed recognizer rescheduled a frame');
  assert.deepEqual(hostErrors, []);
  assert.deepEqual(hostWarnings, []);
  for (const listeners of document.listeners.values()) assert.equal(listeners.length, 0, 'disposed source listeners retained');
  traces.push([numbers(start), encoded.join('|'), snapshots.join('|')].join('\t'));
}
const starts = [[0, 0, 0], [1000, 10, 20], [0, -0, 0]];
for (const start of starts) {
  for (const hold of [0, 699.999, 700, 700.001, -1, NaN]) {
    for (const [x, y] of [[0, 0], [29.999, 29.999], [30, 0], [0, -30], [-29, -29]]) {
      trace(start, [['m', start[0] + 10, start[1] + x, start[2] + y], ['e', start[0] + hold, start[0] + 20], ['f', start[0] + 36], ['f', start[0] + 52], ['t', 400], ['r'], ['t', 400]]);
    }
  }
}
for (const deltaTime of [0, -1, 1, 16, NaN, Infinity, -Infinity]) {
  for (const point of [[35, 0], [-35, 35], [Infinity, 30], [NaN, 35]]) {
    trace([0, 0, 0], [['m', deltaTime, ...point], ['e', 20, 20], ['f', 20], ['f', 36], ['f', Infinity]]);
  }
}
trace([0, 0, 0], [['m', 1, 10, 0], ['m', 2, 20, 0], ['m', 3, 30, 0], ['m', 4, 40, 0], ['m', 5, 55, 0], ['e', 10, 10], ['f', 26], ['f', 42]]);
trace([0, 0, 0], [['t', 0], ['t', 400], ['t', 801], ['r'], ['t', 400], ['t', NaN], ['t', Infinity], ['t', -Infinity]]);
trace([0, 0, 0], [['m', 1, 0, 35], ['e', 10, 10], ['f', NaN], ['f', 30]]);
trace([0, 0, 0], [['m', 1, 0, -35], ['e', 10, 10], ['f', 10], ['f', 26], ['f', -1]]);
for (const start of [[NaN, -0, Infinity], [Infinity, 1, 2], [-Infinity, NaN, -Infinity],
  [9007199254740991, 9007199254740991, -9007199254740991], [0.125, 0.25, -0.5]]) {
  trace(start, [['m', start[0], -0, 35.5], ['e', 700, 701], ['f', 701], ['f', 717], ['t', 400.125]]);
}
const directed = traces.length;
const seed = 0x704c4b31;
let state = seed;
function random() { state ^= state << 13; state ^= state >>> 17; state ^= state << 5; return state >>> 0; }
for (let index = 0; index < 512; index++) {
  const start = [1000 + random() % 1000, (random() % 400 - 200) / 4, (random() % 400 - 200) / 4];
  const operations = [];
  let time = start[0]; let x = start[1]; let y = start[2];
  for (let i = 0, count = random() % 9; i < count; i++) {
    time += [0, 1, 16, -16, 100][random() % 5];
    x += (random() % 400 - 200) / 4; y += (random() % 400 - 200) / 4;
    operations.push(['m', time, x, y]);
  }
  operations.push(['e', start[0] + [0, 699, 700, 701, -1][random() % 5], time]);
  for (let frame = 0; frame < 6; frame++) operations.push(['f', time += [0, 1, 16, -1][random() % 4]]);
  operations.push(['t', time], ['r'], ['t', time + 400]);
  trace(start, operations);
}
const fixture = 'crates/xterm-web/tests/fixtures/touch-policy.tsv';
const corpus = traces.join('\n') + '\n';
if (process.argv.includes('--generate')) {
  await mkdir('crates/xterm-web/tests/fixtures', { recursive: true });
  await writeFile(fixture, corpus);
} else assert.equal((await readFile(fixture, 'utf8')).replace(/\r\n/g, '\n'), corpus, 'source touch policy fixture drift');
const sources = await Promise.all(Object.keys(build.metafile.inputs).sort().map(async path => ({ path,
  sha256: createHash('sha256').update(await readFile(path)).digest('hex') })));
const summary = { node: process.version, platform: process.platform, esbuild: esbuild.version, sources,
  originalTouchSpecificSuitesFound: 0, directedCases: directed, randomCases: 512, seed,
  traces: traces.length, movementCalls: moves, frameAttempts: frames,
  operationSnapshots: traces.reduce((count, line) => count + line.split('\t')[1].split('|').length, 0),
  fixtureSha256: createHash('sha256').update(corpus).digest('hex'), fixtureBytes: Buffer.byteLength(corpus),
  hostSubstitutions: ['DOM Node containment and CustomEvent records', 'controlled Date.now/new Date.getTime clock', 'queued one-turn requestAnimationFrame', 'direct private method calls instead of physical touch hardware'],
  scope: 'Portable four-sample, clock/end, inertia and tap policy only. No claim of native browser dispatch/selection/scroll integration.',
  status: process.argv.includes('--generate') ? 'generated' : 'matched' };
await mkdir('results/touch-policy', { recursive: true });
await writeFile('results/touch-policy/oracle.json', JSON.stringify(summary, null, 2) + '\n');
console.log(JSON.stringify(summary, null, 2));
