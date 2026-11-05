// Execute the unchanged source method. Browser/public evidence is separate.
import assert from 'node:assert/strict';
import { mkdir, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { sourceLoader } from './source-ts-loader.mjs';
const root = process.env.SOURCE_ROOT ? resolve(process.env.SOURCE_ROOT, '..') : process.cwd();
const { source, load, hashes, typescript } = sourceLoader(root);
const { Linkifier } = load(resolve(source, 'src/browser/Linkifier.ts'));
const cases = [];
for (const mode of ['unchanged', 'start y', 'end y', 'start x', 'end x', 'start coercion', 'end coercion', 'reset end', 'throw start', 'throw end']) {
  const target = Object.create(Linkifier.prototype);
  const reads = [], marker = {};
  let cols = 20;
  target._bufferService = { get cols() { reads.push(`cols:${cols}`); return cols; } };
  const axis = (end, key, value) => ({ get [key]() {
    reads.push(`${end}.${key}`);
    if (mode === `throw ${end}` && key === 'y') throw marker;
    if (mode === `${end} ${key}`) cols = 40;
    if (mode === 'reset end' && key === 'y') cols = end === 'start' ? 40 : 20;
    return key === 'y' && mode === `${end} coercion` ? { valueOf() {
      reads.push(`${end}.valueOf`); cols = 40; return value;
    } } : value;
  } });
  const point = (end, x) => Object.defineProperties({}, {
    ...Object.getOwnPropertyDescriptors(axis(end, 'x', x)),
    ...Object.getOwnPropertyDescriptors(axis(end, 'y', 1))
  });
  const range = { start: point('start', 1), end: point('end', 20) };
  let result, caught;
  try { result = target._linkAtPosition({ range }, { x: 2, y: 1 }); } catch (error) { caught = error; }
  const expected = !['end x', 'end coercion', 'reset end'].includes(mode);
  if (mode.startsWith('throw')) assert.equal(caught, marker);
  else assert.equal(result, expected, mode);
  // Source reads each y before its columns operand, and samples current last.
  if (!caught) assert.deepEqual(reads.filter(read => read.startsWith('cols:')), mode === 'reset end' ? ['cols:40', 'cols:20', 'cols:20'] : mode === 'unchanged' ? ['cols:20', 'cols:20', 'cols:20'] : mode === 'start y' ? ['cols:40', 'cols:40', 'cols:40'] : mode === 'end x' || mode === 'end coercion' ? ['cols:20', 'cols:20', 'cols:40'] : ['cols:20', 'cols:40', 'cols:40']);
  cases.push({ mode, result: result ?? null, marker: caught === marker, reads });
}
await mkdir('results/link-live-cols', { recursive: true });
await writeFile('results/link-live-cols/methods.json', JSON.stringify({ typescript, cases, sources: Object.fromEntries(hashes) }, null, 2) + '\n');
console.log(JSON.stringify(cases));
