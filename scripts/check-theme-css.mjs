// Exact text from the unchanged DOM renderer method; no browser-engine claim.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

process.chdir(resolve(import.meta.dirname, '..'));
const require = createRequire(resolve('source_repo/package.json'));
const esbuild = require('esbuild');
mkdirSync('tmp/theme-css', { recursive: true });
mkdirSync('results/theme-css', { recursive: true });
const entry = `export { DomRenderer } from '../../source_repo/src/browser/renderer/dom/DomRenderer';
export { DEFAULT_ANSI_COLORS } from '../../source_repo/src/browser/Types';
export { channels } from '../../source_repo/src/common/Color';`;
writeFileSync('tmp/theme-css/entry.ts', entry);
await esbuild.build({ entryPoints: ['tmp/theme-css/entry.ts'], outfile: 'tmp/theme-css/source.mjs',
  bundle: true, platform: 'node', format: 'esm', nodePaths: [resolve('source_repo/src')],
  tsconfig: 'source_repo/src/browser/tsconfig.json' });
const { DomRenderer, DEFAULT_ANSI_COLORS, channels } = await import(pathToFileURL(resolve('tmp/theme-css/source.mjs')));
const cases = [];
const base = () => ({ foreground: channels.toColor(255,255,255), background: channels.toColor(0,0,0),
  cursor: channels.toColor(255,255,255), cursorAccent: channels.toColor(0,0,0),
  selectionBackgroundOpaque: channels.toColor(77,77,77),
  selectionInactiveBackgroundOpaque: channels.toColor(77,77,77), ansi: DEFAULT_ANSI_COLORS.slice() });
const number = value => Object.is(value, -0) ? '-0' : String(value);
function add(name, id, colors, family = 'courier-new, courier, monospace', size = 15,
  weight = 'normal', weightBold = 'bold', cursorWidth = 1) {
  const source = Object.create(DomRenderer.prototype);
  source._terminalClass = id;
  source._themeStyleElement = { textContent: '' };
  source._optionsService = { rawOptions: { fontFamily: family, fontSize: size,
    fontWeight: weight, fontWeightBold: weightBold, cursorWidth } };
  source._injectCss(colors);
  assert(source._themeStyleElement.textContent.includes(`.xterm-dom-renderer-owner-${id}`));
  cases.push({ name, id: String(id), colors, font: { family, size: number(size), weight, weightBold },
    cursorWidth: number(cursorWidth), expectedCss: source._themeStyleElement.textContent });
}
add('default palette', 1, base());
add('second terminal scoped selector', 2, base(), '"Noto Sans Mono", monospace', 13.5, '500', '900', 2.5);
const independent = base();
independent.foreground = { css: 'rgb(7, 8, 9)', rgba: 0xaAbBcC81 };
independent.background = { css: '#11223344', rgba: 0x11335577 };
independent.cursor = { css: 'rebeccapurple', rgba: 0x663399ff };
independent.cursorAccent = { css: 'white', rgba: 0x010203ff };
independent.selectionBackgroundOpaque = { css: '#ABCDEF', rgba: 0x00112233 };
independent.selectionInactiveBackgroundOpaque = { css: 'rgb(11, 22, 33)', rgba: 0x00000000 };
independent.ansi = [{ css: '#fF000080', rgba: 0x33669980 }];
add('independent retained CSS and packed RGBA', 7, independent, 'emoji😀, "Γραφή"', 1e-7, 'normal', 'bold', -0);
for (const [index, value] of [NaN, Infinity, -Infinity, -0, 1e21, 1e-7].entries()) {
  const colors = base(); colors.ansi = [];
  add(`raw numeric ${index}`, 10 + index, colors, 'monospace', value, '0', '999', value);
}
const seed = 0xc55c0101;
let state = seed;
const random = () => { state ^= state << 13; state ^= state >>> 17; state ^= state << 5; return state >>> 0; };
const randomColor = () => {
  const rgba = random();
  const css = random() & 1 ? '#' + rgba.toString(16).padStart(8, '0').toUpperCase() : `rgb(${random()%256}, ${random()%256}, ${random()%256})`;
  return { css, rgba };
};
for (let index = 0; index < 64; index++) {
  const colors = Object.fromEntries(['foreground','background','cursor','cursorAccent','selectionBackgroundOpaque','selectionInactiveBackgroundOpaque']
    .map(key => [key, randomColor()]));
  colors.ansi = Array.from({ length: [0,1,2,16,256,300][index % 6] }, randomColor);
  add(`seeded palette ${index}`, index === 63 ? Number.MAX_SAFE_INTEGER : 100 + index, colors,
    ['monospace', '"A B", serif', 'Cascadia Code, monospace', '字形😀'][index % 4],
    (random() % 9999) / 100, String(random() % 1000), String(random() % 1000), (random() % 1000) / 100);
}
const fixture = JSON.stringify(cases) + '\n';
const path = 'crates/xterm-web/tests/fixtures/theme-css.json';
if (process.argv.includes('--write-fixtures')) writeFileSync(path, fixture);
else assert.equal(readFileSync(path, 'utf8').replaceAll('\r\n', '\n'), fixture);
const hash = value => createHash('sha256').update(value).digest('hex');
const sourcePaths = ['source_repo/src/browser/renderer/dom/DomRenderer.ts',
  'source_repo/src/browser/renderer/dom/DomRendererRowFactory.ts', 'source_repo/src/browser/renderer/shared/Constants.ts',
  'source_repo/src/common/Color.ts'];
const report = { scope: '_injectCss generated text only; style-node lifecycle, raw UTF-16 bindings and browser computed styles remain task32',
  status: 'passed source fixture check', node: process.version, esbuild: esbuild.version, seed, cases: cases.length,
  ansiLengths: [...new Set(cases.map(item => item.colors.ansi.length))],
  source: Object.fromEntries(sourcePaths.map(path => [path, hash(readFileSync(path))])), fixtureSha256: hash(fixture),
  casesSha256: cases.map(item => ({ name: item.name, bytes: Buffer.byteLength(item.expectedCss), cssSha256: hash(item.expectedCss) })) };
writeFileSync('results/theme-css/oracle.json', JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report, null, 2));
