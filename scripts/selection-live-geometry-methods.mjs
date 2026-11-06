// Unchanged source methods with callbacks that mutate the model's live geometry.
import assert from 'node:assert/strict';
import { mkdir, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { sourceLoader } from './source-ts-loader.mjs';
const { source, load, hashes, typescript } = sourceLoader(process.env.SOURCE_ROOT ? resolve(process.env.SOURCE_ROOT, '..') : process.cwd());
const { SelectionService } = load(resolve(source, 'src/browser/services/SelectionService.ts'));
const { SelectionModel } = load(resolve(source, 'src/browser/selection/SelectionModel.ts'));
const traces = [];
for (const phase of ['teardown', 'refresh']) {
  for (const cols of [10, 32]) {
    const geometry = { cols: 20, rows: 4, buffer: { ybase: 0 } };
    const target = Object.create(SelectionService.prototype);
    target._model = new SelectionModel(geometry);
    const events = [];
    let armed = true;
    const resize = () => { if (armed) { armed = false; geometry.cols = cols; } };
    target._onSelectionChange = { fire() { events.push({ start: target._model.finalSelectionStart, end: target._model.finalSelectionEnd }); } };
    target._removeMouseDownListeners = () => { if (phase === 'teardown') resize(); };
    target.refresh = () => { if (phase === 'refresh') resize(); };
    target.setSelection(8, 0, 15);
    assert.deepEqual(events, [{ start: [8, 0], end: cols === 10 ? [3, 2] : [23, 0] }]);
    target.setSelection(8, 0, 15);
    assert.equal(events.length, 1, 'identical select must compare the live endpoint captured on the first call');
    traces.push({ phase, cols, events });
  }
}
await mkdir('results/selection-live-geometry', { recursive: true });
await writeFile('results/selection-live-geometry/methods.json', JSON.stringify({ typescript, traces, sources: Object.fromEntries(hashes) }, null, 2) + '\n');
console.log(`Unchanged source live-geometry: ${traces.length} cases passed`);
