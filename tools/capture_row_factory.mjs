// Executes unchanged original assertions and records complete RowFactory calls.
// Run scripts/upstream-baseline.mjs first to produce source_repo/out-esbuild.
import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
import { createHash } from 'node:crypto';
import { captureCells } from './render_fixture_helpers.mjs';
const require = createRequire(path.resolve('source_repo/package.json'));
const Mocha = require('mocha');
const base = path.resolve('source_repo/out-esbuild');
const { DomRendererRowFactory } = require(path.join(base, 'browser/renderer/dom/DomRendererRowFactory.js'));
const { CellData } = require(path.join(base, 'common/buffer/CellData.js'));
const calls = [];
const original = DomRendererRowFactory.prototype.createRow;
DomRendererRowFactory.prototype.createRow = function (...args) {
  const [line, row, cursorRow, cursorStyle, inactiveStyle, cursorX, cursorBlink, blinkOn,
    cellWidth, widths, linkStart, linkEnd] = args;
  const cells = captureCells(line, CellData);
  const colors = {};
  for (const key of ['foreground', 'background', 'selectionForeground', 'selectionBackgroundOpaque', 'selectionInactiveBackgroundOpaque']) {
    const color = this._themeService.colors[key];
    if (color) colors[key] = { css: color.css, rgba: color.rgba >>> 0 };
  }
  colors.ansi = this._themeService.colors.ansi.map(color => ({ css: color.css, rgba: color.rgba >>> 0 }));
  const record = { row, cursorRow, cursorStyle: cursorStyle ?? 'block', inactiveStyle: inactiveStyle ?? '',
    cursorX, cursorBlink, blinkOn, cellWidth, linkStart, linkEnd, cells, colors,
    focused: this._coreBrowserService.isFocused,
    cursorHidden: this._coreService.isCursorHidden, cursorInitialized: this._coreService.isCursorInitialized,
    boldBright: this._optionsService.rawOptions.drawBoldTextInBrightColors,
    contrast: this._optionsService.rawOptions.minimumContrastRatio,
    defaultSpacing: this.defaultSpacing, selectionStart: this._selectionStart,
    selectionEnd: this._selectionEnd, column: this._columnSelectMode,
    widths: widths._canvasElements.map(canvas => canvas.widths) };
  const spans = original.apply(this, args);
  const element = this._document.createElement('div');
  element.replaceChildren(...spans);
  record.html = element.innerHTML;
  calls.push(record);
  return spans;
};
const mocha = new Mocha({ reporter: 'dot', grep: /DomRendererRowFactory/ });
mocha.addFile(path.join(base, 'browser/renderer/dom/DomRendererRowFactory.test.js'));
const source = 'source_repo/src/browser/renderer/dom/DomRendererRowFactory.test.ts';
const failed = await new Promise(resolve => mocha.run(resolve));
if (failed) throw new Error(`${failed} original row assertions failed; fixtures not updated`);
if (!calls.length) throw new Error('No original row calls captured');
const result = { source, sha256: createHash('sha256').update(fs.readFileSync(source)).digest('hex'), calls };
fs.mkdirSync('crates/xterm-web/tests/fixtures', { recursive: true });
fs.writeFileSync('crates/xterm-web/tests/fixtures/row-factory.json', JSON.stringify(result) + '\n');
fs.mkdirSync('results/renderer', { recursive: true });
fs.writeFileSync('results/renderer/row-original.json', JSON.stringify({ source, sha256: result.sha256, calls: calls.length, failed }, null, 2) + '\n');
console.log(JSON.stringify({ source, calls: calls.length, failed }));
