// Copyright (c) 2016, 2019, 2026 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Runs the read-only upstream charset reference.
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { strict as assert } from 'node:assert';
import { pathToFileURL, fileURLToPath } from 'node:url';
import path from 'node:path';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const paths = {
  maps: 'source_repo/src/common/data/Charsets.ts',
  service: 'source_repo/src/common/services/CharsetService.ts',
  tests: 'source_repo/src/common/services/CharsetService.test.ts'
};
const fixtureDir = path.join(root, 'crates/xterm-core/tests/fixtures');
const resultDir = path.join(root, 'results/charset');
const tempDir = path.join(root, 'tmp/charset-oracle');
await Promise.all([fixtureDir, resultDir, tempDir].map(dir => mkdir(dir, { recursive: true })));
const texts = Object.fromEntries(await Promise.all(Object.entries(paths).map(async ([key, file]) =>
  [key, await readFile(path.join(root, file), 'utf8')])));
const hash = async file => createHash('sha256').update(await readFile(path.join(root, file))).digest('hex');
// These dependencies are interfaces only. Node's native stripping retains their
// non-type import syntax, so remove the import lines in temporary copies.
const mapFile = path.join(tempDir, 'Charsets.mts');
const serviceFile = path.join(tempDir, 'CharsetService.mts');
await writeFile(mapFile, texts.maps.replace(/^import .*;\r?\n/gm, ''));
await writeFile(serviceFile, texts.service.replace(/^import .*;\r?\n/gm, ''));
const { CHARSETS, DEFAULT_CHARSET } = await import(pathToFileURL(mapFile).href);
const { CharsetService } = await import(pathToFileURL(serviceFile).href);

const outcomes = [];
const suites = [];
let beforeEach = () => {};
globalThis.describe = (name, callback) => {
  suites.push(name);
  try { callback(); } finally { suites.pop(); }
};
globalThis.beforeEach = callback => { beforeEach = callback; };
globalThis.it = (name, callback) => {
  try {
    beforeEach();
    callback();
    outcomes.push({ name: [...suites, name].join(' / '), status: 'passed' });
  } catch (error) {
    outcomes.push({ name: [...suites, name].join(' / '), status: 'failed', error: String(error.stack) });
  }
};
const testFile = path.join(tempDir, 'CharsetService.test.mts');
const baseline = texts.tests
  .replace("import { assert } from 'chai';", "import { strict as assert } from 'node:assert';")
  .replace("from '../data/Charsets'", `from '${pathToFileURL(mapFile).href}'`)
  .replace("from './CharsetService'", `from '${pathToFileURL(serviceFile).href}'`);
await writeFile(testFile, baseline);
try { await import(pathToFileURL(testFile).href); } finally {
  delete globalThis.describe;
  delete globalThis.beforeEach;
  delete globalThis.it;
}

const hex = value => value.toString(16);
const units = value => Array.from({ length: value.length }, (_, i) => hex(value.charCodeAt(i))).join(',');
const designators = Object.keys(CHARSETS).sort();
const canonical = value => value === undefined ? '-' : units(designators.find(id => CHARSETS[id] === value));
assert.equal(DEFAULT_CHARSET, undefined);
assert.equal(CHARSETS.B, undefined);
const mappings = [];
// Include every ASCII character, not just mapped entries, plus an unknown set.
for (const id of [...designators, 'X']) {
  for (let code = 0; code < 128; code++) {
    const mapped = CHARSETS[id]?.[String.fromCharCode(code)];
    mappings.push([units(id), hex(code), mapped === undefined ? '-' : units(mapped)].join('\t'));
  }
}
await writeFile(path.join(fixtureDir, 'charset-maps.tsv'), mappings.join('\n') + '\n');

const service = new CharsetService();
const traces = [];
function step(op, g, id) {
  if (op === 's') service.setgCharset(g, id === undefined ? undefined : CHARSETS[id]);
  else if (op === 'l') service.setgLevel(g);
  else service.reset();
  traces.push([
    op, g === undefined ? '-' : g, id === undefined ? '-' : units(id), service.glevel,
    canonical(service.charset), Array.from(service.charsets, canonical).join(',')
  ].join('\t'));
}
step('s', 1, '0');
step('l', 1);
step('s', 1, '4');
step('s', 7, 'C');
step('l', 7);
step('s', 7, '5');
step('s', 16, '6');
step('l', 16);
step('s', 16, undefined);
step('l', 15);
step('r');
const seed = 0xc4a25e7;
let state = seed;
function random() {
  state ^= state << 13;
  state ^= state >>> 17;
  state ^= state << 5;
  return state >>> 0;
}
for (let i = 0; i < 4096; i++) {
  const op = random() % 16;
  const g = random() % 17;
  if (op === 0) step('r');
  else if (op < 6) step('l', g);
  else {
    const id = [...designators, 'X', undefined][random() % (designators.length + 2)];
    step('s', g, id);
  }
}
await writeFile(path.join(fixtureDir, 'charset-service.tsv'), traces.join('\n') + '\n');
const report = {
  reference: await Promise.all(Object.entries(paths).map(async ([kind, file]) => ({
    kind, path: file, sha256: await hash(file)
  }))),
  environment: { node: process.version, platform: process.platform, arch: process.arch },
  harness: 'Native Node TypeScript stripping; temporary import resolution; Node strict assertions replace chai without modifying assertion bodies.',
  fixtures: {
    mapRows: mappings.length, designators, definedMaps: designators.filter(id => CHARSETS[id] !== undefined).length,
    uniqueMaps: new Set(Object.values(CHARSETS).filter(value => value !== undefined)).size,
    stateRows: traces.length, randomOperations: 4096, seed,
    canonicalIdentity: 'Lexicographically smallest designator sharing the same object, encoded as UTF-16 hex; undefined is -.'
  },
  baseline: {
    discovered: outcomes.length, executed: outcomes.length,
    passed: outcomes.filter(outcome => outcome.status === 'passed').length,
    failed: outcomes.filter(outcome => outcome.status === 'failed').length,
    skipped: 0, unrun: 0, outcomes
  }
};
await writeFile(path.join(resultDir, 'oracle.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ maps: mappings.length, traces: traces.length, seed, baseline: report.baseline }));
if (report.baseline.failed) process.exitCode = 1;
