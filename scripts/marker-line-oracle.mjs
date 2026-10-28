// Copyright (c) 2018-2026 The xterm.js authors. All rights reserved. MIT license.
// Reference-only primitive operators used by Marker/Buffer/DecorationService.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { runScopedOracle } from './oracle-evidence.mjs';

const root = resolve(import.meta.dirname, '..'); process.chdir(root);
const report = await runScopedOracle(resolve('results/marker-line-reference'), {
  kind: 'marker-line-reference', command: 'node scripts/marker-line-oracle.mjs',
  scope: 'JavaScript Number/String operators only; Rust replay and marker/browser integration are separate', native: 'unrun'
}, async () => {
  const hex = text => text.length ? Array.from({ length: text.length }, (_, i) => text.charCodeAt(i).toString(16).padStart(4, '0')).join('') : '_';
  const number = value => Object.is(value, -0) ? '-0' : String(value);
  const encode = value => typeof value === 'string' ? `s${hex(value)}` : `n${number(value)}`;
  function snapshot(left, right) {
    let incremented = left; incremented++;
    const map = new Map([[left, true]]);
    const flags = [left === right, left < right, left > right, left <= right, left >= right, map.has(right)].map(Number).join('');
    return [encode(+left), hex(String(left)), encode(left + right), encode(left - right), encode(incremented), flags, encode(map.keys().next().value)].join('|');
  }
  const values = [0, -0, 1, -1, .5, -.5, 2, 10, NaN, Infinity, -Infinity, Number.MIN_VALUE, Number.MAX_VALUE, 1e-7, 1e21, 2 ** 53, 2 ** 32 - 1,
    '', '0', '01', '2', '10', '-0', '0x10', '0b101', '0o77', 'Infinity', 'NaN', 'foo', ' ', '\ufeff1', '\u00851', '\ud800', '\ue000', '😀', '0foo', '0.5', '1e1000', '+0x1'];
  const rows = [];
  function add(left, right) { rows.push(`${encode(left)}\t${encode(right)}\t${snapshot(left, right)}`); }
  for (const left of values) for (const right of values) add(left, right);
  const directed = rows.length;
  const seed = 0x4d4c494e; let state = seed;
  function random() { state ^= state << 13; state ^= state >>> 17; state ^= state << 5; return state >>> 0; }
  const bytes = new DataView(new ArrayBuffer(8));
  const alphabet = [...'0123456789abcdefABCDEFxboeE.+- Infinity', '\ud800', '\udfff', '\ufeff', '\u0085'];
  function value() {
    if (random() & 1) { bytes.setUint32(0, random(), true); bytes.setUint32(4, random(), true); return bytes.getFloat64(0, true); }
    const length = random() % 13; let text = '';
    for (let i = 0; i < length; i++) text += alphabet[random() % alphabet.length];
    return text;
  }
  for (let i = 0; i < 4096; i++) add(value(), value());
  const fixture = rows.join('\n') + '\n';
  const path = 'crates/xterm-core/tests/fixtures/marker-line.tsv';
  if (process.argv.includes('--write-fixtures')) writeFileSync(path, fixture);
  else assert.equal(readFileSync(path, 'utf8').replaceAll('\r\n', '\n'), fixture);
  const sources = {};
  for (const path of ['source_repo/src/common/buffer/Marker.ts', 'source_repo/src/common/buffer/Buffer.ts', 'source_repo/src/common/services/DecorationService.ts']) sources[path] = createHash('sha256').update(readFileSync(path)).digest('hex');
  return { status: 'passed', native: 'unrun', scope: 'JavaScript primitive reference only, not complete marker or browser equivalence',
    seed, directed, randomized: 4096, cases: rows.length, fixtureSha256: createHash('sha256').update(fixture).digest('hex'), sources,
    operators: ['ToNumber', 'ToString', '+', '-', '++', '===', '<', '>', '<=', '>=', 'Map SameValueZero/key normalization'] };
});
console.log(JSON.stringify(report, null, 2));
