// Invoke the render listener installed by unchanged Linkifier._handleNewLink.
import assert from 'node:assert/strict';
import { mkdir, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { sourceLoader } from './source-ts-loader.mjs';
const root = process.env.SOURCE_ROOT ? resolve(process.env.SOURCE_ROOT, '..') : process.cwd();
const { source, load, hashes, typescript } = sourceLoader(root);
const { Linkifier } = load(resolve(source, 'src/browser/Linkifier.ts'));
const cases = [];
for (const mode of ['plain', 'second start outside', 'second end outside', 'throw second start',
  'throw second end', 'first start outside', 'first end outside', 'zero start', 'clear first start', 'clear second start']) {
  const target = Object.create(Linkifier.prototype), reads = [], events = [], marker = {};
  let listener, armed = false, startReads = 0, endReads = 0, requested = false, caught;
  target._lastMouseEvent = {};
  target._element = { classList: {} };
  target._bufferService = { cols: 20, buffer: { ydisp: 0 } };
  target._renderService = { onRenderedViewportChange(callback) { listener = callback; return { dispose() {} }; } };
  target._linkCacheDisposables = [];
  target._positionFromMouseEvent = () => ({ x: 2, y: 2 });
  target._askForLink = () => { requested = true; };
  const point = end => ({ x: end ? 10 : 1, get y() {
    if (!armed) return 2;
    const ordinal = end ? ++endReads : ++startReads;
    reads.push(`${end ? 'end' : 'start'}:${ordinal}`);
    if (mode === `throw second ${end ? 'end' : 'start'}` && ordinal === 2) throw marker;
    if (!end && mode === `clear ${ordinal === 1 ? 'first' : 'second'} start`) target._clearCurrentLink();
    if (mode === `${ordinal === 1 ? 'first' : 'second'} ${end ? 'end' : 'start'} outside`) return end ? 3 : 1;
    return 2;
  } });
  const link = { text: mode, range: { start: point(false), end: point(true) },
    decorations: { underline: false, pointerCursor: false }, hover() {}, leave() { events.push('leave'); } };
  target._handleNewLink({ link });
  armed = true;
  try { listener({ start: mode === 'zero start' ? 0 : 1, end: 1 }); } catch (error) { caught = error; }
  if (mode.startsWith('clear')) assert(caught instanceof TypeError && caught.message.includes('link'));
  else assert.equal(caught, mode.startsWith('throw') ? marker : undefined);
  assert.deepEqual(events, ['plain', 'zero start'].includes(mode) || mode.startsWith('clear') ? ['leave'] : []);
  assert.equal(requested, ['plain', 'zero start', 'second start outside', 'second end outside'].includes(mode));
  assert.deepEqual(reads, mode === 'first start outside' || mode === 'clear first start' ? ['start:1']
    : mode === 'first end outside' || mode === 'zero start' ? ['start:1', 'end:1']
    : ['second start outside', 'throw second start', 'clear second start'].includes(mode) ? ['start:1', 'end:1', 'start:2']
    : ['start:1', 'end:1', 'start:2', 'end:2']);
  cases.push({ mode, reads, events, requested, caughtMarker: caught === marker, caughtTypeError: caught instanceof TypeError });
}
await mkdir('results/link-render-recheck', { recursive: true });
await writeFile('results/link-render-recheck/methods.json', JSON.stringify({ typescript, cases,
  sources: Object.fromEntries(hashes), scope: 'Directed render-range rechecks, not full selection/links parity.' }, null, 2) + '\n');
console.log(`${cases.length} unchanged-source render recheck cases PASS`);
