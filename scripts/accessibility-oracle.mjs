// Copyright (c) 2017-2018 The xterm.js authors. All rights reserved.
// Licensed under MIT. Unchanged announcement methods, not a fake browser pass.
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { canonicalTsvRecords } from './tsv-records.mjs';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const require = createRequire(import.meta.url);
// This is the reference's existing build dependency, not a new production one.
const sourceRoot = path.resolve(process.env.XT_SOURCE_ROOT ?? path.join(root, 'source_repo'));
const esbuild = require(path.join(sourceRoot, 'node_modules/esbuild'));
const build = esbuild.buildSync({
  absWorkingDir: sourceRoot,
  stdin: { contents: "export { AccessibilityManager } from './src/browser/AccessibilityManager.ts'; export * as Strings from './src/browser/LocalizableStrings.ts';", resolveDir: sourceRoot, loader: 'ts' },
  bundle: true, write: false, format: 'esm', platform: 'node', metafile: true,
  tsconfigRaw: { compilerOptions: { experimentalDecorators: true } },
});
const { AccessibilityManager, Strings } = await import(`data:text/javascript;base64,${Buffer.from(build.outputFiles[0].contents).toString('base64')}`);
const defaultLabel = Strings.tooMuchOutput.get();
const units = value => Array.from({ length: value.length }, (_, i) => value.charCodeAt(i));
const hex = value => value.map(unit => unit.toString(16)).join(',');
const string = value => String.fromCharCode(...value);
function model() {
  // Constructor and DOM services are intentionally NOT tested by this corpus.
  // The prototype methods below execute unchanged against their own field shape.
  const model = Object.create(AccessibilityManager.prototype);
  model._liveRegion = { textContent: '' };
  model._liveRegionLineCount = 0;
  model._charsToConsume = [];
  model._charsToAnnounce = '';
  return model;
}
const mask = Buffer.alloc(65536 / 8);
for (let unit = 0; unit <= 65535; unit++) {
  const state = model(); state._handleKey(String.fromCharCode(unit));
  if (state._charsToConsume.length === 0) mask[unit >> 3] |= 1 << (unit & 7);
}
const fixtures = [];
let operations = 0;
function trace(steps) {
  const state = model(); Strings.tooMuchOutput.set(defaultLabel);
  const snapshots = [];
  for (const step of steps) {
    const [kind, encoded = ''] = step.split(':');
    const data = encoded.split(',').filter(Boolean).map(value => parseInt(value, 16));
    switch (kind) {
      case 'c': state._handleChar(string(data)); break;
      case 'k': state._handleKey(string(data)); break;
      case 'l': Strings.tooMuchOutput.set(string(data)); break;
      case 'p': state._liveRegion.textContent = string(data); break;
      case 't': state._handleTab(Number(encoded)); break;
      case 'b': state._clearLiveRegion(); break;
      case 'f': state._announceCharacters(); break;
      default: throw new Error(`Unknown step ${step}`);
    }
    snapshots.push({ live: units(state._liveRegion.textContent), pending: units(state._charsToAnnounce),
      consume: state._charsToConsume.map(units), lines: state._liveRegionLineCount });
    operations++;
  }
  fixtures.push(`${steps.join(';')}\t${JSON.stringify(snapshots)}`);
}
const value = text => hex(units(text));
const label = `l:${value('overflow')}`;
for (const steps of [
  ['c:41', 'k:41', 'c:41', 'f', 'c:42', 'f'],
  ['k:', 'c:', 'f'], ['k:61,62', 'c:61', 'c:62', 'f'],
  ['k:d83d,de00', 'c:d83d,de00', 'k:d800', 'c:d800', 'c:dc00', 'f'],
  ['c:61,a,62', 't:80', 'f'], ['c:41', 'k:42', 'b', 'c:42', 'f'],
  [label, ...Array(21).fill('c:a'), 'c:41', 'f', 'c:42', 'f'],
  [label, ...Array(21).fill('c:a'), `l:${value('changed')}`, 'f', 'c:41', 'f', 'k:42', 'c:42', 'f'],
  ['l:', 'c:a', 'f', 'f'], ['k:41', 'k:42', 'c:43', 'b', 'c:42', 'f'],
  ['p:41', 'c:42', 'f', 'b', 'f'], ['k:0', 'k:7f', 'k:9f', 'k:feff', 'c:feff', 'f'],
]) trace(steps);
const directed = fixtures.length;
const seed = 0x0a11e72;
let state = seed;
function random() { state ^= state << 13; state ^= state >>> 17; state ^= state << 5; return state >>> 0; }
const interesting = [0, 9, 10, 31, 32, 65, 127, 128, 159, 160, 0x200d, 0xfe0f, 0xfeff, 0xd800, 0xd83d, 0xde00, 0xdfff, 0xffff];
for (let index = 0; index < 256; index++) {
  const steps = [];
  for (let step = 0; step < 128; step++) {
    const kind = random() % 12;
    if (kind < 7) {
      const text = Array.from({ length: random() % 6 }, () => random() % 3 ? interesting[random() % interesting.length] : random() % 65536);
      steps.push(`${kind < 4 ? 'c' : kind < 6 ? 'k' : 'p'}:${hex(text)}`);
    } else if (kind < 9) steps.push('f');
    else if (kind === 9) steps.push('b');
    else if (kind === 10) steps.push(`t:${random() % 9}`);
    else steps.push(`l:${[[], [0x3f], [0xd800], units(defaultLabel)][random() % 4].map(value => value.toString(16)).join(',')}`);
  }
  trace(steps);
}
Strings.tooMuchOutput.set(defaultLabel);
const corpus = `${fixtures.join('\n')}\n`;
const control = `${mask.toString('hex')}\n`;
const destination = path.join(root, 'crates/xterm-web/tests/fixtures');
await mkdir(destination, { recursive: true });
const fixtureHashes = {};
for (const [name, contents] of [['accessibility-state.tsv', corpus], ['accessibility-control.hex', control]]) {
  const file = path.join(destination, name);
  if (process.argv.includes('--generate')) await writeFile(file, contents);
  const raw = await readFile(file);
  assert.equal(canonicalTsvRecords(raw.toString('utf8')), contents, `accessibility fixture drift: ${name}`);
  fixtureHashes[name] = { rawSha256: createHash('sha256').update(raw).digest('hex'), canonicalSha256: createHash('sha256').update(contents).digest('hex') };
}
const sources = await Promise.all(Object.keys(build.metafile.inputs).filter(value => value !== '<stdin>').sort()
  .map(async source => ({ path: `source_repo/${source.replaceAll(path.sep, '/')}`, sha256: createHash('sha256').update(await readFile(path.join(sourceRoot, source))).digest('hex') })));
const report = { node: process.version, esbuild: esbuild.version, seed, directedTraces: directed,
  randomTraces: 256, randomStepsPerTrace: 128, operations, bmpKeys: 65536, sources,
  fixtures: fixtureHashes,
  fixtureSha256: fixtureHashes['accessibility-state.tsv'].rawSha256, controlSha256: fixtureHashes['accessibility-control.hex'].rawSha256,
  upstreamUnitCasesClaimed: 0,
  scope: 'unchanged private announcement methods; native policy only, constructor/DOM/browser/Terminal are not exercised',
};
await mkdir(path.join(root, 'results/accessibility'), { recursive: true });
await writeFile(path.join(root, 'results/accessibility/oracle.json'), `${JSON.stringify(report, null, 2)}\n`);
console.log(JSON.stringify(report, null, 2));
