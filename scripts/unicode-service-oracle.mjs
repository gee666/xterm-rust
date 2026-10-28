// Copyright (c) 2019, 2021 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Execute read-only upstream UnicodeService.
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { strict as assert } from 'node:assert';
import { stripTypeScriptTypes } from 'node:module';
import { runInNewContext } from 'node:vm';
import { createHash } from 'node:crypto';
import path from 'node:path';
import { verifyOrGenerate } from './oracle-fixture.mjs';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
async function fixture(relative, content) {
  const file = path.join(root, relative);
  await verifyOrGenerate(file, content, process.argv.includes('--generate'));
}
const files = ['Lifecycle.ts', 'Event.ts', 'services/UnicodeService.ts', 'input/UnicodeV6.ts', 'services/UnicodeService.test.ts'];
const sources = Object.fromEntries(await Promise.all(files.map(async name => [name, await readFile(path.join(root, 'source_repo/src/common', name), 'utf8')])));
const transpile = source => stripTypeScriptTypes(source.replace(/^import .*;\r?\n/gm, '').replace(/^export /gm, ''), { mode: 'transform' });
const baseline = [];
let beforeEach = () => {};
const sandbox = {
  assert: {
    equal: assert.strictEqual,
    deepEqual(actual, expected) { assert.deepEqual(JSON.parse(JSON.stringify(actual)), JSON.parse(JSON.stringify(expected))); },
    doesNotThrow: assert.doesNotThrow,
    throws(callback, expected) {
      assert.throws(callback, error => String(error.message).includes(expected));
    },
  },
  describe(name, callback) { callback(); },
  beforeEach(callback) { beforeEach = callback; },
  it(name, callback) {
    beforeEach();
    callback();
    baseline.push({ name, status: 'passed' });
  },
};
const runtime = files.map(file => transpile(sources[file])).join('\n');
const { UnicodeService, UnicodeV6 } = runInNewContext(`${runtime}\n({UnicodeService, UnicodeV6});`, sandbox);
assert.equal(baseline.length, 5);

const traceLog = [];
class TraceProvider {
  version = 'trace';
  wcwidth(codepoint) {
    traceLog.push(`w:${codepoint}`);
    return codepoint % 3;
  }
  charProperties(codepoint, preceding) {
    traceLog.push(`p:${codepoint}:${preceding}`);
    const state = ((preceding >>> 3) + codepoint) & 0xffffff;
    return (state << 3) | ((codepoint % 4) << 1) | ((codepoint & 1) === 0 && preceding !== 0 ? 1 : 0);
  }
}
const standard = new UnicodeService();
standard.register(new UnicodeV6());
const custom = new UnicodeService();
custom.register(new TraceProvider());
const inputs = [[], [0xd800], [0xdc00], [0xd800, 0x41], [0xd800, 0x301], [0xd800, 0xd800, 0xdc00], [0xd800, 0x4e00], [0xdbff, 0xdfff], [0x41, 0x301, 0x302], [0x4e00, 0x301, 0xd800]];
const seed = 0x71de63a1;
let state = seed;
function random() {
  state ^= state << 13;
  state ^= state >>> 17;
  state ^= state << 5;
  return state >>> 0;
}
for (let i = 0; i < 4096; i++) {
  const input = [];
  const length = random() % 40;
  for (let n = 0; n < length; n++) {
    const next = random();
    input.push(next % 5 === 0 ? 0xd800 + (next % 0x800) : next & 0xffff);
  }
  inputs.push(input);
}
const rows = [];
for (const input of inputs) {
  const text = String.fromCharCode(...input);
  traceLog.length = 0;
  const v6 = standard.getStringCellWidth(text);
  const traced = custom.getStringCellWidth(text);
  rows.push([input.map(unit => unit.toString(16)).join(','), v6, traced, traceLog.join(',')].join('\t'));
}
const fixtures = path.join(root, 'crates/xterm-core/tests/fixtures');
await mkdir(fixtures, { recursive: true });
await fixture('crates/xterm-core/tests/fixtures/unicode-service-widths.tsv', rows.join('\n') + '\n');

const registry = new UnicodeService();
const events = [];
registry.onChange(version => events.push(version));
const names = ['6', '11', '2', '01', '-0', '4294967294', '4294967295', '', '__proto__', 'constructor', '0'];
for (const version of names) registry.register({ version, wcwidth: () => 1, charProperties: () => 2 });
const registryResult = { names, versions: Array.from(registry.versions), active: registry.activeVersion, events };
await mkdir(path.join(root, 'results/unicode-service'), { recursive: true });
await writeFile(path.join(root, 'results/unicode-service/registry.json'), JSON.stringify(registryResult, null, 2) + '\n');
const registryTraces = [];
const tracedRegistry = new UnicodeService();
const notifications = [];
tracedRegistry.onChange(version => notifications.push(version));
const encoded = text => text === '' ? '~' : Array.from(text, char => char.charCodeAt(0).toString(16)).join(',');
const encodedList = values => values.length === 0 ? '-' : values.map(encoded).join(';');
function registryStep(op, name = '', width = 0) {
  let error = '-';
  try {
    if (op === 'r') tracedRegistry.register({ version: name, wcwidth: () => width, charProperties: () => width << 1 });
    else if (op === 's') tracedRegistry.activeVersion = name;
    else tracedRegistry.dispose();
  } catch (failure) {
    assert.equal(op, 's');
    assert.equal(failure.message, `unknown Unicode version "${name}"`);
    error = failure.message;
  }
  const activeWidth = tracedRegistry.wcwidth(65);
  registryTraces.push([op, encoded(name), width, encodedList(tracedRegistry.versions), encoded(tracedRegistry.activeVersion), activeWidth, error, encodedList(notifications)].join('\t'));
}
registryStep('r', '', 0);
registryStep('r', '6', 1);
registryStep('r', '6', 2);
registryStep('s', '6');
registryStep('s', 'missing');
for (const name of names) registryStep('r', name, random() % 3);
for (let i = 0; i < 512; i++) {
  const name = [...names, 'missing'][random() % (names.length + 1)];
  registryStep(random() % 3 === 0 ? 's' : 'r', name, random() % 3);
}
registryStep('d');
registryStep('r', 'after-dispose', 2);
registryStep('s', 'after-dispose');
await fixture('crates/xterm-core/tests/fixtures/unicode-service-registry.tsv', registryTraces.join('\n') + '\n');
const report = {
  scope: 'UnicodeService only; full terminal equivalence NOT PROVEN',
  environment: { node: process.version, platform: process.platform, architecture: process.arch },
  baseline: { discovered: baseline.length, executed: baseline.length, passed: baseline.length, failed: 0, skipped: 0, unrun: 0, tests: baseline },
  differential: { seed, deterministicInputs: 10, randomInputs: 4096, totalInputs: rows.length, providers: ['6', 'trace'], comparison: 'width and exact provider call log', registryOperations: registryTraces.length }, 
  sources: Object.entries(sources).map(([name, content]) => ({ path: `source_repo/src/common/${name}`, sha256: createHash('sha256').update(content).digest('hex') })),
  fixtureHashes: {
    'unicode-service-widths.tsv': createHash('sha256').update(rows.join('\n') + '\n').digest('hex'),
    'unicode-service-registry.tsv': createHash('sha256').update(registryTraces.join('\n') + '\n').digest('hex'),
  },
};
await writeFile(path.join(root, 'results/unicode-service/oracle.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report, null, 2));
