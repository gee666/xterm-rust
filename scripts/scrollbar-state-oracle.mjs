import { readFile, mkdir } from 'node:fs/promises';
import { stripTypeScriptTypes } from 'node:module';
import { runInNewContext } from 'node:vm';
import { strict as assert } from 'node:assert';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { verifyOrGenerate } from './oracle-fixture.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const source = await readFile(path.join(root, 'source_repo/src/browser/scrollable/scrollbarState.ts'), 'utf8');
const code = stripTypeScriptTypes(source.replace(/^export /gm, ''));
const State = runInNewContext(`${code}\nScrollbarState`);
const view = new DataView(new ArrayBuffer(8));
const bits = value => {
  if (Number.isNaN(value)) return 'NaN';
  view.setFloat64(0, value, true);
  return view.getBigUint64(0, true).toString(16).padStart(16, '0');
};
const values = [0, -0, -1, 1, 0.49999999999999994, 0.5, -0.5, -1.5, 10, 20, 100, 200,
  Number.MIN_VALUE, Number.MAX_VALUE, NaN, Infinity, -Infinity];
let randomState = 0x5c7011;
const random = () => { randomState ^= randomState << 13; randomState ^= randomState >>> 17; randomState ^= randomState << 5; return randomState >>> 0; };
const number = () => random() % 3 ? values[random() % values.length] : (random() % 800 - 200) / 2;
const output = state => [state.getArrowSize(), state.getScrollPosition(), state.getRectangleLargeSize(),
  state.getRectangleSmallSize(), state.getSliderSize(), state.getSliderPosition()];
const rows = [];
function step(state, op, args, probe) {
  let returned = '-';
  if (op === 'new') state = new State(...args);
  else if (op === 'clone') state = state.clone();
  else {
    const result = state[op](args[0]);
    if (typeof result === 'boolean') returned = String(result);
  }
  const observed = [...output(state), state.getDesiredScrollPositionFromOffset(probe),
    state.getDesiredScrollPositionFromOffsetPaged(probe), state.getDesiredScrollPositionFromDelta(probe)];
  rows.push([op, args.map(bits).join(','), bits(probe), returned, Number(state.isNeeded()), observed.map(bits).join(',')].join('\t'));
  return state;
}
const operations = ['setVisibleSize', 'setScrollSize', 'setScrollPosition', 'setScrollbarSize', 'setArrowSize', 'setOppositeScrollbarSize', 'clone'];
for (let scenario = 0; scenario < 128; scenario++) {
  const inputs = scenario === 0 ? [0, 10, 0, 100, 200, 40] : Array.from({ length: 6 }, number);
  let state = step(null, 'new', inputs, number());
  for (let index = 0; index < 7; index++) {
    const op = scenario === 0 ? ['setOppositeScrollbarSize', 'clone', 'setVisibleSize', 'setArrowSize', 'setScrollbarSize', 'setScrollPosition', 'clone'][index] : operations[random() % operations.length];
    const arg = scenario === 0 ? [20, 0, 100, 5, 12, -0.5, 0][index] : number();
    state = step(state, op, op === 'clone' ? [] : [arg], number());
  }
}
assert.equal(rows.length, 1024);
const destination = 'crates/xterm-web/tests/fixtures/scrollbar-state.tsv';
await mkdir(path.dirname(path.join(root, destination)), { recursive: true });
await verifyOrGenerate(path.join(root, destination), rows.join('\n') + '\n', process.argv.includes('--generate'));
console.log(`ScrollbarState: ${rows.length} unchanged-source snapshots; seed 0x5c7011`);
