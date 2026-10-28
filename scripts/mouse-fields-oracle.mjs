// Copyright (c) 2019-2026 The xterm.js authors. All rights reserved. MIT license.
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { loadMouseReference } from './mouse-reference-module.mjs';
import { scenarios, observe } from '../crates/xterm-wasm-support/tests/mouse_fields_cases.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
const reference = loadMouseReference();
const service = new reference.MouseStateService();
const fixtures = scenarios().map(spec => {
  service.activeProtocol = spec.protocol;
  service.activeEncoding = spec.encoding;
  return { spec, expected: observe(event => service.restrictMouseEvent(event),
    event => service.encodeMouseEvent(event), spec) };
});
assert.equal(fixtures.length, 1125);
const text = JSON.stringify(fixtures) + '\n';
const fixture = path.join(root, 'crates/xterm-wasm-support/tests/reference/mouse-fields.json');
if (process.argv.includes('--generate')) writeFileSync(fixture, text);
else assert.equal(readFileSync(fixture, 'utf8').replaceAll('\r\n', '\n'), text, 'Raw mouse field fixture drift');
const directory = path.join(root, 'results/mouse-fields');
mkdirSync(directory, { recursive: true });
writeFileSync(path.join(directory, 'source.json'), JSON.stringify({ cases: fixtures.length,
  sourceHashes: reference.sourceHashes, moduleSha256: reference.moduleSha256,
  node: process.version, platform: process.platform,
  scope: 'Original-source builtin raw field reads/operators only. Rust execution is recorded separately.' }, null, 2) + '\n');
console.log(`${fixtures.length} unchanged-source mouse field cases verified`);
