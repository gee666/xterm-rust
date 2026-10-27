// Run the unchanged upstream assertion suite and seeded Rust/JS color comparisons.
// Requires Node 24 and tmp/color-oracle npm dependencies documented in docs/color.md.
import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { stripTypeScriptTypes } from 'node:module';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';

const root = process.cwd();
const temporary = path.resolve('tmp/color-oracle');
fs.mkdirSync(temporary, { recursive: true });
fs.mkdirSync('results/color', { recursive: true });
const source = 'source_repo/src/common/Color.ts';
const tests = 'source_repo/src/common/Color.test.ts';
const original = fs.readFileSync(source, 'utf8');
const typeImport = "import { IColor, IColorRGB } from './Types';";
if (original.split(typeImport).length !== 2) throw new Error('Color type import changed');
const colorJs = stripTypeScriptTypes(original.replace(typeImport, ''), { mode: 'transform' });
const colorUrl = pathToFileURL(path.join(temporary, 'Color.mjs')).href;
fs.writeFileSync(path.join(temporary, 'Color.mjs'), colorJs);
const { color, rgba, rgb, css, channels } = await import(colorUrl);
const mochaUrl = pathToFileURL(path.join(temporary, 'node_modules/mocha/index.js')).href;
const chaiUrl = pathToFileURL(path.join(temporary, 'node_modules/chai/index.js')).href;
const { default: Mocha } = await import(mochaUrl);
const testText = fs.readFileSync(tests, 'utf8');
const testJs = stripTypeScriptTypes(testText, { mode: 'transform' })
  .replace("from 'chai'", `from ${JSON.stringify(chaiUrl)}`)
  .replace("from './Color'", `from ${JSON.stringify(colorUrl)}`);
const testFile = path.join(temporary, 'Color.test.mjs');
fs.writeFileSync(testFile, testJs);
const mocha = new Mocha({ reporter: 'spec' });
mocha.addFile(testFile);
await mocha.loadFilesAsync();
let runner;
const failures = await new Promise(resolve => { runner = mocha.run(resolve); });
const baseline = {
  source, tests, sourceSha256: createHash('sha256').update(original).digest('hex'),
  testSha256: createHash('sha256').update(testText).digest('hex'),
  node: process.version, mocha: '11.7.5', chai: '6.2.2', platform: process.platform,
  discovered: runner.stats.tests, assertionsDiscovered: (testText.match(/assert\./g) ?? []).length,
  passed: runner.stats.passes, failed: failures, skipped: runner.stats.pending,
};
fs.writeFileSync('results/color/original.json', JSON.stringify(baseline, null, 2) + '\n');
if (failures) throw new Error(`${failures} original Color tests failed`);

const seed = 0xc010a123;
let state = seed;
function random() { state ^= state << 13; state ^= state >>> 17; state ^= state << 5; return state >>> 0; }
const input = [], expected = [];
function add(op, args, result) { input.push([op, ...args].join('\t')); expected.push(String(result)); }
function encode(color) { return `${color.css}|${color.rgba}`; }
for (let i = 0; i < 4096; i++) {
  const bg = random(), fg = random(), ratio = random() % 300 / 10;
  add('blend', [bg, fg], rgba.blend(bg, fg));
  add('ensure', [bg, fg, ratio], rgba.ensureContrastRatio(bg, fg, ratio));
  add('reduce', [bg, fg, ratio], rgba.reduceLuminance(bg, fg, ratio));
  add('increase', [bg, fg, ratio], rgba.increaseLuminance(bg, fg, ratio));
  add('luminance', [bg], rgb.relativeLuminance(bg).toFixed(4));
  const sourceColor = channels.toColor(...rgba.toChannels(fg));
  const opacity = (random() % 2001 - 500) / 1000;
  add('opacity', [fg, opacity], encode(color.opacity(sourceColor, opacity)));
  add('multiply', [fg, opacity], encode(color.multiplyOpacity(sourceColor, opacity)));
}
for (const opacity of [NaN, Infinity, -Infinity, 0, -0.5, 0.5, 1, 1e20, -1e20, 1e300, -1e300, 4503599627370497 / 255]) {
  const sourceColor = channels.toColor(18, 52, 86, 128);
  add('opacity', [sourceColor.rgba, opacity], encode(color.opacity(sourceColor, opacity)));
  add('multiply', [sourceColor.rgba, opacity], encode(color.multiplyOpacity(sourceColor, opacity)));
}
const parseCases = [
  '#000', '#123', '#ABC', '#aBcD', '#102030', '#AABBCC', '#01234567', '#fffffffe',
  '#00000', '#1234567', '#12', '#gggg', '', 'red', 'transparent', 'TRANSPARENT',
  'rgb(0,0,0)', 'rgba(1,2,3)', 'rgb(1,2,3,.5)', 'rgba(255,128,0,0.25)',
  'rgb(999,999,999)', 'rgba(999,256,511,9.99)', 'rgb(0000,0,0)', 'rgb(0,0,0,2)',
  'xxxrgb(1,2,3)zzz', 'rgb(1,2,3,00.5)', 'rgba(0,0,0,.00000001)',
  'rgb(\u00a01,2,3)', 'rgb(1,2,3,1.5)', 'rgba(0,0,0,1.)', 'rgb(1, 2, 3, .9)',
  'x#abc', 'xx#abcx', 'x#abcdxx', '💻#abc', '#abczzz', '#12abcd!',
];
for (const text of parseCases) {
  let result;
  try { result = encode(css.toColor(text)); } catch { result = 'error'; }
  add('parse', [text], result);
}
const cargo = spawnSync(process.env.CARGO ?? 'cargo', ['run', '--quiet', '-p', 'xterm-core', '--example', 'color_oracle'], {
  cwd: root, encoding: 'utf8', input: input.join('\n') + '\n', maxBuffer: 32 * 1024 * 1024,
  timeout: 120000,
});
if (cargo.error) throw cargo.error;
if (cargo.status !== 0) throw new Error(cargo.stderr);
const actual = cargo.stdout.trimEnd().split(/\r?\n/);
const mismatches = [];
for (let i = 0; i < Math.max(actual.length, expected.length); i++) {
  if (actual[i] !== expected[i]) mismatches.push({ index: i, input: input[i], actual: actual[i], expected: expected[i] });
}
const differential = {
  seed, iterations: 4096, operations: input.length, parseCases: parseCases.length,
  compared: actual.length, mismatches: mismatches.length, platform: process.platform, node: process.version,
  sourceSha256: baseline.sourceSha256, status: mismatches.length ? 'failed' : 'passed',
};
fs.writeFileSync('results/color/differential.json', JSON.stringify(differential, null, 2) + '\n');
if (mismatches.length) {
  fs.writeFileSync('results/color/counterexamples.json', JSON.stringify(mismatches, null, 2) + '\n');
  console.error(mismatches.slice(0, 10));
  throw new Error(`${mismatches.length} color differential mismatches`);
}
console.log(JSON.stringify(differential, null, 2));
