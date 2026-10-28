// Copyright (c) 2019, 2026 The xterm.js authors. All rights reserved. MIT license.
// Builds the original source in memory; no reference source or assertion edits.
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { loadMouseReference } from './mouse-reference-module.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
const reference = loadMouseReference();
const { MouseStateService } = reference;
const service = new MouseStateService();
const buttons = [-1, -2, -32768, 32767, -2147483648, 2147483647, 4];
const rows = [];
for (const protocol of ['NONE', 'X10', 'VT200', 'DRAG', 'ANY']) {
  service.activeProtocol = protocol;
  for (const encoding of ['DEFAULT', 'SGR', 'SGR_PIXELS']) {
    service.activeEncoding = encoding;
    for (const button of buttons) for (const action of [0, 1, 2, 3, 32]) {
      for (let flags = 0; flags < 8; flags++) {
        const event = { col: 1, row: 223, x: 120.5, y: 1000, button, action,
          ctrl: !!(flags & 1), alt: !!(flags & 2), shift: !!(flags & 4) };
        const original = { ...event };
        const accepted = service.restrictMouseEvent(event);
        const output = service.encodeMouseEvent(event);
        const units = Array.from({ length: output.length }, (_, i) => output.charCodeAt(i).toString(16)).join(',') || '-';
        rows.push([protocol, encoding, original.col, original.row, original.x, original.y, button, action,
          +original.ctrl, +original.alt, +original.shift, +accepted, +event.ctrl, +event.alt, +event.shift, units].join('\t'));
      }
    }
  }
}
const fixture = 'crates/xterm-core/tests/fixtures/mouse-raw-buttons.tsv';
const text = rows.join('\n') + '\n';
if (process.argv.includes('--generate')) writeFileSync(path.join(root, fixture), text);
else if (readFileSync(path.join(root, fixture), 'utf8').replaceAll('\r\n', '\n') !== text) throw new Error('raw mouse fixture drift');
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
const directory = path.join(root, 'results/mouse-state');
mkdirSync(directory, { recursive: true });
writeFileSync(path.join(directory, 'raw-buttons-oracle.json'), JSON.stringify({
  node: process.version, platform: process.platform, esbuild: reference.esbuild,
  sourceHashes: reference.sourceHashes, buttons, rows: rows.length, fixture,
  fixtureSha256: sha256(readFileSync(path.join(root, fixture))), canonicalSha256: sha256(text),
  scope: 'Raw signed/bitwise mouse button compatibility only; not browser routing or terminal equivalence.'
}, null, 2) + '\n');
console.log(JSON.stringify({ rows: rows.length, buttons, status: 'passed unchanged-source corpus check' }));
