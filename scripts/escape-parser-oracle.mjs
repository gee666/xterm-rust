// Copyright (c) 2018 The xterm.js authors. All rights reserved.
// Licensed under MIT. Compare full escape-parser traces from unchanged sources.
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
// Use the reference's build dependency, not a production Rust dependency. Bundle
// directly from TypeScript so stale out-esbuild files cannot become the oracle.
const esbuild = require('../source_repo/node_modules/esbuild');
const build = esbuild.buildSync({
  entryPoints: ['source_repo/src/common/parser/EscapeSequenceParser.ts'],
  bundle: true, write: false, format: 'esm', platform: 'node', metafile: true,
});
const { EscapeSequenceParser, TransitionTable } = await import(`data:text/javascript;base64,${Buffer.from(build.outputFiles[0].contents).toString('base64')}`);
// A supported short custom table exercises JavaScript undefined-to-zero access.
const shortTable = new EscapeSequenceParser(new TransitionTable(10));
shortTable.currentState = 1;
shortTable.precedingJoinState = 123;
let printed = false;
shortTable.setPrintHandler(() => { printed = true; });
assert.equal(shortTable.parse(Uint32Array.of(65), 1), undefined);
assert.equal(shortTable.currentState, 0);
assert.equal(shortTable.precedingJoinState, 123);
assert.equal(printed, false);
const sourcePaths = Object.keys(build.metafile.inputs).sort();
const sources = await Promise.all(sourcePaths.map(async path => ({ path, sha256: createHash('sha256').update(await readFile(path)).digest('hex') })));
const hex = values => Array.from(values, value => value.toString(16)).join(',');
const units = text => hex(Array.from({ length: text.length }, (_, i) => text.charCodeAt(i)));
const params = value => value.toArray().map(v => Array.isArray(v) ? `s${v.join(',')}` : `p${v}`).join(':');
const fixtures = [];
let writes = 0;
function trace(operations) {
  const parser = new EscapeSequenceParser();
  const reports = [];
  parser.setPrintHandler((data, start, end) => reports.push(`P:${hex(data.slice(start, end))}`));
  parser.setExecuteHandlerFallback(code => reports.push(`X:${code}`));
  parser.setCsiHandlerFallback((id, value) => reports.push(`C:${id >>> 0}:${params(value)}`));
  parser.setEscHandlerFallback(id => reports.push(`E:${id >>> 0}`));
  parser.setErrorHandler(state => { reports.push(`R:${state.position}:${state.code}:${state.currentState}:${state.collect >>> 0}:${params(state.params)}`); return state; });
  parser.setOscHandlerFallback((id, action, value) => reports.push(`O:${id}:${action === 'START' ? 'S' : action === 'PUT' ? `P:${units(value)}` : `E:${+value}`}`));
  parser.setDcsHandlerFallback((id, action, value) => reports.push(`D:${id >>> 0}:${action === 'HOOK' ? `H:${params(value)}` : action === 'PUT' ? `P:${units(value)}` : `U:${+value}`}`));
  parser.setApcHandlerFallback((id, action, value) => reports.push(`A:${id >>> 0}:${action === 'START' ? 'S' : action === 'PUT' ? `P:${units(value)}` : `E:${+value}`}`));
  const snapshots = [];
  for (const operation of operations) {
    reports.length = 0;
    if (operation === 'r') parser.reset();
    else if (operation[0] === 's') parser.currentState = Number(operation.slice(1));
    else if (operation[0] === 'j') parser.precedingJoinState = Number(operation.slice(1));
    else {
      const data = Uint32Array.from(operation.slice(1).split(',').filter(Boolean).map(value => parseInt(value, 16)));
      assert.equal(parser.parse(data, data.length), undefined);
      writes++;
    }
    snapshots.push(`${parser.currentState}/${parser._collect >>> 0}/${params(parser._params)}/${parser.precedingJoinState}/${reports.join('|')}`);
  }
  fixtures.push(`${operations.join('|')}\t${snapshots.join('~')}`);
}
// Exhaust all byte classes in every reachable enum state, plus wide codepoints.
for (let state = 0; state < 17; state++) {
  for (const code of [...Array.from({ length: 161 }, (_, i) => i), 0x20ac, 0xd800, 0x10000, 0x10ffff, 0xffffffff]) {
    trace([`s${state}`, 'j123', `p${code.toString(16)}`]);
  }
}
const matrixCases = fixtures.length;
const seed = 0xe5ca9e01;
let state = seed;
function next() { state ^= state << 13; state ^= state >>> 17; state ^= state << 5; return state >>> 0; }
const strings = [
  'abc ö€𝄞', '\x1b[31;5::123:;8m', '\x1b[?1;25h', '\x1b[>4;2m', '\x1b[1 q', '\x1b]0;hello€\x07',
  '\x1b]52;c;SGVsbG8=\x1b\\', '\x1bP1;2::55;3+$abc;€\x9c', '\x1bP$qabc\x18', '\x1b_ GXhello\x1b\\',
  '\x1b_Xa\t\n\r\x7fb\x9c', '\x9d123;xyz\x1a', '\x9b<;c', '\x98ignored\x9c', '\x1b(0', '\x1b%G',
  '\x1b[1€abc\x1b[0m', '\x1b[' + '9'.repeat(64) + 'm', '\x1b[' + ';'.repeat(40) + ':'.repeat(40) + 'm',
  '\0\x08\t\n\r\x18\x1a', '\x1bPqA\x7fB\x1b\\', '\x1b[\x1b[31m',
];
const chunks = strings.map(text => Array.from(text, character => character.codePointAt(0)));
const randomizedCases = 512;
for (let iteration = 0; iteration < randomizedCases; iteration++) {
  const operations = [];
  for (let round = 0; round < 32; round++) {
    if (next() % 7 === 0) operations.push('r');
    if (next() % 7 === 0) operations.push(`j${next()}`);
    const data = next() % 5 ? chunks[next() % chunks.length] : Array.from({ length: next() % 24 }, () => [next() % 256, 0xd800, 0x10000, 0xffffffff][next() % 4]);
    if (iteration % 4 === 0) { operations.push(`p${hex(data)}`); continue; }
    for (let offset = 0; offset < data.length;) {
      const width = iteration % 4 === 1 ? 1 : 1 + next() % 9;
      operations.push(`p${hex(data.slice(offset, offset + width))}`); offset += width;
    }
  }
  trace(operations);
}
const corpus = fixtures.join('\n') + '\n';
const fixturePath = 'crates/xterm-core/tests/fixtures/escape-parser.tsv';
if (process.argv.includes('--generate')) {
  await mkdir('crates/xterm-core/tests/fixtures', { recursive: true });
  await writeFile(fixturePath, corpus);
} else assert.equal((await readFile(fixturePath, 'utf8')).replace(/\r\n/g, '\n'), corpus, 'escape parser fixture differs from unchanged JS source');
const summary = { node: process.version, platform: process.platform, esbuild: esbuild.version, sources,
  seed, matrixCases, randomizedCases, parseCalls: writes, fixtures: fixtures.length, customTableAssertions: 4,
  fixtureSha256: createHash('sha256').update(corpus).digest('hex'), fixtureBytes: Buffer.byteLength(corpus),
  status: process.argv.includes('--generate') ? 'generated' : 'matched',
  scope: 'full state, collect, Params/subparams, precedingJoinState, fallback/error callback order and boundaries; registrations/async have separate tests' };
await mkdir('results/escape-parser', { recursive: true });
await writeFile('results/escape-parser/oracle.json', JSON.stringify(summary, null, 2) + '\n');
console.log(JSON.stringify(summary, null, 2));
