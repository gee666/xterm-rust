// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Executes unchanged upstream assertions and records behavior.
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { stripTypeScriptTypes } from 'node:module';
import { createHash } from 'node:crypto';
import path from 'node:path';

const root = 'source_repo/src/common';
const hashes = {};
const cache = new Map();
const assertionUrl = `data:text/javascript;base64,${Buffer.from(`
import native from 'node:assert/strict';
export const assert = {...native, throws(fn, expected) {
  if (typeof expected === 'string') {
    return native.throws(fn, error => error.message.includes(expected));
  }
  return native.throws(fn, expected);
}};
`).toString('base64')}`;
async function moduleUrl(file) {
  file = path.posix.normalize(file);
  if (cache.has(file)) return cache.get(file);
  const source = await readFile(file, 'utf8');
  hashes[file] = createHash('sha256').update(source).digest('hex');
  // The upstream uses ordinary imports for interfaces. Keep runtime enums and
  // classes, but remove names erased by TypeScript before resolving dependencies.
  const runtimeSource = source.replace(/import\s*\{([^}]+)\}\s*from\s*(['"])([^'"]+)\2;/g,
    (_, names, quote, specifier) => {
      const runtime = names.split(',').map(name => name.trim()).filter(name =>
        !/^I[A-Z]/.test(name) && name !== 'CoreMouseEncoding');
      return runtime.length ? `import { ${runtime.join(', ')} } from ${quote}${specifier}${quote};` : '';
    });
  let js = stripTypeScriptTypes(runtimeSource, { mode: 'transform' });
  for (const match of [...js.matchAll(/from ['"]([^'"]+)['"]/g)]) {
    const specifier = match[1];
    const dependency = specifier === 'chai' ? assertionUrl :
      await moduleUrl(path.posix.join(path.posix.dirname(file), `${specifier}.ts`));
    js = js.replace(match[0], `from ${JSON.stringify(dependency)}`);
  }
  const url = `data:text/javascript;base64,${Buffer.from(js).toString('base64')}`;
  cache.set(file, url);
  return url;
}
const outcomes = [];
const suites = [];
globalThis.describe = (name, callback) => {
  suites.push(name);
  try { callback(); } finally { suites.pop(); }
};
globalThis.it = (name, callback) => {
  try {
    callback();
    outcomes.push({ name: [...suites, name].join(' / '), status: 'passed' });
  } catch (error) {
    outcomes.push({ name: [...suites, name].join(' / '), status: 'failed', error: String(error.stack) });
  }
};
try { await import(await moduleUrl(`${root}/services/MouseStateService.test.ts`)); }
finally { delete globalThis.describe; delete globalThis.it; }
const { MouseStateService } = await import(await moduleUrl(`${root}/services/MouseStateService.ts`));
const protocols = ['NONE', 'X10', 'VT200', 'DRAG', 'ANY'];
const encodings = ['DEFAULT', 'SGR', 'SGR_PIXELS'];
const buttons = [0, 1, 2, 3, 4, 8, 9, 10, 11, 12, 13, 14, 15];
const actions = [0, 1, 2, 3, 32];
const numbers = [-Infinity, -65568, -55328, -10272, -8225, -33, -32, -1, -0, 0, 0.5, 1, 1.5, 222, 223, 223.5, 224, 255, 65536, 1e21, Infinity, NaN];
const number = value => Object.is(value, -0) ? '-0' : String(value);
const units = value => Array.from({ length: value.length }, (_, i) => value.charCodeAt(i).toString(16)).join(',') || '-';
const rows = [];
const service = new MouseStateService();
function trace(protocol, encoding, input) {
  service.activeProtocol = protocol;
  service.activeEncoding = encoding;
  const event = { ...input };
  const accepted = service.restrictMouseEvent(event);
  // Encoding is independently callable in the source even after restriction
  // returns false. Recording both catches accidental filtering in the encoder.
  const encoded = service.encodeMouseEvent(event);
  rows.push([protocol, encoding, ...['col', 'row', 'x', 'y'].map(key => number(input[key])),
    input.button, input.action, +input.ctrl, +input.alt, +input.shift, +accepted,
    +event.ctrl, +event.alt, +event.shift, units(encoded)].join('\t'));
}
for (const protocol of protocols) for (const encoding of encodings) {
  for (const button of buttons) for (const action of actions) for (let mods = 0; mods < 8; mods++) {
    trace(protocol, encoding, { col: 1, row: 223, x: 120.5, y: 1000, button, action,
      ctrl: !!(mods & 1), alt: !!(mods & 2), shift: !!(mods & 4) });
  }
  for (let index = 0; index < numbers.length; index++) {
    const values = Array.from({ length: 4 }, (_, offset) => numbers[(index + offset) % numbers.length]);
    trace(protocol, encoding, { col: values[0], row: values[1], x: values[2], y: values[3],
      button: 0, action: 1, ctrl: true, alt: true, shift: true });
  }
}
const seed = 0x6d0a5e;
let state = seed;
function random() {
  state ^= state << 13; state ^= state >>> 17; state ^= state << 5;
  return state >>> 0;
}
for (let index = 0; index < 4096; index++) {
  trace(protocols[random() % protocols.length], encodings[random() % encodings.length], {
    col: numbers[random() % numbers.length], row: numbers[random() % numbers.length],
    x: numbers[random() % numbers.length], y: numbers[random() % numbers.length],
    button: buttons[random() % buttons.length], action: actions[random() % actions.length],
    ctrl: !!(random() & 1), alt: !!(random() & 1), shift: !!(random() & 1)
  });
}
const fixture = 'crates/xterm-core/tests/fixtures/mouse-state.tsv';
const corpus = `${rows.join('\n')}\n`;
await mkdir(path.dirname(fixture), { recursive: true });
if (process.argv.includes('--generate')) await writeFile(fixture, corpus);
else assert.equal(await readFile(fixture, 'utf8'), corpus, 'Mouse corpus differs from unchanged source');
const report = {
  node: process.version, platform: process.platform, sourceHashes: hashes, seed, traces: rows.length,
  corpusSha256: createHash('sha256').update(corpus).digest('hex'),
  schema: ['protocol', 'encoding', 'col', 'row', 'x', 'y', 'button', 'action', 'ctrl', 'alt', 'shift',
    'accepted', 'resultCtrl', 'resultAlt', 'resultShift', 'utf16HexUnits'],
  upstream: { discovered: outcomes.length, executed: outcomes.length, passed: outcomes.filter(test => test.status === 'passed').length,
    failed: outcomes.filter(test => test.status === 'failed').length, skipped: 0, unrun: 0 },
  outcomes
};
await mkdir('results/mouse-state', { recursive: true });
await writeFile('results/mouse-state/oracle.json', `${JSON.stringify(report, null, 2)}\n`);
console.log(JSON.stringify({ ...report, outcomes: undefined, sourceHashes: undefined }, null, 2));
assert.equal(outcomes.length, 8, 'upstream mouse test inventory changed');
assert.equal(report.upstream.failed, 0, 'upstream mouse assertions failed');
