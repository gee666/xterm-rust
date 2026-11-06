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
  const inputs = await Promise.all(['mappings', 'addon-mappings', 'public-mappings', 'internal-mappings'].map(p => load(`tools/compatibility/${p}.json`)));
  return Object.fromEntries(['modules', 'apiOwners', 'suites'].map(k => [k, inputs.flatMap(i => i[k] ?? [])]));
};

test('public mappings retain source-specific names, representations, gaps and uncertified counts', async () => {
  const mappings = await load('tools/compatibility/public-mappings.json');
  const apis = await load('results/compatibility/apis.json');
  const modules = await load('results/compatibility/modules.json');
  const runtime = await load('results/compatibility/runtime-contracts.json');
  assert.equal(mappings.modules.length, 11);
  assert.equal(mappings.apiOwners.length, 32);
  assert.equal(mappings.suites.length, 5);
  assert(mappings.apiOwners.every(a => a.source.startsWith('source_repo/typings/')));
  for (const mapping of mappings.modules) assert.equal(modules.find(m => m.source === mapping.source).status, 'partial', mapping.source);
  for (const source of ['source_repo/typings/xterm-headless.d.ts', 'source_repo/typings/xterm.d.ts']) {
    const member = (owner, name) => apis.find(a => a.source === source && a.owner === owner && a.name === name);
    for (const [owner, name] of [['Terminal', 'write'], ['Terminal', 'onData'], ['Terminal', 'unicode'], ['ITerminalOptions', 'allowProposedApi'], ['IBuffer', 'getLine'], ['IBufferCell', 'getFgColor'], ['IParser', 'registerApcHandler'], ['IUnicodeHandling', 'activeVersion'], ['IModes', 'showCursor']]) {
      const row = member(owner, name);
      assert.equal(row?.status, 'partial', `${source}:${owner}.${name}`);
      assert(row.implementation.length && row.tests.length && row.evidence.length && row.command && row.binding && row.gap);
    }
    assert.equal(member('Terminal', 'unicode').proposedRuntimeGate, true);
    assert.equal(member('Terminal', 'parser').proposedRuntimeGate, false);
    assert.equal(member('Terminal', 'buffer').proposedRuntimeGate, false);
  }
  const headlessStrings = apis.find(a => a.module === '@xterm/headless' && a.owner === 'Terminal' && a.name === 'strings');
  assert.equal(headlessStrings.status, 'missing');
  assert.deepEqual(headlessStrings.implementation, []);
  assert.match(headlessStrings.gap, /declared.*absent/);
  const browserStrings = apis.find(a => a.module === '@xterm/xterm' && a.owner === 'Terminal' && a.name === 'strings');
  assert.equal(browserStrings.status, 'partial');
  assert(browserStrings.implementation.includes('browser/xterm.mjs'));
  // Data-only declarations outside the inspected families must stay unmapped.
  assert(apis.filter(a => a.name === 'IViewportRange' || a.owner === 'IViewportRange').every(a => a.status === 'missing'));
  assert(apis.filter(a => a.source.includes('addon-webgl/typings')).every(a => a.status === 'missing'));
  assert.equal(runtime.length, 2);
  for (const row of runtime) {
    assert.equal(row.status, 'partial');
    assert.deepEqual(row.proposedMembers, ['unicode']);
    assert.deepEqual(row.constructorOnlyOptions, ['cols', 'rows']);
    assert(row.binding.includes('cols/rows') && row.gap && row.command);
  }
  const suites = await load('results/compatibility/suites.json');
  for (const mapping of mappings.suites) {
    const row = suites.find(s => s.source === mapping.source);
    assert.equal(row.status, 'partial');
    for (const field of ['discovered', 'executed', 'passed', 'failed', 'skipped', 'unrun']) assert.equal(row.execution.rust[field], null);
  }
  const hashes = await load('results/compatibility/evidence-manifest.json');
  assert(hashes.some(h => h.path === 'tools/compatibility/public-mappings.json'));
  assert.equal((await load('results/compatibility/summary.json')).equivalence, 'NOT PROVEN');
});

test('known absent members override owners and malformed member exclusions fail closed', async () => {
  const dir = `tmp/public-mapping-test-${randomUUID()}`;
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
    const changed = structuredClone(original);
    const owner = changed.apiOwners.find(a => a.owner === 'Terminal' && a.source === 'source_repo/typings/xterm-headless.d.ts');
    owner.unmappedMembers = {write:'Directed missing-member validation fixture'};
    await check(changed);
    const apis = await load(`${dir}/out/apis.json`);
    const write = module => apis.find(a => a.module === module && a.owner === 'Terminal' && a.name === 'write');
    assert.equal(write('@xterm/headless').status, 'missing');
    assert.deepEqual(write('@xterm/headless').implementation, []);
    assert.equal(write('@xterm/headless').gap, owner.unmappedMembers.write);
    assert.equal(write('@xterm/xterm').status, 'partial', 'Exact-source exclusion must not leak to browser Terminal');
    owner.unmappedMembers = {absentMember:'No such member'};
    await check(changed, /Unknown unmapped API member/);
    owner.unmappedMembers = {write:''};
    await check(changed, /Unmapped API member requires a gap/);
    owner.unmappedMembers = [];
    await check(changed, /unmappedMembers must be a member-to-gap object/);
    owner.unmappedMembers = null;
    await check(changed, /unmappedMembers must be a member-to-gap object/);
    owner.unmappedMembers = {write:17};
    await check(changed, /Unmapped API member requires a gap/);
    // Explicit alternative mappings do not silently import supplemental data.
    const base = await load('tools/compatibility/mappings.json');
    await check(base);
    assert((await load(`${dir}/out/runtime-contracts.json`)).every(r => r.status === 'missing'));
  } finally { await rm(path.join(root, dir), {recursive:true}); }
});
