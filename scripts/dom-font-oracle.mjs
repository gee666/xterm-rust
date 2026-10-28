// Execute unchanged source DomRenderer/WidthCache with jsdom and bounded
// canvas/clock doubles. Browser tests separately exercise real Rust canvases.
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { createHash } from 'node:crypto';
const sourceRoot = path.resolve(process.env.SOURCE_ROOT ?? 'source_repo');
const require = createRequire(path.join(sourceRoot, 'package.json'));
const ts = require('typescript');
const { JSDOM } = require('jsdom');
const browser = new JSDOM('<!doctype html><body></body>');
const document = browser.window.document;
// General CSS canvas parsing is outside this font-caller trace. jsdom has no
// canvas backend; the measurement canvases below receive explicit doubles.
browser.window.HTMLCanvasElement.prototype.getContext = () => null;
const cache = new Map(), hashes = {};
const sandbox = vm.createContext({ console, document, window: browser.window,
  navigator: browser.window.navigator, setTimeout() { throw Error('Unexpected timer'); },
  clearTimeout() {}, Float32Array, Map, WeakMap, Set, WeakSet, TextDecoder, TextEncoder });
function load(relative) {
  const file = path.resolve(sourceRoot, 'src', relative);
  if (cache.has(file)) return cache.get(file).exports;
  const source = fs.readFileSync(file, 'utf8');
  hashes[path.relative(sourceRoot, file).replaceAll('\\', '/')] = createHash('sha256').update(source).digest('hex');
  const code = ts.transpileModule(source, { compilerOptions: {
    module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022,
    experimentalDecorators: true
  } }).outputText;
  const module = { exports: {} };
  cache.set(file, module);
  const localRequire = id => {
    if (!id.startsWith('.')) throw Error(`Unexpected source dependency ${id}`);
    return load(path.relative(path.join(sourceRoot, 'src'), path.resolve(path.dirname(file), `${id}.ts`)));
  };
  vm.runInContext(`(function(require,module,exports){${code}\n})`, sandbox, { filename: file })(localRequire, module, module.exports);
  return module.exports;
}
const { DomRenderer } = load('browser/renderer/dom/DomRenderer.ts');
const { OptionsService } = load('common/services/OptionsService.ts');
const { ThemeService } = load('browser/services/ThemeService.ts');
const { Emitter } = load('common/Event.ts');
const units = text => Array.from({ length: text.length }, (_, i) => text.charCodeAt(i));
function fixture() {
  const events = [], contexts = [];
  const options = new OptionsService({});
  for (const [key, text] of [['fontFamily', 'Face\ud800\0\udc00'], ['fontSize', '15e0'],
    ['fontWeight', 'normal'], ['fontWeightBold', 'bold']]) {
    options.rawOptions[key] = { [Symbol.toPrimitive](hint) { events.push(['coerce', key, hint]); return text; } };
  }
  const create = document.createElement;
  const probe = { events, contexts, onFont: undefined };
  document.createElement = function(...args) {
    const element = create.apply(this, args);
    if (args[0] === 'canvas') {
      const index = contexts.length;
      contexts.push(element);
      events.push(['canvas', index]);
      const context = { measureText() { return { width: 3 }; },
        set font(value) { events.push(['font', index, units(value)]); probe.onFont?.(); } };
      element.getContext = () => context;
    }
    return element;
  };
  const parent = document.createElement('div');
  document.body.appendChild(parent);
  const nodes = Array.from({ length: 4 }, () => document.createElement('div'));
  const event = () => new Emitter().event;
  const theme = new ThemeService(options);
  const terminal = { cols: 4, rows: 2 };
  const buffer = { cols: 4, rows: 2 };
  // The row factory is not called by the font paths under test.
  const instantiation = { createInstance() { return { defaultSpacing: 0 }; } };
  const renderer = new DomRenderer(terminal, document, ...nodes,
    { onShowLinkUnderline: event(), onHideLinkUnderline: event() }, instantiation,
    { width: 8, height: 16 }, options, buffer, {},
    { isFocused: false, dpr: 1, window: { setInterval() { throw Error('Unexpected interval'); } } }, theme);
  probe.renderer = renderer;
  probe.options = options;
  probe.close = () => { renderer.dispose(); theme.dispose(); options.dispose(); parent.remove(); document.createElement = create; };
  return probe;
}
const result = {};
try {
  let f = fixture();
  result.constructor = f.events.slice();
  assert.equal(result.constructor.filter(event => event[0] === 'font').length, 4);
  f.close();
  f = fixture();
  f.events.length = 0;
  const style = f.renderer._themeStyleElement;
  const descriptor = Object.getOwnPropertyDescriptor(browser.window.Node.prototype, 'textContent');
  Object.defineProperty(style, 'textContent', { configurable: true, set(value) {
    f.options.rawOptions.fontFamily = '\ud800a\udc00'; descriptor.set.call(this, value);
  } });
  f.options.options.cursorBlink = true;
  result.afterCssMutation = f.events.filter(event => event[0] === 'font');
  assert.equal(result.afterCssMutation.length, 4);
  assert(result.afterCssMutation.every(event => event[2].slice(-3).join() === [0xd800, 97, 0xdc00].join()));
  f.close();
  f = fixture();
  let calls = 0;
  f.onFont = () => { calls++; f.renderer.handleCharSizeChanged(); };
  f.options.rawOptions.fontFamily = 'changed';
  f.options.options.cursorBlink = true;
  result.reentrantCalls = calls;
  assert.equal(calls, 4);
  f.close();
  f = fixture();
  f.events.length = 0;
  const error = {};
  Object.defineProperty(f.renderer._themeStyleElement, 'textContent', { set() { throw error; } });
  assert.throws(() => { f.options.options.cursorBlink = true; }, actual => actual === error);
  result.failedCssCanvasWrites = f.events.filter(event => event[0] === 'font').length;
  assert.equal(result.failedCssCanvasWrites, 0);
  f.close();
  result.sourceHashes = Object.fromEntries(Object.entries(hashes).sort(([a], [b]) => a.localeCompare(b)));
  const fixturePath = 'crates/xterm-web/tests/fixtures/dom-font.json';
  const json = JSON.stringify(result, null, 2) + '\n';
  if (process.argv.includes('--write-fixtures')) fs.writeFileSync(fixturePath, json);
  else assert.equal(fs.readFileSync(fixturePath, 'utf8').replaceAll('\r\n', '\n'), json);
  fs.mkdirSync('results/dom-font', { recursive: true });
  fs.writeFileSync('results/dom-font/source.json', JSON.stringify({ status: 'passed', cases: 4,
    scope: 'Unchanged source font callers with jsdom/canvas doubles, not browser pixel parity', ...result }, null, 2) + '\n');
  console.log('DomRenderer font callers: four source cases passed');
} finally { browser.window.close(); }
