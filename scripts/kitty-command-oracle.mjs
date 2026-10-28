// Copyright (c) 2026 The xterm.js authors. All rights reserved.
// Licensed under MIT. Original Kitty type assertions and deterministic controls.
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { stripTypeScriptTypes } from 'node:module';
import { createHash } from 'node:crypto';
import vm from 'node:vm';

const sourcePath = 'source_repo/addons/addon-image/src/kitty/KittyGraphicsTypes.ts';
const testPath = sourcePath.replace('.ts', '.test.ts');
const source = await readFile(sourcePath, 'utf8');
const tests = await readFile(testPath, 'utf8');
const erase = text => stripTypeScriptTypes(text.replace(/^import .*;\r?$/gm, '').replace(/^export /gm, ''), { mode: 'transform' });
const outcomes = [];
const cases = [];
const suites = [];
const context = vm.createContext({ assert,
  capture: text => cases.push(text),
  describe(name, callback) { suites.push(name); callback(); suites.pop(); },
  it(name, callback) {
    try { callback(); outcomes.push({ name: [...suites, name].join(' / '), status: 'passed' }); }
    catch (error) { outcomes.push({ name: [...suites, name].join(' / '), status: 'failed', error: String(error.stack) }); }
  },
});
vm.runInContext(erase(source) + '\nglobalThis.parse = parseKittyCommand;', context);
vm.runInContext('const originalParse = parseKittyCommand; parseKittyCommand = text => { capture(text); return originalParse(text); };', context);
vm.runInContext(erase(tests), context);
assert.equal(outcomes.length, 23, 'original Kitty type inventory changed');
assert.equal(outcomes.filter(test => test.status === 'failed').length, 0, 'original assertions failed');
const originalCases = cases.length;
const keys = ['f', 'i', 'I', 's', 'v', 'x', 'y', 'w', 'h', 'X', 'Y', 'c', 'r', 'm', 'q', 'C', 'z', 'p'];
const numeric = ['', '0', '-0', '+0', '00', '0x10', '012', '1.5', '1e2', '12suffix', '- 1', 'Infinity', '-Infinity', 'NaN', '١٢', '\ufeff-42tail', '\u00a0+15z', '9007199254740993', '900719925474099267', '9'.repeat(512)];
for (const key of keys) for (const value of numeric) cases.push(`${key}=${value}`);
cases.push('a=,o=,t=,d=', 'a=one,a=two=i,f=1,f=', 'bad,noequals,=x,aa=9,\ud800=3,a=\ud800\udfff', keys.map((key, i) => `${key}=${i}`).join(','));
const seed = 0x6b677431;
let state = seed;
function random() { state ^= state << 13; state ^= state >>> 17; state ^= state << 5; return state >>> 0; }
for (let iteration = 0; iteration < 1024; iteration++) {
  const parts = [];
  for (let index = 0, length = random() % 16; index < length; index++) {
    const key = [...keys, 'a', 'o', 't', 'd', 'unknown'][random() % (keys.length + 5)];
    const value = [...numeric, 'T', 't', 'd', 'z', 'A', '\ud800', '=', ','][random() % (numeric.length + 8)];
    parts.push(`${key}=${value}`);
  }
  cases.push(parts.join(','));
}
const units = text => Array.from({ length: text.length }, (_, i) => text.charCodeAt(i).toString(16)).join(',');
function bits(value) { const buffer = Buffer.alloc(8); buffer.writeDoubleBE(value); return buffer.toString('hex'); }
function canonical(command) {
  return Object.keys(command).sort().map(key => `${key}=${typeof command[key] === 'string' ? `s:${units(command[key])}` : `n:${bits(command[key])}`}`).join('|');
}
const corpus = cases.map(input => `${units(input)}\t${canonical(context.parse(input))}`).join('\n') + '\n';
const fixture = 'crates/xterm-addons/tests/fixtures/kitty-command.tsv';
await mkdir('crates/xterm-addons/tests/fixtures', { recursive: true });
if (process.argv.includes('--generate')) await writeFile(fixture, corpus);
else assert.equal((await readFile(fixture, 'utf8')).replace(/\r\n/g, '\n'), corpus, 'Kitty command oracle drift');
const summary = { node: process.version, platform: process.platform, seed, cases: cases.length, originalCases,
  randomizedCases: 1024, sourcePath, sourceSha256: createHash('sha256').update(source).digest('hex'),
  testPath, testSha256: createHash('sha256').update(tests).digest('hex'), outcomes,
  fixtureSha256: createHash('sha256').update(corpus).digest('hex'), status: process.argv.includes('--generate') ? 'generated' : 'matched' };
await mkdir('results/kitty-graphics', { recursive: true });
await writeFile('results/kitty-graphics/command-oracle.json', JSON.stringify(summary, null, 2) + '\n');
console.log(JSON.stringify({ ...summary, outcomes: undefined }, null, 2));
