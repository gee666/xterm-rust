// Run unchanged source methods in memory. The opened-Terminal trace is separate.
import assert from 'node:assert/strict';
import { mkdir, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { sourceLoader } from './source-ts-loader.mjs';
const root = process.env.SOURCE_ROOT ? resolve(process.env.SOURCE_ROOT, '..') : process.cwd();
const { source, load, hashes, typescript } = sourceLoader(root);
const { Linkifier } = load(resolve(source, 'src/browser/Linkifier.ts'));
const cases = [];
for (const mode of ['unchanged', 'first decoration read', 'last decoration read', 'underline reentry', 'clear event', 'throw', 'throw hover']) {
  const target = Object.create(Linkifier.prototype);
  const outer = { marker: 'outer' }, nested = { marker: 'nested' }, marker = { marker: 'throw' };
  const calls = [];
  target._lastMouseEvent = outer;
  target._element = { classList: { add() {} } };
  target._bufferService = { cols: 20, buffer: { ydisp: 0 } };
  target._renderService = { onRenderedViewportChange() { return { dispose() {} }; } };
  target._linkCacheDisposables = [];
  target._positionFromMouseEvent = () => ({ x: 4, y: 1 });
  target._fireUnderlineEvent = () => { target._lastMouseEvent = nested; };
  let reads = 0;
  const link = { text: 'link', range: { start: { x: 2, y: 1 }, end: { x: 8, y: 1 } },
    hover(event) {
      calls.push(event === outer ? 'outer' : event === nested ? 'nested' : 'undefined');
      if (mode === 'throw hover') throw marker;
    } };
  Object.defineProperty(link, 'decorations', { configurable: true, get() {
    reads++;
    if (mode === 'first decoration read' && reads === 1 || mode === 'last decoration read' && reads === 4) target._lastMouseEvent = nested;
    if (mode === 'clear event') target._lastMouseEvent = undefined;
    if (mode === 'throw') throw marker;
    return { underline: mode === 'underline reentry', pointerCursor: false };
  }, set(value) { Object.defineProperty(link, 'decorations', { value, writable: true }); } });
  let caught;
  try { target._handleNewLink({ link }); } catch (error) { caught = error; }
  const expected = mode === 'clear event' ? 'undefined' : mode.includes('decoration read') ? 'nested' : 'outer';
  assert.deepEqual(calls, mode === 'throw' ? [] : [expected]);
  assert.equal(caught === marker, mode.startsWith('throw'));
  cases.push({ mode, reads, calls, marker: caught === marker });
}
await mkdir('results/link-hover-event', { recursive: true });
await writeFile('results/link-hover-event/methods.json', JSON.stringify({ typescript, cases, sources: Object.fromEntries(hashes) }, null, 2) + '\n');
console.log(JSON.stringify(cases));
