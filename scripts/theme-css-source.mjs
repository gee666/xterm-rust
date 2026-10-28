// Tiny source-only loader. Uses Node's built-in TS erasure, no build subprocess.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import vm from 'node:vm';
import { createHash } from 'node:crypto';

const paths = {
  color: 'source_repo/src/common/Color.ts',
  renderer: 'source_repo/src/browser/renderer/dom/DomRenderer.ts',
  row: 'source_repo/src/browser/renderer/dom/DomRendererRowFactory.ts',
  shared: 'source_repo/src/browser/renderer/shared/Constants.ts'
};
function section(text, startMarker, endMarker) {
  const start = text.indexOf(startMarker);
  const end = text.indexOf(endMarker, start + startMarker.length);
  assert(start >= 0 && end > start, `source section missing: ${startMarker}`);
  return text.slice(start, end + endMarker.length);
}
export function loadCssSource() {
  const source = Object.fromEntries(Object.entries(paths).map(([name, path]) => [name, readFileSync(path, 'utf8')]));
  const constants = section(source.renderer, 'const enum Constants {', '\n}');
  const rowConstants = section(source.row, 'export const enum RowCss {', '\n}');
  const inject = section(source.renderer, '  private _injectCss(', '\n  }');
  const selector = section(source.renderer, '  private get _terminalSelector()', '\n  }');
  assert(inject.includes('this._themeStyleElement.textContent = styles;'));
  const erase = text => stripTypeScriptTypes(text.replace(/^import .*;\r?$/gm, '').replace(/^export /gm, ''), { mode: 'transform' });
  const context = vm.createContext({});
  vm.runInContext(erase(source.color) + '\n' + erase(source.shared) + '\n' + erase(constants + '\n' + rowConstants + '\nclass CssSource {\n' + inject + '\n' + selector + '\n}') +
    '\nglobalThis.reference = { CssSource, css, color, channels };', context);
  const reference = context.reference;
  return {
    ...reference,
    generate(id, colors, options) {
      const renderer = new reference.CssSource();
      renderer._terminalClass = id;
      renderer._themeStyleElement = { textContent: '' };
      renderer._optionsService = { rawOptions: options };
      renderer._injectCss(colors);
      return renderer._themeStyleElement.textContent;
    },
    hashes: Object.fromEntries(Object.entries(paths).map(([name, path]) => [path, createHash('sha256').update(source[name]).digest('hex')]))
  };
}
export function units(value) {
  return Array.from({ length: value.length }, (_, index) => value.charCodeAt(index));
}
export function text(value) {
  return value.map(unit => String.fromCharCode(unit)).join('');
}
