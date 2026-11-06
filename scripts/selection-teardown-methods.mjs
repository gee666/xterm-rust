// Unchanged SelectionService methods, with DOM teardown/refresh observation only.
import assert from 'node:assert/strict';
import { mkdir, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { sourceLoader } from './source-ts-loader.mjs';
const { source, load, hashes, typescript } = sourceLoader(process.env.SOURCE_ROOT ? resolve(process.env.SOURCE_ROOT, '..') : process.cwd());
const { SelectionService } = load(resolve(source, 'src/browser/services/SelectionService.ts'));
const { SelectionModel } = load(resolve(source, 'src/browser/selection/SelectionModel.ts'));
const traces = [];
const raw = model => ({ start: model.selectionStart ?? null, end: model.selectionEnd ?? null,
  length: model.selectionStartLength, all: model.isSelectAllActive });
const cleared = { start: null, end: null, length: 0, all: false };
for (const action of ['observe', 'throw', 'nested', 'fields-none', 'fields-end', 'fields-all', 'fields-both', 'fields-throw']) {
  const target = Object.create(SelectionService.prototype);
  target._model = new SelectionModel({ cols: 20, rows: 4, buffer: { ybase: 0 } });
  Object.assign(target._model, { selectionStart: [1, 1], selectionEnd: [8, 2], selectionStartLength: 4, isSelectAllActive: true });
  const events = [];
  const marker = {};
  let entered = false, caught = false;
  target._onSelectionChange = { fire() { events.push(['change', raw(target._model)]); } };
  target.refresh = () => events.push(['refresh', raw(target._model)]);
  target._removeMouseDownListeners = () => {
    assert.deepEqual(raw(target._model), cleared);
    events.push(['stop', raw(target._model)]);
    if (entered) return;
    entered = true;
    if (action === 'throw') throw marker;
    if (action === 'nested') target.setSelection(7, 0, 2);
    if (action.startsWith('fields-')) Object.assign(target._model, {
      selectionStart: [7, 2], selectionStartLength: 8,
      selectionEnd: ['fields-end', 'fields-both', 'fields-throw'].includes(action) ? [9, 1] : undefined,
      isSelectAllActive: ['fields-all', 'fields-both', 'fields-throw'].includes(action)
    });
    if (action === 'fields-throw') throw marker;
  };
  try { target.setSelection(2, 0, 3); } catch (error) { assert.equal(error, marker); caught = true; }
  const result = raw(target._model);
  assert.equal(caught, action.endsWith('throw'));
  assert.deepEqual(result, action === 'throw' ? cleared : action === 'fields-throw' ? {
    start: [7, 2], end: [9, 1], length: 8, all: true
  } : {
    start: [2, 0], end: ['fields-end', 'fields-both'].includes(action) ? [9, 1] : null,
    length: 3, all: ['fields-all', 'fields-both'].includes(action)
  });
  assert.deepEqual(events.map(([name]) => name), action.endsWith('throw') ? ['stop'] :
    action === 'nested' ? ['stop', 'stop', 'refresh', 'change', 'refresh', 'change'] : ['stop', 'refresh', 'change']);
  traces.push({ action, caught, result, events });
}
await mkdir('results/selection-teardown', { recursive: true });
await writeFile('results/selection-teardown/methods.json', JSON.stringify({ typescript, traces, sources: Object.fromEntries(hashes) }, null, 2) + '\n');
console.log(`Unchanged selection teardown: ${traces.length} cases passed`);
