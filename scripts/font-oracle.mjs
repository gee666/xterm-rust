// Run unchanged font-parser assertions against a temporary reference copy.
// Copyright (c) 2018 The xterm.js authors. MIT license for reference algorithms.
import fs from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { createHash } from 'node:crypto';
import { stripTypeScriptTypes } from 'node:module';

const root = process.cwd();
const temporary = path.join(root, 'tmp/font-oracle');
const results = path.join(root, 'results/font-addons');
const fixture = path.join(root, 'crates/xterm-addons/tests/fixtures/font-parse.tsv');
await Promise.all([fs.mkdir(temporary, { recursive: true }), fs.mkdir(results, { recursive: true }), fs.mkdir(path.dirname(fixture), { recursive: true })]);
const sourcePath = 'source_repo/addons/addon-ligatures/src/parse.ts';
const testsPath = 'source_repo/addons/addon-ligatures/src/parse.test.ts';
const source = await fs.readFile(sourcePath, 'utf8');
const tests = await fs.readFile(testsPath, 'utf8');
await fs.writeFile(path.join(temporary, 'reference.ts'), source);
await fs.writeFile(path.join(temporary, 'parse.ts'), `import reference from './reference.ts';
export default function parse(input: string): string[] {
  try { const value = reference(input); globalThis.fontCases.push({ input, value }); return value; }
  catch (error) { globalThis.fontCases.push({ input, error: error.message }); throw error; }
}
`);
await fs.writeFile(path.join(temporary, 'parse.test.ts'), tests.replace("import { assert } from 'chai';", "import assert from 'node:assert/strict';").replace("from './parse';", "from './parse.ts';"));
const outcomes = [];
globalThis.fontCases = [];
globalThis.describe = (_name, callback) => callback();
globalThis.it = (name, callback) => {
  try { callback(); outcomes.push({ name, status: 'passed' }); }
  catch (error) { outcomes.push({ name, status: 'failed', error: String(error.stack) }); }
};
await import(pathToFileURL(path.join(temporary, 'parse.test.ts')).href);
const parse = (await import(pathToFileURL(path.join(temporary, 'parse.ts')).href)).default;
const edges = ['', ',', ',,', '"last"', '"last",', 'x ,y', 'x\\', '"x\\', '\\110000', '\\D800', '\\10FFFF', '\\000020 f', 'a\uFEFFb, c', 'a\u0085b', '"a"b, c', '\\1234567'];
for (const value of edges) { try { parse(value); } catch {} }
let seed = 0xF017CA5E;
function random() { seed ^= seed << 13; seed ^= seed >>> 17; seed ^= seed << 5; return seed >>> 0; }
const alphabet = [0, 9, 10, 13, 32, 34, 39, 44, 48, 49, 54, 65, 70, 71, 92, 97, 102, 103, 0xA0, 0x85, 0xD800, 0xDC00, 0xFEFF, 0x2028];
for (let i = 0; i < 8192; i++) {
  const units = Array.from({ length: random() % 40 }, () => alphabet[random() % alphabet.length]);
  try { parse(String.fromCharCode(...units)); } catch {}
}
function hex(value) { return Array.from({ length: value.length }, (_, i) => value.charCodeAt(i).toString(16).padStart(4, '0')).join('') || '-'; }
await fs.writeFile(fixture, globalThis.fontCases.map(({ input, value, error }) => `${hex(input)}\t${error ? 'E:' + error : 'V:' + value.map(hex).join(',')}`).join('\n') + '\n');
const fallbackSourcePath = 'source_repo/addons/addon-ligatures/src/index.ts';
const addonSourcePath = 'source_repo/addons/addon-ligatures/src/LigaturesAddon.ts';
const fallbackSource = await fs.readFile(fallbackSourcePath, 'utf8');
const addonSource = await fs.readFile(addonSourcePath, 'utf8');
const fallbackCode = stripTypeScriptTypes(fallbackSource.slice(fallbackSource.indexOf('function getFallbackRanges')));
const findFallback = new Function(fallbackCode + '; return getFallbackRanges;')();
const defaultList = new Function('return [' + addonSource.match(/fallbackLigatures \?\? \[([\s\S]*?)\]\)\.sort/)[1] + '];')();
const fallbackCases = [];
function fallbackCase(text, patterns) {
  patterns.sort((a, b) => b.length - a.length);
  fallbackCases.push([text, patterns, findFallback(text, patterns)]);
}
for (const pattern of defaultList) fallbackCase('😀 ' + pattern + ' ' + pattern, [...defaultList]);
for (let i = 0; i < 4096; i++) {
  const patterns = i % 2 ? [...defaultList] : Array.from({length: random() % 8}, () => String.fromCharCode(...Array.from({length: 1 + random() % 5}, () => alphabet[random() % alphabet.length])));
  const text = i % 3 ? Array.from({length: random() % 20}, () => defaultList[random() % defaultList.length]).join('x') : String.fromCharCode(...Array.from({length: random() % 40}, () => alphabet[random() % alphabet.length]));
  fallbackCase(text, patterns);
}
const fallbackFixture = path.join(root, 'crates/xterm-addons/tests/fixtures/font-fallback.tsv');
await fs.writeFile(fallbackFixture, fallbackCases.map(([text, patterns, ranges]) => [hex(text), patterns.map(hex).join(','), ranges.map(range => range.join(':')).join(',')].join('\t')).join('\n') + '\n');
const report = {
  node: process.version,
  seed: '0xF017CA5E',
  source: [sourcePath, testsPath].map((file, i) => ({ file, sha256: createHash('sha256').update(i ? tests : source).digest('hex') })),
  original: { discovered: outcomes.length, executed: outcomes.length, passed: outcomes.filter(v => v.status === 'passed').length, failed: outcomes.filter(v => v.status === 'failed').length, skipped: 0, unrun: 0, outcomes },
  corpus: { rows: globalThis.fontCases.length, random: 8192, fixture: path.relative(root, fixture), sha256: createHash('sha256').update(await fs.readFile(fixture)).digest('hex') },
  fallback: { rows: fallbackCases.length, random: 4096, defaultPatterns: defaultList.length, fixture: path.relative(root, fallbackFixture), sources: [fallbackSourcePath, addonSourcePath].map((file, i) => ({ file, sha256: createHash('sha256').update(i ? addonSource : fallbackSource).digest('hex') })) },
  notProven: ['GSUB shaping', 'browser font access', 'ligature lifecycle', 'three-browser web-font relayout', 'native Linux/macOS']
};
await fs.writeFile(path.join(results, 'parser-oracle.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report));
if (report.original.failed) process.exitCode = 1;
