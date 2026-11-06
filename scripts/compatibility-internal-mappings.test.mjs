// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import test from 'node:test';
const load = async name => JSON.parse(await readFile(new URL(`../${name}`, import.meta.url), 'utf8'));

test('common/browser guides cover source paths without manufacturing execution or addon bindings', async () => {
  const mapping = await load('tools/compatibility/internal-mappings.json');
  assert.equal(mapping.modules.length, 86);
  assert.equal(mapping.apiOwners.length, 29);
  assert.equal(mapping.suites.length, 15);
  const modules = await load('results/compatibility/modules.json');
  for (const module of modules.filter(m => /source_repo\/src\/(common|browser|headless)\//.test(m.source))) {
    assert.equal(module.status, 'partial', module.source);
    assert(module.implementation.length && module.evidence.length);
    // The older Win32 mapping keeps its tests inline in the implementation.
    if (mapping.modules.some(m => m.source === module.source)) assert(module.tests.length, module.source);
    assert(module.command && module.binding && module.gap);
  }
  assert.equal(modules.find(m => m.source === 'source_repo/css/xterm.css').status, 'partial');
  const utils = modules.find(m => m.source.endsWith('/renderer/shared/RendererUtils.ts'));
  assert(utils.implementation.includes('crates/xterm-web/src/renderer/utils.rs'));
  assert(utils.tests.includes('crates/xterm-web/tests/webgl_cases/renderer_utils.rs'));
  assert.equal(utils.command, 'cargo test --locked -p xterm-web --test webgl');
  const selection = modules.find(m => m.source.endsWith('/renderer/shared/SelectionRenderModel.ts'));
  assert(selection.implementation.includes('crates/xterm-web/src/renderer/selection_model.rs'));
  assert.deepEqual(selection.tests, ['crates/xterm-web/tests/webgl_renderer.rs']);
  assert.equal(selection.command, 'cargo test --locked -p xterm-web --test webgl_renderer');
  assert.match(selection.gap, /indirect selection geometry coverage/);
  for (const row of [utils, selection]) {
    assert(!row.tests.includes('crates/xterm-web/tests/render_callback_iteration.rs'));
    assert(!row.tests.includes('crates/xterm-web/tests/selection_model.rs'));
  }
  const missing = modules.filter(m => m.status === 'missing');
  assert.equal(missing.length, 17);
  assert(missing.every(m => m.source.startsWith('source_repo/addons/')));
  assert(missing.some(m => m.source.endsWith('/WebglAddon.ts')));
  const suites = await load('results/compatibility/suites.json');
  assert.equal(suites.length, 120);
  for (const suite of suites) {
    assert.equal(suite.status, 'partial', suite.source);
    assert(suite.gap, suite.source);
    for (const field of ['discovered', 'executed', 'passed', 'failed', 'skipped', 'unrun']) assert.equal(suite.execution.rust[field], null);
  }
  const fixtures = await load('results/compatibility/fixtures.json');
  assert.equal(fixtures.length, 162);
  assert(fixtures.every(f => f.status === 'unrun' && f.implementation.length === 0));
  const summary = await load('results/compatibility/summary.json');
  assert.equal(summary.equivalence, 'NOT PROVEN');
  assert.equal(summary.counts.staticTestDefinitions, 2426);
  assert.equal(summary.counts.staticBenchmarkDefinitions, 32);
  assert.equal(summary.counts.modulesWithoutFocusedCriteria, 4);
  assert.equal(summary.counts.apisWithoutFocusedCriteria, 261);
  assert.equal(summary.counts.casesWithoutFocusedCriteria, 690);
});

test('record mappings stay source-qualified and leave uninspected or absent shapes missing', async () => {
  const mapping = await load('tools/compatibility/internal-mappings.json');
  assert(mapping.apiOwners.every(a => /^source_repo\/typings\/xterm(-headless)?\.d\.ts$/.test(a.source)));
  const apis = await load('results/compatibility/apis.json');
  const member = (source, owner, name) => apis.find(a => a.source.endsWith(source) && a.owner === owner && a.name === name);
  assert.equal(member('xterm.d.ts', 'IDecoration', 'onRender').status, 'partial');
  assert.equal(member('xterm.d.ts', 'ILink', 'activate').status, 'partial');
  assert.equal(member('xterm.d.ts', 'ILocalizableStrings', 'promptLabel').status, 'partial');
  assert.equal(member('xterm-headless.d.ts', 'ILocalizableStrings', 'promptLabel').status, 'missing');
  assert.equal(member('xterm-headless.d.ts', 'Terminal', 'strings').status, 'missing');
  for (const api of apis.filter(a => a.name === 'IViewportRange' || a.owner === 'IViewportRange' || a.name === 'IBufferElementProvider' || a.owner === 'IBufferElementProvider')) {
    assert.equal(api.status, 'missing', api.id);
    assert.deepEqual(api.implementation, []);
  }
  assert.equal(member('xterm.d.ts', 'IWindowOptions', 'restoreWin').status, 'partial');
  const hashes = await load('results/compatibility/evidence-manifest.json');
  assert(hashes.some(h => h.path === 'tools/compatibility/internal-mappings.json'));
});
