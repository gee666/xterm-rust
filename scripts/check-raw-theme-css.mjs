// Source-only UTF-16 oracle. No bundler/compiler subprocess or browser launch.
import assert from 'node:assert/strict';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { resolve } from 'node:path';
import { loadCssSource, units, text } from './theme-css-source.mjs';
process.chdir(resolve(import.meta.dirname, '..'));
mkdirSync('results/theme-css-raw', { recursive: true });
const source = loadCssSource();
const color = (css, rgba) => ({ css: text(css), rgba });
const patterns = [[], [0xd800], [0xdc00], [0xd800,97,0xdc00], [0,126,48,48,52,49,126],
  [0xd83d,0xde00], [0xfeff,0x85,0x2028], [0xffff,0,0xfffe], units('rgb(/*'), units('字形')];
const slots = ['foreground','background','cursor','cursorAccent','selectionBackgroundOpaque','selectionInactiveBackgroundOpaque'];
const cases = [];
for (let index = 0; index < 30; index++) {
  const colors = Object.fromEntries(slots.map((name, slot) => [name, color(patterns[(index + slot) % patterns.length], (0x11223300 + ((index + slot) % 256)) >>> 0)]));
  colors.ansi = Array.from({ length: index % 4 }, (_, item) => color(patterns[(index + item + 2) % patterns.length], 0xabcdef80 + item));
  const options = {
    fontFamily: text(patterns[(index + 1) % patterns.length]), fontSize: text(patterns[(index + 2) % patterns.length]),
    fontWeight: text(patterns[(index + 3) % patterns.length]), fontWeightBold: text(patterns[(index + 4) % patterns.length]),
    cursorWidth: text(patterns[(index + 5) % patterns.length])
  };
  const output = source.generate(index + 1, colors, options);
  cases.push({ name: `raw units ${index}`, id: index + 1,
    colors: Object.fromEntries(Object.entries(colors).map(([key, value]) => [key, key === 'ansi' ? value.map(c => ({ css: units(c.css), rgba: c.rgba })) : { css: units(value.css), rgba: value.rgba }])),
    options: Object.fromEntries(Object.entries(options).map(([name, value]) => [name, units(value)])), expected: units(output) });
}
// Actual source callbacks can mutate later options and colors. Keep this trace
// separately from immutable cases rather than pretending a pre-coerced snapshot.
const mutations = [];
const hints = [];
const liveColors = Object.fromEntries(slots.map(name => [name, { css: '#ffffff', rgba: 0xffffffff }]));
liveColors.ansi = [];
const liveOptions = { fontSize: 'old-size', fontWeight: 'old-weight', fontWeightBold: 'old-bold', cursorWidth: 'old-width' };
const primitive = (name, result, action) => ({ [Symbol.toPrimitive](hint) { hints.push([name, hint]); action(); return result; } });
liveOptions.fontFamily = primitive('family', text([97,0xd800]), () => {
  liveColors.foreground = { css: 'after-family', rgba: 0x00ff00ff };
  mutations.push('foreground green');
  liveOptions.fontSize = primitive('size', '15e0', () => {
    liveColors.foreground = { css: 'after-size', rgba: 0x0000ffff };
    mutations.push('foreground blue');
  });
});
liveOptions.fontWeight = primitive('weight', 'normal', () => { liveColors.background = { css: 'ignored direct BG', rgba: 0x11223344 }; });
liveOptions.fontWeightBold = primitive('weightBold', 'bold', () => { liveColors.cursor = { css: 'before-width', rgba: 0xffffffff }; });
liveOptions.cursorWidth = primitive('cursorWidth', '2.5', () => { liveColors.cursor = { css: 'after-width', rgba: 0xffffffff }; });
const liveOutput = source.generate(99, liveColors, liveOptions);
assert.deepEqual(hints.map(item => item[0]), ['family','size','weight','weightBold','cursorWidth']);
assert(hints.every(item => item[1] === 'string'));
assert(liveOutput.includes('pointer-events: none; color: #ffffff;'));
assert(liveOutput.includes('.xterm-dim { color: #0000ff80;'));
assert(liveOutput.includes('box-shadow: 2.5px 0 0 after-width inset;'));
const live = { id: 99, expected: units(liveOutput), hints, mutations };
const fixture = JSON.stringify({ cases, live }) + '\n';
const fixturePath = 'crates/xterm-web/tests/fixtures/theme-css-raw.json';
if (process.argv.includes('--write-fixtures')) writeFileSync(fixturePath, fixture);
else assert.equal(readFileSync(fixturePath, 'utf8').replaceAll('\r\n','\n'), fixture);
const fast = [];
for (const unit of [0, 0x20, 0x85, 0xa0, 0x1680, 0x2028, 0xfeff, 0xffff, 0xd800, 0xdbff, 0xdc00, 0xdfff, 0xd83d]) {
  for (const [prefix, suffix] of [['', '#abc'], ['', '#abcde'], ['#', 'abcde'], ['', 'rgb(1,2,3)'], ['rgba(', '1,2,3,.5)'], ['#abcd', '']]) {
    const input = [...units(prefix), unit, ...units(suffix)];
    try { const result = source.css.toColor(text(input)); fast.push({ input, css: units(result.css), rgba: result.rgba }); }
    catch { fast.push({ input, unsupported: true }); }
  }
}
const fastFixture = JSON.stringify(fast) + '\n';
const fastPath = 'crates/xterm-web/tests/fixtures/color-utf16-fast.json';
if (process.argv.includes('--write-fixtures')) writeFileSync(fastPath, fastFixture);
else assert.equal(readFileSync(fastPath, 'utf8').replaceAll('\r\n','\n'), fastFixture);
const hash = value => createHash('sha256').update(value).digest('hex');
const report = { scope: 'CSS full text and Node fast-color stage; not canvas fallback, cache, DOM lifecycle or browser proof',
  node: process.version, source: source.hashes, styles: cases.length, liveCallbackHints: hints,
  fastColorCases: fast.length, fixtureSha256: hash(fixture), fastFixtureSha256: hash(fastFixture),
  expectedUtf16Units: cases.reduce((sum, item) => sum + item.expected.length, live.expected.length), status: 'passed source checks' };
writeFileSync('results/theme-css-raw/oracle.json', JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report, null, 2));
