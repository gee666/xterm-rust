// Run unchanged source methods; these directed checks do not prove full parity.
import assert from 'node:assert/strict';
import { mkdir, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { sourceLoader } from './source-ts-loader.mjs';
const root = process.env.SOURCE_ROOT ? resolve(process.env.SOURCE_ROOT, '..') : process.cwd();
const { source, load, hashes, typescript } = sourceLoader(root);
const { Linkifier } = load(resolve(source, 'src/browser/Linkifier.ts'));
const cases = [];
for (const mode of ['plain', 'range', 'start.x', 'start.y', 'end.x', 'end.y', 'coerce start.x', 'coerce start.y', 'coerce end.y', 'throw range', 'throw end.y']) {
  for (const show of [true, false]) {
  const target = Object.create(Linkifier.prototype), reads = [], marker = {};
  let cols = 20, offset = 2, event, caught, fired;
  target._bufferService = {
    get cols() { reads.push(`cols:${cols}`); return cols; },
    buffer: { get ydisp() { reads.push(`ydisp:${offset}`); return offset; } }
  };
  target._onShowLinkUnderline = { fire(value) { event = value; fired = true; } };
  target._onHideLinkUnderline = { fire(value) { event = value; fired = false; } };
  const read = (key, value) => {
    reads.push(key);
    if (mode === `throw ${key}`) throw marker;
    if (mode === key) { cols = 40; offset = 3; }
    if (mode === `coerce ${key}`) return { valueOf() {
      reads.push(`coerce ${key}`); cols = 40; offset = 3; return value;
    } };
    return value;
  };
  const range = { start: { get x() { return read('start.x', 2); }, get y() { return read('start.y', 5); } },
    end: { get x() { return read('end.x', 8); }, get y() { return read('end.y', 6); } } };
  const link = { get range() { read('range', 0); return range; } };
  try { target._fireUnderlineEvent(link, show); } catch (error) { caught = error; }
  if (mode.startsWith('throw')) { assert.equal(caught, marker); assert.equal(event, undefined); }
  else {
    assert.equal(caught, undefined);
    assert.equal(fired, show);
    assert.deepEqual(event, { x1: 1, y1: mode === 'range' ? 1 : 2, x2: 8,
      y2: mode === 'range' ? 2 : 3, cols: mode === 'plain' ? 20 : 40, fg: undefined });
    assert.equal(reads[0], 'range');
    assert.equal(reads[1], `ydisp:${mode === 'range' ? 3 : 2}`);
    assert.equal(reads.at(-1), `cols:${mode === 'plain' ? 20 : 40}`);
  }
  cases.push({ mode, show, reads, event: event ?? null, caughtMarker: caught === marker });
  }
}
await mkdir('results/link-underline-geometry', { recursive: true });
await writeFile('results/link-underline-geometry/methods.json', JSON.stringify({ typescript, cases, sources: Object.fromEntries(hashes) }, null, 2) + '\n');
console.log(`${cases.length} unchanged-source underline geometry cases PASS`);
