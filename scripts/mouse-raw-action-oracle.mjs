// Copyright (c) 2019, 2026 The xterm.js authors. All rights reserved. MIT license.
// Uses the shared unchanged-source loader; no reference edits or copied policies.
import assert from 'node:assert/strict';
import {stripTypeScriptTypes} from 'node:module';
import vm from 'node:vm';
import {readFileSync, writeFileSync, mkdirSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {fileURLToPath} from 'node:url';
import path from 'node:path';
import {loadMouseReference} from './mouse-reference-module.mjs';

const args = process.argv.slice(2);
assert(args.length === 0 || (args.length === 1 && args[0] === '--generate'), 'Usage: node scripts/mouse-raw-action-oracle.mjs [--generate]');
const root = fileURLToPath(new URL('../', import.meta.url));
const reference = loadMouseReference();
const service = new reference.MouseStateService();
const protocols = ['NONE', 'X10', 'VT200', 'DRAG', 'ANY'];
const encodings = ['DEFAULT', 'SGR', 'SGR_PIXELS'];
const buttons = [0, 3, 4, 8];
const actions = [-Infinity, -4294967297, -4294967296, -2147483649, -2147483648,
  -65536, -33, -1, -0, 0, 0.5, 1, 1.5, 2, 2.5, 3, 3.5, 4, 5, 31, 31.5,
  32, 32.5, 33, 255, 256, 2147483647, 2147483648, 4294967295, 4294967296,
  4294967297, Number.MAX_SAFE_INTEGER, Infinity, NaN, 1e100, -1e100];
const number = value => Object.is(value, -0) ? '-0' : String(value);
const units = value => Array.from({length:value.length}, (_, i) => value.charCodeAt(i).toString(16)).join(',') || '-';
const rows = [];
function trace(protocol, encoding, button, action, flags) {
  service.activeProtocol = protocol;
  service.activeEncoding = encoding;
  const event = {col:1, row:223, x:120.5, y:1000, button, action,
    ctrl:!!(flags & 1), alt:!!(flags & 2), shift:!!(flags & 4)};
  const original = {...event};
  const accepted = service.restrictMouseEvent(event);
  const encoded = service.encodeMouseEvent(event);
  assert(Object.is(event.action, action), 'source restriction/encoding mutated action');
  rows.push([protocol, encoding, original.col, original.row, original.x, original.y,
    button, number(action), +original.ctrl, +original.alt, +original.shift, +accepted,
    +event.ctrl, +event.alt, +event.shift, units(encoded)].join('\t'));
}
for (const protocol of protocols) for (const encoding of encodings) {
  for (const button of buttons) for (const action of actions) for (let flags = 0; flags < 8; flags++) {
    trace(protocol, encoding, button, action, flags);
  }
}
const deterministicRows = rows.length;
const seed = 0x524ac710;
let state = seed;
function random() {state ^= state << 13; state ^= state >>> 17; state ^= state << 5; return state >>> 0;}
for (let i = 0; i < 2048; i++) {
  // Fractions and uint32 wrap boundaries distinguish strict checks from ToInt32.
  const action = (random() - 2147483648) * (i % 2 ? 0.5 : 4);
  trace(protocols[random() % protocols.length], encodings[random() % encodings.length],
    buttons[random() % buttons.length], action, random() & 7);
}
// Execute unchanged trigger/equality method bodies. Constructor/DOM setup is
// irrelevant to the already-normalized report path and is not simulated here.
const mouseFile = 'source_repo/src/browser/services/MouseService.ts';
const typesFile = 'source_repo/src/common/Types.ts';
const mouseBytes = readFileSync(path.join(root, mouseFile));
const typesBytes = readFileSync(path.join(root, typesFile));
const source = mouseBytes.toString('utf8');
const start = source.indexOf('  private _triggerMouseEvent(');
const end = source.indexOf('\n}', start);
assert(start >= 0 && end > start, 'source trigger method boundaries changed');
const declarations = ['CoreMouseButton', 'CoreMouseAction', 'CoreMouseEventType'].map(name => {
  const match = typesBytes.toString('utf8').match(new RegExp('export const enum ' + name + '\\s*\\{[^}]*\\}'));
  assert(match, `${name} declaration missing`);
  return match[0].replace('export ', '');
}).join('\n');
const script = stripTypeScriptTypes(declarations + '\nclass MouseProbe {\n' + source.slice(start, end) + '\n}', {mode:'transform'});
const context = vm.createContext({});
vm.runInContext(script, context);
const Probe = vm.runInContext('MouseProbe', context);
const routingRows = [];
for (const protocol of protocols) for (const encoding of encodings) {
  service.activeProtocol = protocol; service.activeEncoding = encoding;
  for (const button of buttons) for (const action of actions) {
    const probe = new Probe();
    const outputs = [];
    probe._mouseStateService = service;
    probe._bufferService = {cols:80, rows:24};
    probe._coreService = {triggerBinaryEvent: value => outputs.push(['binary', value]),
      triggerDataEvent: (value, user) => {assert.equal(user, true); outputs.push(['data', value]);}};
    const event = {col:0, row:0, x:120.5, y:1000, button, action, ctrl:true, alt:true, shift:true};
    const accepted = probe._triggerMouseEvent(event);
    assert(Object.is(event.action, action), 'source routing mutated action');
    assert(outputs.length <= 1);
    routingRows.push([protocol, encoding, button, number(action), +accepted, event.col, event.row,
      +event.ctrl, +event.alt, +event.shift, outputs[0]?.[0] ?? '-', units(outputs[0]?.[1] ?? '')].join('\t'));
  }
}
const routingFixture = 'crates/xterm-web/tests/fixtures/mouse-raw-actions.tsv';
const routingCorpus = routingRows.join('\n') + '\n';
if (args.includes('--generate')) writeFileSync(path.join(root, routingFixture), routingCorpus);
else assert.equal(readFileSync(path.join(root, routingFixture), 'utf8').replaceAll('\r\n', '\n'), routingCorpus, 'raw action routing fixture drift');
const custom = new reference.MouseStateService();
const observed = [];
custom.addProtocol('RAW', {events:1, restrict(event) {
  observed.push(event.action); event.action = -0; return true;
}});
custom.addEncoding('RAW', event => {observed.push(event.action); event.action = 1.5; return '\ud800';});
custom.activeProtocol = 'RAW'; custom.activeEncoding = 'RAW';
const customEvent = {action:NaN};
assert.equal(custom.restrictMouseEvent(customEvent), true);
assert.equal(units(custom.encodeMouseEvent(customEvent)), 'd800');
assert(Number.isNaN(observed[0]) && Object.is(observed[1], -0));
assert.equal(customEvent.action, 1.5);
const fixture = 'crates/xterm-core/tests/fixtures/mouse-raw-actions.tsv';
const corpus = rows.join('\n') + '\n';
if (args.includes('--generate')) writeFileSync(path.join(root, fixture), corpus);
else assert.equal(readFileSync(path.join(root, fixture), 'utf8').replaceAll('\r\n', '\n'), corpus, 'raw action fixture drift');
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const output = path.join(root, 'results/mouse-raw-actions');
mkdirSync(output, {recursive:true});
writeFileSync(path.join(output, 'source.json'), JSON.stringify({status:'passed', node:process.version,
  platform:process.platform, esbuild:reference.esbuild, sourceHashes:reference.sourceHashes,
  moduleSha256:reference.moduleSha256,
  routingSourceHashes:{[mouseFile]:hash(mouseBytes), [typesFile]:hash(typesBytes)},
  routingRows:routingRows.length, routingFixture, routingCanonicalSha256:hash(routingCorpus),
  actions:actions.map(number), buttons, seed,
  deterministicRows, seededRows:2048, rows:rows.length, fixture, canonicalSha256:hash(corpus),
  directedCustomAssertions:1,
  scope:'Numeric action representation, builtin policies and normalized report routing only. Raw registries, nonnumeric actions and real DOM/browser routing are not established.'}, null, 2) + '\n');
console.log(JSON.stringify({status:'passed', deterministicRows, seededRows:2048, rows:rows.length, seed}));
