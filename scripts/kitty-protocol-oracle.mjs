// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
// Protocol/state oracle. Only the platform display boundary is deferred/recorded;
// no browser pixel or live-addon equivalence is claimed by these traces.
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
import vm from 'node:vm';
import { eraseModuleBoundaries } from './parser-oracle-common.mjs';
const require = createRequire(import.meta.url);
const Base64Decoder = require('../source_repo/node_modules/xterm-wasm-parts/lib/base64/Base64Decoder.wasm.js').default;
const context = vm.createContext({ Blob, Base64Decoder });
const mutant = process.argv.find(arg => arg.startsWith('--mutant='))?.slice('--mutant='.length);
assert.ok(!mutant || ['quiet', 'first-command', 'control-boundary'].includes(mutant), 'unknown source mutant');
assert.ok(!(mutant && process.argv.includes('--generate')), 'never generate committed expectations from a mutant');
const sources = [];
for (const [file, exports] of [
  ['KittyGraphicsTypes', 'parseKittyCommand, KittyAction, KittyFormat, KittyCompression, KittyPixelConstants'],
  ['KittyImageStorage', 'KittyImageStorage'], ['KittyGraphicsHandler', 'KittyGraphicsHandler'],
]) {
  const path = `source_repo/addons/addon-image/src/kitty/${file}.ts`;
  const source = await readFile(path, 'utf8');
  sources.push({ path, sha256: createHash('sha256').update(source).digest('hex') });
  let body = source;
  if (file === 'KittyGraphicsHandler' && mutant) {
    const edits = { quiet: ['isOk && quiet >= 1', 'isOk && quiet > 1'],
      'first-command': ['finalCmd = pending.cmd;', 'finalCmd = cmd;'],
      'control-boundary': ['this._controlLength + copyLength > Constants.MAX_CONTROL_DATA_SIZE', 'this._controlLength + copyLength >= Constants.MAX_CONTROL_DATA_SIZE'] };
    const [before, after] = edits[mutant];
    assert.equal(body.split(before).length, 2, `source mutation anchor ${mutant}`);
    body = body.replace(before, after);
  }
  const withoutImports = body.replace(/^import\s[\s\S]*?from\s+['"][^'"]+['"];\r?$/gm, '');
  vm.runInContext(`Object.assign(globalThis, (() => {\n${eraseModuleBoundaries(withoutImports)}\nreturn {${exports}};\n})());`, context);
}
vm.runInContext('globalThis.P = Promise;', context);
const units = text => Array.from({ length: text.length }, (_, i) => text.charCodeAt(i));
const points = text => Array.from(text, character => character.codePointAt(0));
function number(value) {
  if (value === undefined) return null;
  const bytes = Buffer.alloc(8); bytes.writeDoubleBE(value); return bytes.toString('hex');
}
function command(value) {
  if (!value) return null;
  return Object.fromEntries(Object.keys(value).sort().map(key => [key,
    typeof value[key] === 'string' ? { s: units(value[key]) } : { n: number(value[key]) }]));
}
const decoder = value => value ? { loaded: value.loadedBytes, free: String(value.freeBytes), data: Array.from(value.data8) } : null;
const rows = [];
let operationCount = 0;
async function trace(name, limit, operations) {
  const events = [], jobs = [], promises = [];
  let throwing = false;
  const shared = { onImageDeleted: undefined, addImage() { throw new Error('display storage is a separate oracle'); }, deleteImage(id) { this.onImageDeleted?.(id); } };
  const storage = new context.KittyImageStorage(shared);
  const handler = new context.KittyGraphicsHandler({ kittySizeLimit: limit, pixelLimit: 16777216 }, {}, storage,
    { _core: { coreService: { triggerDataEvent(data) { events.push(['reply', units(data)]); if (throwing) throw new Error('reply failure'); } } } });
  handler._displayImage = (image, cmd) => {
    events.push(['display', number(image.id), command(cmd)]);
    return new context.P(resolve => jobs.push({ resolve, image, command: cmd }));
  };
  const snapshots = [];
  async function snapshot() {
    const images = [];
    for (const [key, image] of handler.images) images.push({ key: number(key), id: number(image.id),
      width: number(image.width), height: number(image.height), format: number(image.format), compression: units(image.compression ?? ''),
      mime: image.data.type, data: Array.from(new Uint8Array(await image.data.arrayBuffer())) });
    const pending = Array.from(handler.pendingTransmissions, ([key, value]) => ({ key: number(key), command: command(value.cmd),
      total: number(value.totalEncodedSize), error: value.decodeError, decoder: decoder(value.decoder) }));
    return { aborted: handler._aborted, decodeError: handler._decodeError, inControl: handler._inControlData,
      control: Array.from(handler._controlData.slice(0, handler._controlLength)), parsed: command(handler._parsedCommand),
      limit: number(handler._encodedSizeLimit), total: number(handler._totalEncodedSize), last: number(handler._lastPendingKey),
      active: decoder(handler._activeDecoder), pending, images, placements: Array.from(handler._kittyIdToStorageId, ([id, storage]) => [number(id), storage]),
      lastImageId: number(storage.lastImageId), events: structuredClone(events) };
  }
  for (const operation of operations) {
    operationCount++;
    try {
      switch (operation[0]) {
        case 's': handler.start(); break;
        case 'p': handler.put(Uint32Array.from(operation[1]), 0, operation[1].length); break;
        case 'e': {
          const result = handler.end(operation[1]);
          if (result && typeof result.then === 'function') { promises.push(result); events.push(['end', 'async']); }
          else events.push(['end', 'sync', result]);
          break;
        }
        case 'r': {
          const job = jobs.shift(), result = promises.shift();
          if (!job) { assert.equal(result, undefined); events.push(['resolve', 'none']); break; }
          job.resolve(operation[1]); events.push(['resolved', await result]); break;
        }
        case 'x': handler.reset(); break;
        case 'd': handler.dispose(); break;
        case 'throw': throwing = operation[1]; break;
        case 'pq': handler.pendingTransmissions.get(operation[1]).cmd.quiet = operation[2]; break;
        case 'pt': handler.pendingTransmissions.get(operation[1]).totalEncodedSize = operation[2]; break;
        case 'pe': handler.pendingTransmissions.get(operation[1]).decodeError = operation[2]; break;
        case 'pr': handler.pendingTransmissions.get(operation[1]).decoder.release(); break;
        case 'jc': jobs[0].command[operation[1]] = operation[2]; break;
        default: throw new Error(`unknown operation ${operation[0]}`);
      }
    } catch (error) { events.push(['error', error.name, error.message]); }
    snapshots.push(await snapshot());
  }
  assert.equal(jobs.length, 0, `${name}: unconsumed display job`);
  assert.equal(promises.length, 0, `${name}: unconsumed result`);
  rows.push({ name, limit, operations, snapshots });
}
function transaction(control, payload, width = 0) {
  const data = points(payload === null ? control : `${control};${payload}`);
  const ops = [['s']];
  for (let offset = 0; offset < data.length; offset += width || data.length) ops.push(['p', data.slice(offset, offset + (width || data.length))]);
  return [...ops, ['e', true], ['r', true]];
}
const cases = [
  ['a=t,i=1,f=32,s=1,v=1', 'AAAAAA=='], ['i=0,f=24,s=1,v=1', 'AAAA'], ['a=,i=2', 'QQ=='],
  ['a=t,i=9', ''], ['a=t,i=9', null], ['a=q,i=3', null], ['a=q,i=3,t=f', null], ['a=q,i=3,t=f', ''],
  ['a=q,i=3,f=24,s=-1,v=1', 'QQ=='], ['a=q,i=3,f=0,s=1,v=1', 'QQ=='], ['a=q,i=3,f=100', 'QQ=='],
  ['a=q,i=3,s=,v=1', 'QQ=='], ['a=q', '!!!!'], ['a=T,i=3,f=32,s=1,v=1', 'AAAAAA=='],
  ['a=T,f=32,s=1,v=1', 'AAAAAA=='], ['a=p,i=4,p=9', null], ['a=p', null], ['a=d,d=z,i=1', null],
  ['a=d', null], ['a=t,i=1,I=2', 'QQ=='], ['a=q,i=1,I=2', null], ['a=t,i=foo', 'QQ=='],
  ['a=t,i=-0', 'QQ=='], ['a=t,i=1,q=1', 'QQ=='], ['a=q,i=1,q=1', '!!!!'], ['a=q,i=1,q=2', '!!!!'],
  ['a=t,i=1,o=z', 'QQ=='], ['a=t,i=5', 'QQ'],
];
for (let index = 0; index < cases.length; index++) for (const width of [0, 1, 3]) await trace(`command-${index}-width-${width}`, 64, transaction(...cases[index], width));
await trace('unsupported-display-reuses-old', 64, [...transaction('a=t,i=1,f=32,s=1,v=1', 'AAAAAA=='), ...transaction('a=T,i=1,t=f', 'QQ==')]);
for (const dispose of ['x', 'd']) await trace(`pending-${dispose}`, 64, [...transaction('a=T,i=1', 'QQ==').slice(0, -1), [dispose], ['r', true], ...transaction('a=q,i=2', null)]);
await trace('late-quiet-read', 64, [...transaction('a=T,i=1', 'QQ==').slice(0, -1), ['jc', 'quiet', 2], ['r', true]]);
await trace('late-id-not-new-response', 64, [...transaction('a=T', 'QQ==').slice(0, -1), ['jc', 'id', 5], ['r', true]]);
await trace('display-failure', 64, [...transaction('a=T,i=1,p=2', 'QQ==').slice(0, -1), ['r', false], ...transaction('a=p,i=1,p=-2', null).slice(0, -1), ['r', false]]);
await trace('first-metadata-wins', 64, [...transaction('a=t,i=1,f=32,s=1,v=1,m=1,q=2', 'AAAA'), ...transaction('m=0,q=0', 'AA==')]);
await trace('mutable-pending', 64, [...transaction('a=t,i=1,m=1', 'QQ'), ['pq', 1, 1], ['pt', 1, -100], ...transaction('m=0', '==')]);
await trace('pending-error-flag', 64, [...transaction('a=t,i=1,m=1', 'QQ'), ['pe', 1, true], ...transaction('m=0', '==')]);
await trace('released-pending-decoder', 64, [...transaction('a=t,i=1,m=1', 'QQ'), ['pr', 1], ...transaction('m=0', 'QQ==')]);
await trace('interleaved-last-key', 64, [...transaction('a=t,i=1,m=1', 'QQ'), ...transaction('a=t,i=2,m=1', 'Qg'), ...transaction('i=1,m=0', '=='), ...transaction('m=0', '=='), ...transaction('a=d,d=i,i=2', null)]);
for (const length of [512, 513]) for (const width of [0, 1]) await trace(`control-${length}-${width}`, 64, transaction('x'.repeat(length), 'QQ==', width));
for (const limit of [0, 1, 2, 3, 4, 8]) await trace(`limit-${limit}`, limit, [...transaction('a=t,i=1,m=1', 'QQ'), ...transaction('m=0', '=='), ...transaction('a=q,i=2', null)]);
await trace('reply-exception-pre-abort', 64, [['throw', true], ['s'], ['p', points('a=q,i=1,I=2;')], ['throw', false], ['e', true], ['r', true]]);
await trace('aborted-transfer', 64, [['s'], ['p', points('a=t,i=1;QQ')], ['e', false], ...transaction('a=t,i=2', 'Qg==')]);
const seed = 0x6b677032;
let randomState = seed;
function random() { randomState ^= randomState << 13; randomState ^= randomState >>> 17; randomState ^= randomState << 5; return randomState >>> 0; }
for (let index = 0; index < 128; index++) {
  const ops = [];
  for (let round = 0; round < 4; round++) {
    const item = cases[random() % cases.length];
    const tx = transaction(...item, [0, 1, 3][random() % 3]);
    if (random() % 7 === 0) tx.splice(tx.length - 1, 0, ['x']);
    if (random() % 3 === 0) tx[tx.length - 1] = ['r', false];
    ops.push(...tx);
  }
  await trace(`seed-${index}`, [8, 16, 64][random() % 3], ops);
}
const fixture = 'crates/xterm-addons/tests/fixtures/kitty-protocol.json';
const corpus = JSON.stringify(rows) + '\n';
assert.ok(Buffer.byteLength(corpus) < 20 * 1024 * 1024, 'source fixture must stay within the tiny-runner budget');
await mkdir('crates/xterm-addons/tests/fixtures', { recursive: true });
let counterexample;
if (mutant) {
  const expected = JSON.parse(await readFile(fixture, 'utf8'));
  for (let row = 0; row < rows.length && !counterexample; row++) for (let step = 0; step < rows[row].snapshots.length; step++) {
    if (JSON.stringify(rows[row].snapshots[step]) !== JSON.stringify(expected[row].snapshots[step])) {
      counterexample = { trace: rows[row].name, step, operation: rows[row].operations[step], expected: expected[row].snapshots[step], actual: rows[row].snapshots[step] };
      break;
    }
  }
  assert.ok(counterexample, `oracle failed to reject ${mutant}`);
} else if (process.argv.includes('--generate')) await writeFile(fixture, corpus);
else assert.equal((await readFile(fixture, 'utf8')).replace(/\r\n/g, '\n'), corpus, 'Kitty protocol oracle drift');
const report = { node: process.version, platform: process.platform, seed, traces: rows.length, operations: operationCount, sources,
  fixtureBytes: Buffer.byteLength(corpus), fixtureSha256: createHash('sha256').update(corpus).digest('hex'),
  mutant: mutant ?? null, counterexample,
  status: mutant ? 'source-mutant-rejected' : process.argv.includes('--generate') ? 'generated' : 'matched',
  scope: 'original native protocol/state/reply logic with recorded deferred display boundary; not original browser suite or live pixels' };
await mkdir('results/kitty-graphics', { recursive: true });
await writeFile(`results/kitty-graphics/${mutant ? `protocol-negative-${mutant}` : 'protocol-oracle'}.json`, JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ ...report, counterexample: counterexample ? { trace: counterexample.trace, step: counterexample.step, operation: counterexample.operation } : undefined }, null, 2));
