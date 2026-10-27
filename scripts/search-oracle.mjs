// Copyright (c) 2026 xterm-rust contributors. MIT license.
// Original engine assertions execute unchanged; appended observers retain inputs.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const root = resolve(import.meta.dirname, '..');
const { build } = await import(pathToFileURL(resolve(root, 'source_repo/node_modules/esbuild/lib/main.js')));
const { default: Mocha } = await import(pathToFileURL(resolve(root, 'source_repo/node_modules/mocha/index.js')));
const temp = resolve(root, 'tmp/search-oracle');
const output = resolve(root, 'results/search');
const fixtures = resolve(root, 'crates/xterm-core/tests/fixtures');
for (const dir of [temp, output, fixtures]) mkdirSync(dir, { recursive: true });
const sourceHashes = {};
const hex = string => Array.from({ length: string.length }, (_, i) => string.charCodeAt(i).toString(16)).join(',') || '-';
const tri = flag => flag === undefined ? 'u' : flag ? 't' : 'f';
const opts = o => [o?.caseSensitive, o?.regex, o?.wholeWord].map(tri).join('');
const pos = p => `${p.y},${p.x}`;
const selected = s => s ? `${pos(s.start)},${pos(s.end)}` : '-';
function rows(terminal) {
  const active = terminal.buffer.active;
  return Array.from({ length: active.length }, (_, row) => {
    const line = active.getLine(row);
    const cells = Array.from({ length: terminal.cols }, (_, col) => {
      const cell = line.getCell(col);
      return `${hex(cell.getChars())}:${cell.getWidth()}`;
    });
    while (cells.at(-1) === '-:1') cells.pop();
    return `${+line.isWrapped}~${cells.join(';') || '-'}`;
  }).join('|');
}
const cases = [];
const translations = [];
let current = 'unlabelled';
globalThis.__searchCapture = (terminal, method, args, invoke) => {
  const isFind = method === 'find';
  const options = args[isFind ? 3 : 1];
  const input = [terminal.cols, rows(terminal), hex(args[0]), method,
    isFind ? args[1] : 0, isFind ? args[2] : 0, opts(options),
    isFind || args[2] === undefined ? '~' : hex(args[2]), selected(terminal.getSelectionPosition())];
  try {
    const result = invoke();
    const expected = result ? `${hex(result.term)}/${result.row}/${result.col}/${result.size}` : 'none';
    cases.push([...input, expected, current].join('\t'));
    return result;
  } catch (error) {
    const expected = /Invalid col/.test(String(error)) ? 'error-column' : 'error-regex';
    cases.push([...input, expected, current].join('\t'));
    throw error;
  }
};
globalThis.__searchTranslate = (terminal, args, invoke) => {
  const input = [terminal.cols, rows(terminal), args[0], +args[1]];
  const result = invoke();
  translations.push([...input, hex(result[0]), result[1].join(','), current].join('\t'));
  return result;
};
const plugin = {
  name: 'observe-search', setup(api) {
    api.onLoad({ filter: /[/\\](SearchEngine|SearchLineCache)\.ts$/ }, args => {
      const original = readFileSync(args.path, 'utf8');
      sourceHashes[args.path.slice(root.length + 1).replaceAll('\\', '/')] = createHash('sha256').update(original).digest('hex');
      const append = args.path.endsWith('SearchEngine.ts') ? `
for (const method of ['find', 'findNextWithSelection', 'findPreviousWithSelection']) {
  const original = SearchEngine.prototype[method];
  SearchEngine.prototype[method] = function (...args) {
    return globalThis.__searchCapture(this._terminal, method, args, () => original.apply(this, args));
  };
}` : `
const originalTranslate = SearchLineCache.prototype.translateBufferLineToStringWithWrap;
SearchLineCache.prototype.translateBufferLineToStringWithWrap = function (...args) {
  return globalThis.__searchTranslate(this._terminal, args, () => originalTranslate.apply(this, args));
};`;
      return { contents: original + append, loader: 'ts', resolveDir: resolve(args.path, '..') };
    });
  }
};
const buildOptions = {
  absWorkingDir: root, bundle: true, platform: 'node', format: 'cjs', metafile: true,
  tsconfig: 'source_repo/src/browser/tsconfig.json', plugins: [plugin],
  alias: { common: resolve(root, 'source_repo/src/common'), browser: resolve(root, 'source_repo/src/browser') }
};
const outcomes = [];
const suites = [];
for (const suite of ['SearchEngine', 'SearchLineCache']) {
  const tests = `source_repo/addons/addon-search/src/${suite}.test.ts`;
  const outfile = resolve(temp, `${suite}.cjs`);
  const built = await build({ ...buildOptions, entryPoints: [tests], outfile });
  for (const file of Object.keys(built.metafile.inputs).filter(f => f.startsWith('source_repo/') && !f.includes('/node_modules/'))) {
    sourceHashes[file] = createHash('sha256').update(readFileSync(resolve(root, file))).digest('hex');
  }
  const mocha = new Mocha({ reporter: 'dot' });
  mocha.addFile(outfile);
  await mocha.loadFilesAsync();
  let runner;
  const failed = await new Promise(done => {
    runner = mocha.run(done);
    runner.on('test', test => { current = `upstream:${test.fullTitle()}`; });
    runner.on('pass', test => outcomes.push({ name: test.fullTitle(), status: 'passed' }));
    runner.on('pending', test => outcomes.push({ name: test.fullTitle(), status: 'pending' }));
    runner.on('fail', (test, error) => outcomes.push({ name: test.fullTitle(), status: 'failed', error: String(error.stack) }));
  });
  suites.push({ tests, discovered: runner.total, executed: runner.stats.tests - runner.stats.pending,
    passed: runner.stats.passes, failed, pending: runner.stats.pending, unrun: runner.total - runner.stats.tests });
}
const entry = resolve(temp, 'entry.ts');
writeFileSync(entry, `
export { SearchEngine } from '${resolve(root, 'source_repo/addons/addon-search/src/SearchEngine.ts').replaceAll('\\', '/')}';
export { SearchLineCache } from '${resolve(root, 'source_repo/addons/addon-search/src/SearchLineCache.ts').replaceAll('\\', '/')}';
export { SearchState } from '${resolve(root, 'source_repo/addons/addon-search/src/SearchState.ts').replaceAll('\\', '/')}';
export { BufferLine } from '${resolve(root, 'source_repo/src/common/buffer/BufferLine.ts').replaceAll('\\', '/')}';
export { CellData } from '${resolve(root, 'source_repo/src/common/buffer/CellData.ts').replaceAll('\\', '/')}';
`);
const outfile = resolve(temp, 'entry.cjs');
await build({ ...buildOptions, entryPoints: [entry], outfile });
const { SearchEngine, SearchLineCache, SearchState, BufferLine, CellData } = await import(pathToFileURL(outfile));
function terminalFor(lines, cols, selection) {
  const event = () => ({ dispose() {} });
  return { cols, rows: lines.length,
    buffer: { active: { length: lines.length, baseY: 0, getLine: row => lines[row] } },
    clearSelection() {}, getSelectionPosition: () => selection,
    onLineFeed: event, onCursorMove: event, onResize: event };
}
function narrowLine(text, cols, wrapped = false) {
  const line = new BufferLine(cols, undefined, wrapped);
  for (let col = 0; col < text.length; col++) {
    const cell = new CellData(); cell.setFromCharData([0, text[col], 1, 0]); line.setCell(col, cell);
  }
  return line;
}
function fixed(label, text, term, options = {}, method = 'find', col = 0, selection, cached) {
  current = `fixed:${label}`;
  const cols = 80;
  const terminal = terminalFor([narrowLine(text, cols)], cols, selection);
  const cache = new SearchLineCache(terminal);
  const engine = new SearchEngine(terminal, cache);
  try {
    if (method === 'find') engine.find(term, 0, col, options);
    else engine[method](term, options, cached);
  } catch (error) {
    if (!(error instanceof SyntaxError) && !/Invalid col/.test(String(error))) throw error;
  } finally { cache.dispose(); }
}
for (const delimiter of ' ~!@#$%^&*()+`-=[]{}|\\;:\"\',./<>?') {
  fixed(`delimiter-${delimiter.charCodeAt(0)}`, `x${delimiter}word${delimiter}y`, 'word', { wholeWord: true });
}
for (const delimiter of ['_', '\t', '中']) {
  fixed(`non-delimiter-${delimiter.charCodeAt(0)}`, `${delimiter}word${delimiter}`, 'word', { wholeWord: true });
}
fixed('reject-first-whole-word-candidate', 'aword word', 'word', { wholeWord: true });
fixed('contextual-final-sigma', 'ΟΣ ΟΣΑ', 'ος');
fixed('expanding-lowercase-indices', 'İB', 'b');
for (const pattern of ['(a)\\1', '(?<=a)b', '(?=a)', '^a', 'a$', 'aa', '.', '[invalid', '.*?']) {
  for (const method of ['find', 'findPreviousWithSelection']) {
    fixed(`${method}-${pattern}`, 'aaababa', pattern, { regex: true }, method);
  }
}
const selectionAtStart = { start: { x: 0, y: 0 }, end: { x: 3, y: 0 } };
for (const method of ['findNextWithSelection', 'findPreviousWithSelection']) {
  for (const cached of [undefined, 'aaa']) {
    fixed(`selection-${method}-${cached}`, 'aaa aaa', 'aaa', {}, method, 0, selectionAtStart, cached);
  }
}
for (const width of [1, 2]) {
  for (const chars of ['𝄞', 'e\u0301', '👩‍👩‍👧‍👦', '\ud800']) {
    current = `fixed:combined-${width}-${hex(chars)}`;
    const line = new BufferLine(8);
    const cell = new CellData(); cell.setFromCharData([0, chars, width, 0]); line.setCell(0, cell);
    if (width === 2) {
      const tail = new CellData(); tail.setFromCharData([0, '', 0, 0]); line.setCell(1, tail);
    }
    const terminal = terminalFor([line], 8);
    const cache = new SearchLineCache(terminal);
    const engine = new SearchEngine(terminal, cache);
    engine.find('.', 0, 0, { regex: true });
    engine.find(chars, 0, 0);
    cache.dispose();
  }
}
for (const wide of [false, true]) {
  current = `fixed:wrapped-wide-${wide}`;
  const first = narrowLine('abc', 4);
  const next = narrowLine('z', 4, true);
  if (wide) {
    const cell = new CellData(); cell.setFromCharData([0, '中', 2, 0]); next.setCell(0, cell);
    const tail = new CellData(); tail.setFromCharData([0, '', 0, 0]); next.setCell(1, tail);
  }
  const terminal = terminalFor([first, next], 4);
  const cache = new SearchLineCache(terminal);
  const engine = new SearchEngine(terminal, cache);
  cache.translateBufferLineToStringWithWrap(0, true);
  engine.find(wide ? 'c中' : 'c z', 0, 0);
  engine.find(wide ? '中' : 'z', 1, 0);
  engine.findPreviousWithSelection(wide ? '中' : 'z');
  cache.dispose();
}
const seed = 0x5ea2c001;
let state = seed;
const random = () => { state ^= state << 13; state ^= state >>> 17; state ^= state << 5; return state >>> 0; };
const strings = ['a', 'A', 'b', ' ', '中', '𝄞', 'e\u0301', 'Σ', 'ΟΣ', 'İ', '\ud800', '\udc00', '', '👩‍👩‍👧‍👦'];
const terms = ['a', 'A', '中', '𝄞', 'e\u0301', 'σ', 'ος', 'İ', '\ud800', ' ', '.', '[aA]+', '(a)\\1', '(?<=a)b', '(?=a)', '^', '[invalid', '.*?', 'a|b'];
for (let i = 0; i < 2048; i++) {
  current = `random:${i}`;
  const cols = 8 + random() % 13;
  const lines = Array.from({ length: 1 + random() % 5 }, (_, row) => {
    const line = new BufferLine(cols, undefined, row > 0 && random() % 3 === 0);
    for (let col = 0; col < cols; col++) {
      const chars = strings[random() % strings.length];
      const width = chars === '中' || chars === '𝄞' ? 2 : 1;
      if (width === 2 && col + 1 === cols) break;
      const cell = new CellData();
      cell.setFromCharData([0, chars, width, 0]);
      line.setCell(col, cell);
      if (width === 2) {
        const continuation = new CellData(); continuation.setFromCharData([0, '', 0, 0]);
        line.setCell(++col, continuation);
      }
    }
    return line;
  });
  const selection = i % 3 ? {
    start: { x: random() % cols, y: random() % lines.length },
    end: { x: random() % (cols + 1), y: random() % lines.length }
  } : undefined;
  const terminal = terminalFor(lines, cols, selection);
  const cache = new SearchLineCache(terminal);
  const engine = new SearchEngine(terminal, cache);
  const term = terms[random() % terms.length];
  const options = { caseSensitive: !!(random() & 1), regex: i % 3 === 0, wholeWord: i % 7 === 0 };
  const cached = i % 2 ? term : undefined;
  try {
    if (i % 3 === 0) engine.find(term, random() % (lines.length + 2) - 1, random() % (cols + 2) - 1, options);
    if (i % 3 === 1) engine.findNextWithSelection(term, options, cached);
    if (i % 3 === 2) engine.findPreviousWithSelection(term, options, cached);
    cache.translateBufferLineToStringWithWrap(random() % (lines.length + 1), !!(i % 2));
  } catch (error) {
    if (!(error instanceof SyntaxError) && !/Invalid col/.test(String(error))) throw error;
  } finally { cache.dispose(); }
}
const states = [];
for (let i = 0; i < 1024; i++) {
  const old = i % 11 === 0 ? undefined : {
    caseSensitive: [undefined, false, true][random() % 3], regex: [undefined, false, true][random() % 3],
    wholeWord: [undefined, false, true][random() % 3], incremental: !!(i % 2), decorations: i % 3 ? {} : undefined
  };
  const next = i % 13 === 0 ? undefined : {
    caseSensitive: [undefined, false, true][random() % 3], regex: [undefined, false, true][random() % 3],
    wholeWord: [undefined, false, true][random() % 3], incremental: !!(i % 2), decorations: i % 5 ? {} : undefined
  };
  const term = i % 7 ? terms[random() % terms.length] : '';
  const cached = i % 3 ? term : undefined;
  const search = new SearchState(); search.cachedSearchTerm = cached; search.lastSearchOptions = old;
  states.push([old ? opts(old) : '-', next ? opts(next) : '-', +(!!next?.decorations),
    cached === undefined ? '~' : hex(cached), hex(term), +search.isValidSearchTerm(term),
    +search.didOptionsChange(next), +search.shouldUpdateHighlighting(term, next)].join('\t'));
}
for (const [name, records] of [['search-engine.tsv', cases], ['search-translation.tsv', translations], ['search-state.tsv', states]]) {
  const corpus = records.join('\n') + '\n';
  if (process.argv.includes('--generate')) writeFileSync(resolve(fixtures, name), corpus);
  else assert.equal(readFileSync(resolve(fixtures, name), 'utf8').replaceAll('\r\n', '\n'), corpus, `${name}: retained corpus differs`);
}
const report = { node: process.version, platform: process.platform, overallEquivalence: 'NOT PROVEN',
  sourceHashes, seed: '0x5ea2c001', randomizedEngineCases: 2048, stateCases: states.length,
  engineCases: cases.length, translationCases: translations.length, suites, outcomes,
  mappedEngineTests: [...new Set(cases.filter(c => c.includes('\tupstream:')).map(c => c.split('\t').at(-1)))],
  mappedTranslationTests: [...new Set(translations.filter(c => c.includes('\tupstream:')).map(c => c.split('\t').at(-1)))],
  fixtures: Object.fromEntries(['search-engine.tsv', 'search-translation.tsv', 'search-state.tsv'].map(name =>
    [name, createHash('sha256').update(readFileSync(resolve(fixtures, name))).digest('hex')])),
  unrun: ['Addon/browser integration and decoration assertions under task59', 'Native Linux/macOS runtime'] };
writeFileSync(resolve(output, 'oracle.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ suites, engine: cases.length, translation: translations.length, states: states.length }));
assert(suites.every(s => s.failed === 0 && s.pending === 0 && s.unrun === 0));
