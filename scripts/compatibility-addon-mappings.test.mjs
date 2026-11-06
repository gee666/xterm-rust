// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
import assert from 'node:assert/strict';
import {randomUUID} from 'node:crypto';
import {readFile, writeFile, mkdir, rm} from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import path from 'node:path';
import test from 'node:test';
const root = fileURLToPath(new URL('../', import.meta.url));
const load = async p => JSON.parse(await readFile(path.join(root, p), 'utf8'));
const run = args => spawnSync(process.execPath, ['scripts/compatibility-inventory.mjs', ...args], {
  cwd: root, encoding: 'utf8', timeout: 60_000, maxBuffer: 4 * 1024 * 1024
});
const combine = async () => {
  const base = await load('tools/compatibility/mappings.json');
  const addons = await load('tools/compatibility/addon-mappings.json');
  return Object.fromEntries(['modules', 'apiOwners', 'suites'].map(k => [k, [...(base[k] ?? []), ...(addons[k] ?? [])]]));
};

test('all fourteen addons link scoped components without certifying APIs, GPU or suite outcomes', async () => {
  const addons = await load('results/compatibility/addons.json');
  const modules = await load('results/compatibility/modules.json');
  const apis = await load('results/compatibility/apis.json');
  const suites = await load('results/compatibility/suites.json');
  assert.equal(addons.length, 14);
  assert(addons.every(a => a.status === 'partial' && a.gap.includes('do not certify')));
  const webglPrefix = 'source_repo/addons/addon-webgl/';
  for (const suffix of ['src/WebglAddon.ts', 'src/WebglRenderer.ts', 'typings/addon-webgl.d.ts']) {
    assert.equal(modules.find(m => m.source === webglPrefix + suffix).status, 'missing');
  }
  assert(apis.filter(a => a.source.startsWith(webglPrefix)).every(a => a.status === 'missing'));
  assert.equal(modules.find(m => m.source.endsWith('/src/ImageAddon.ts')).status, 'partial');
  assert(apis.some(a => a.owner === 'ImageAddon' && a.name === 'getImageAtBufferCell' && a.status === 'partial'));
  const addonSuites = suites.filter(s => s.source.startsWith('source_repo/addons/'));
  assert.equal(addonSuites.length, 50);
  for (const suite of addonSuites) {
    assert.equal(suite.status, 'partial', suite.source);
    assert(suite.tests.length && suite.evidence.length && suite.gap);
    for (const field of ['discovered', 'executed', 'passed', 'failed', 'skipped', 'unrun']) assert.equal(suite.execution.rust[field], null);
  }
  const summary = await load('results/compatibility/summary.json');
  assert.equal(summary.equivalence, 'NOT PROVEN');
  assert.deepEqual(summary.counts.moduleStatuses, {missing: 114, partial: 95});
  assert.deepEqual(summary.counts.apiStatuses, {missing: 602, partial: 168});
  const hashes = await load('results/compatibility/evidence-manifest.json');
  for (const p of ['tools/compatibility/mappings.json', 'tools/compatibility/addon-mappings.json', 'tools/compatibility/behavior-criteria.json']) assert(hashes.some(h => h.path === p));
});

test('qualified owners cannot leak to another declaration and unknown/duplicate mappings fail', async () => {
  const dir = `tmp/addon-mapping-test-${randomUUID()}`;
  await mkdir(path.join(root, dir), {recursive:true});
  try {
    const original = await combine();
    const mapping = `${dir}/mappings.json`;
    const args = [`--out=${dir}/out`, `--mappings=${mapping}`];
    const check = async (value, error) => {
      await writeFile(path.join(root, mapping), JSON.stringify(value));
      const result = run(args);
      assert.ifError(result.error);
      if (error) { assert.notEqual(result.status, 0); assert.match(result.stderr, error); }
      else assert.equal(result.status, 0, result.stderr);
    };
    const qualified = structuredClone(original);
    const disposable = qualified.apiOwners.find(a => a.owner === 'IDisposable');
    disposable.source = 'source_repo/typings/xterm-headless.d.ts';
    await check(qualified);
    const apis = await load(`${dir}/out/apis.json`);
    const headless = apis.filter(a => a.source === disposable.source && (a.owner === 'IDisposable' || a.name === 'IDisposable'));
    const browser = apis.filter(a => a.source === 'source_repo/typings/xterm.d.ts' && (a.owner === 'IDisposable' || a.name === 'IDisposable'));
    assert(headless.length && headless.every(a => a.status === 'partial'));
    assert(browser.length && browser.every(a => a.status === 'missing'));
    await check({...original, apiOwners:[...original.apiOwners, original.apiOwners[0]]}, /Duplicate API owner mapping/);
    await check({...original, suites:[...original.suites, original.suites[0]]}, /Duplicate suite mapping/);
    const badOwner = structuredClone(original);
    badOwner.apiOwners[0].source = 'source_repo/addons/addon-fit/typings/addon-fit.d.ts';
    await check(badOwner, /Unknown API owner mapping/);
    const badSuite = structuredClone(original);
    badSuite.suites[0].source = 'source_repo/test/absent.test.ts';
    await check(badSuite, /Unknown suite mapping/);
  } finally { await rm(path.join(root, dir), {recursive:true}); }
});
