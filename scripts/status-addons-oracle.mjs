// Copyright (c) 2026 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Run with Node --experimental-transform-types.
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { pathToFileURL } from 'node:url';
import path from 'node:path';

const root = process.cwd();
const source = path.resolve('source_repo/addons/addon-program-status/src/ProgramStatus.ts');
const tests = path.resolve('source_repo/addons/addon-program-status/src/ProgramStatus.test.ts');
const progressSource = path.resolve('source_repo/addons/addon-progress/src/ProgressAddon.ts');
for (const dir of ['tmp/status-addons', 'results/status-addons', 'crates/xterm-addons/tests/fixtures']) {
  await mkdir(dir, { recursive: true });
}
const original = await import(pathToFileURL(source));
const fixture = { parse: [], progress: [] };
const units = s => Array.from({ length: s.length }, (_, i) => s.charCodeAt(i));
globalThis.statusParse = data => {
  const result = original.parseProgramStatusReport(data);
  fixture.parse.push({ input: units(data), expected: result ?? null });
  return result;
};
await writeFile('tmp/status-addons/tracked.mts', `export { ProgramStatusStore } from '${pathToFileURL(source).href}';\nexport const parseProgramStatusReport = globalThis.statusParse;\n`);
const baselineText = (await readFile(tests, 'utf8'))
  .replace("import { assert } from 'chai';", "import { strict as assert } from 'node:assert';")
  .replace("from './ProgramStatus'", "from './tracked.mts'");
await writeFile('tmp/status-addons/baseline.mts', baselineText);
const suites = [], outcomes = [], before = [];
globalThis.describe = (name, fn) => {
  suites.push(name); before.push([]);
  try { fn(); } finally { suites.pop(); before.pop(); }
};
globalThis.beforeEach = fn => before.at(-1).push(fn);
globalThis.it = (name, fn) => {
  try {
    for (const hooks of before) for (const hook of hooks) hook();
    fn(); outcomes.push({ name: [...suites, name].join(' / '), status: 'passed' });
  } catch (error) { outcomes.push({ name, status: 'failed', error: String(error.stack) }); }
};
await import(pathToFileURL(path.resolve('tmp/status-addons/baseline.mts')));
for (const key of ['describe', 'beforeEach', 'it']) delete globalThis[key];

const seed = 0x750109;
let state = seed;
const random = () => { state ^= state << 13; state ^= state >>> 17; state ^= state << 5; return state >>> 0; };
const words = ['state=working', 'state=clear', 'state=done', 'state=blocked', 'kind=auth', 'id=a/b', 'id=', 'msg=QQ', 'msg=QR', 'msg=77u/aGk=', 'app=root', 'progress=007', 'progress=101', 'msg=AA==', 'foo=bar'];
for (let i = 0; i < 2048; i++) {
  const parts = Array.from({ length: random() % 10 }, () => words[random() % words.length]);
  if (i % 8 === 0) parts.push(String.fromCharCode(random() % 65536) + '=a');
  globalThis.statusParse(parts.join(':'));
}
for (const [key, boundary] of [['pad',4072],['msg',2732],['title',256],['app',32],['id',128]]) {
  for (const delta of [-1,0,1]) for (const c of ['a', '\u00e9', '\ud800', '\ud83d\ude00']) {
    globalThis.statusParse(`state=idle:${key}=${c.repeat(boundary + delta)}`);
  }
}
delete globalThis.statusParse;

const { ProgressAddon } = await import(pathToFileURL(progressSource));
class Emitter {
  listeners = [];
  event = listener => { this.listeners.push(listener); return { dispose() {} }; };
  fire(value) { for (const listener of this.listeners) listener(value); }
  dispose() { this.listeners = []; }
}
const addon = new ProgressAddon();
let callback;
addon.activate({ parser: { registerOscHandler(id, cb) { callback = cb; return { dispose() {} }; } }, _core: { _onData: new Emitter() } });
let events = [];
addon.onChange(value => events.push(value));
const progressValues = ['', '0', '12', '123', '-1', '12x', '0'.repeat(1000), '9'.repeat(1000), '1.5', '\ud800'];
for (let i = 0; i < 2048; i++) {
  const input = `${i % 17 === 0 ? '5' : '4'};${random() % 7}${i % 3 ? ';' + progressValues[random() % progressValues.length] : ''}${i % 19 === 0 ? ';extra' : ''}`;
  events = [];
  const handled = callback(input);
  fixture.progress.push({ input: units(input), handled, expected: addon.progress, events });
}
await writeFile('crates/xterm-addons/tests/fixtures/status-addons.json', JSON.stringify(fixture) + '\n');
const hash = async file => createHash('sha256').update(await readFile(file)).digest('hex');
const result = {
  command: 'node --experimental-transform-types scripts/status-addons-oracle.mjs',
  node: process.version, platform: process.platform, arch: process.arch, seed,
  source: path.relative(root, source), sourceSha256: await hash(source), testsSha256: await hash(tests), progressSha256: await hash(progressSource),
  baseline: { discovered: outcomes.length, executed: outcomes.length, passed: outcomes.filter(o => o.status === 'passed').length, failed: outcomes.filter(o => o.status === 'failed').length, skipped: 0, unrun: 0, outcomes },
  parseCases: fixture.parse.length, progressCases: fixture.progress.length,
  fixtureSha256: await hash('crates/xterm-addons/tests/fixtures/status-addons.json'),
  limitations: ['Pure ProgramStatus.test.ts baseline only. Headless/browser integration suites not executed here.', 'Progress oracle uses a fixture terminal, not live core.']
};
await writeFile('results/status-addons/oracle.json', JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify(result));
if (result.baseline.failed) process.exitCode = 1;
