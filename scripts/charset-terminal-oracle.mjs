// Live source-terminal charset behavior, including full cell/state snapshots.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { createHash } from 'node:crypto';
import { writeFileSync } from 'node:fs';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { hex, snapshot } from './terminal-snapshot.mjs';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const require = createRequire(path.join(root, 'source_repo/package.json'));
const temp = path.join(root, 'tmp/charset-terminal');
const results = path.join(root, 'results/charset-terminal');
await mkdir(results, { recursive: true });
const receipt = path.join(results, 'last-attempt.json');
await writeFile(receipt, JSON.stringify({ status: 'running', command: 'node scripts/charset-terminal-oracle.mjs' }) + '\n');
process.once('uncaughtExceptionMonitor', error => {
  writeFileSync(receipt, JSON.stringify({ status: 'failed', command: 'node scripts/charset-terminal-oracle.mjs', error: String(error.stack) }) + '\n');
});
await mkdir(temp, { recursive: true });
const { build } = require('esbuild');
const bundle = path.join(temp, 'source.cjs');
const built = await build({ absWorkingDir: root, stdin: { resolveDir: root, loader: 'ts', contents:
  "export {Terminal} from './source_repo/src/headless/public/Terminal';\nexport {CHARSETS} from './source_repo/src/common/data/Charsets';" },
  outfile: bundle, bundle: true, platform: 'node', format: 'cjs', target: 'node24',
  tsconfig: 'source_repo/src/headless/tsconfig.json', metafile: true, logLevel: 'silent' });
const hashes = {};
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
for (const file of Object.keys(built.metafile.inputs).filter(file => file.startsWith('source_repo/'))) hashes[file] = sha(await readFile(path.join(root, file)));
const { Terminal, CHARSETS } = require(bundle);
const ids = Object.keys(CHARSETS).sort();
const custom = { '!': '', '?': 'ij', '#': '\u0000x', '@': '\ud83d\ude00', '[': '\ud800', ']': '\udc00', '~': 'z' };
function canonical(map) {
  if (map === undefined) return '-';
  if (map === custom) return 'custom';
  const id = ids.find(id => CHARSETS[id] === map);
  assert.notEqual(id, undefined, 'Unexpected source map identity');
  return id;
}
const list = values => Array.from(values, canonical).join(',');
const service = t => {
  const s = t._core._charsetService;
  return `${s.glevel}/${canonical(s.charset)}/${list(s.charsets)}`;
};
const saved = b => `${b.savedGlevel}/${canonical(b.savedCharset)}/${list(b.savedCharsets)}`;
const printable = Array.from({ length: 95 }, (_, i) => String.fromCharCode(i + 32)).join('');
const cases = [];
const add = (name, text, options = {}) => cases.push({ name, text, cols: options.cols || 100, rows: 3, custom: !!options.custom });
for (const [intermediate, shift] of [['(', '\x0f'], [')', '\x0e'], ['-', '\x0e'], ['*', '\x1bn'], ['.', '\x1b}'], ['+', '\x1bo'], ['/', '\x1bo']]) {
  for (const id of ids) add(`designate:${intermediate}:${id}`, `\x1b${intermediate}${id}${shift}${printable}`);
}
for (const shift of ['\x0e', '\x0f', '\x1bn', '\x1b}', '\x1bo', '\x1b|', '\x1b~']) {
  add(`shift:${hex(shift)}`, `\x1b(4\x1b)0\x1b*A\x1b+K${shift}#@[\\]^\x7f\u0080😀`);
}
for (const final of ['G', '@']) {
  add(`default-preserves-designations:${final}`, `\x1b)0\x1b*A\x1b+K\x0eq\x1b%${final}\x0eq\x1bn#\x1bo@`);
}
add('default-only-G0', '\x1b(0q\x1b%Gq');
add('unknown-designation-unhandled', '\x1b(0q\x1b(Xq');
add('slash-does-not-designate-G3', '\x1b+K\x1b/0\x1bo@');
add('save-restore-slot-copy-and-level', '\x1b)0\x0e\x1b7\x1b)A\x0f#\x1b8q');
add('restore-preserves-extra-slot-length', '\x1b)0\x0e\x1b7\x1b+K\x1b8q\x1bo@');
add('default-select-after-save', '\x1b)0\x0e\x1b7\x1b%G\x1b8q');
add('soft-reset-clears-designations', '\x1b(0\x1b)A\x0e#\x1b[!p#q');
add('full-reset-clears-designations', '\x1b(0\x1b)A\x0e#\x1bc#q');
add('decset2-defaults-all-G', '\x1b(0\x1b)0\x1b*A\x1b+K\x1bo@\x1b[?2h@\x0eq');
add('alternate-save-restore', '\x1b)0\x0e\x1b[?1049h\x1b)A#\x1b[?1049lq');
add('wide-wrapping', '\x1b(0abcdefg中q\x1b(B😀', { cols: 8 });
add('custom-map-first-unit-and-empty', '!?#@[]~\u00ad\x7f\u0080', { custom: true });
add('custom-map-save-clone-identity', '\x1b7?\x1b(B?\x1b8?', { custom: true });
const lines = [];
const sourceLogs = [];
const logger = (id, profile) => Object.fromEntries(['trace', 'debug', 'info', 'warn', 'error'].map(level =>
  [level, (...args) => sourceLogs.push({ id, profile, level, args })]));
let sourceWrites = 0;
for (let id = 0; id < cases.length; id++) {
  const test = cases[id];
  let expected;
  for (const profile of ['string', 'units', 'bytes']) {
    const t = new Terminal({ cols: test.cols, rows: test.rows, scrollback: 10, allowProposedApi: true, screenReaderMode: true, logger: logger(id, profile) });
    if (test.custom) t._core._charsetService.setgCharset(0, custom);
    const a11y = [];
    t._core._inputHandler.onA11yChar(char => a11y.push(hex(char)));
    const chunks = profile === 'string' ? [test.text] : profile === 'units'
      ? Array.from({ length: test.text.length }, (_, i) => test.text[i])
      : Array.from(Buffer.from(test.text, 'utf8'), byte => Uint8Array.of(byte));
    await new Promise(resolve => chunks.forEach((chunk, i) => t.write(chunk, i === chunks.length - 1 ? resolve : undefined)));
    sourceWrites += chunks.length;
    const actual = [snapshot(t), service(t), saved(t._core.buffers.normal), saved(t._core.buffers.alt), a11y.join(';')].join('\t');
    if (expected === undefined) expected = actual;
    else assert.equal(actual, expected, `Source partition drift ${test.name} ${profile}`);
    t.dispose();
  }
  lines.push([id, test.name, test.cols, test.rows, test.custom ? 1 : 0, hex(test.text), expected].join('\t'));
}
const callbackRows = [];
const callbackInput = '\x1b(0qq';
for (const profile of ['string', 'units', 'bytes']) {
  const t = new Terminal({ cols: 8, rows: 3, scrollback: 10, allowProposedApi: true, screenReaderMode: true, logger: logger('callback', profile) });
  const observations = [];
  t._core._inputHandler.onA11yChar(char => {
    const b = t._core.buffer;
    const atCursor = b.lines.get(b.ybase + b.y)?.getCell(b.x)?.getChars();
    observations.push([hex(char), b.x, b.y, atCursor === undefined ? 'missing' : hex(atCursor), service(t)].join('/'));
    if (observations.length === 1) t._core._charsetService.setgCharset(0, CHARSETS.A);
  });
  const chunks = profile === 'string' ? [callbackInput] : profile === 'units' ? callbackInput.split('')
    : Array.from(Buffer.from(callbackInput, 'utf8'), byte => Uint8Array.of(byte));
  await new Promise(resolve => chunks.forEach((chunk, i) => t.write(chunk, i === chunks.length - 1 ? resolve : undefined)));
  sourceWrites += chunks.length;
  callbackRows.push([profile, hex(callbackInput), observations.join(';'), snapshot(t), service(t)].join('\t'));
  t.dispose();
}
const callbackFixture = 'crates/xterm-core/tests/fixtures/charset-callback.tsv';
const callbackText = callbackRows.join('\n') + '\n';
if (process.argv.includes('--generate')) await writeFile(path.join(root, callbackFixture), callbackText);
else assert.equal(await readFile(path.join(root, callbackFixture), 'utf8'), callbackText, 'Charset callback fixture drift');
const fixture = 'crates/xterm-core/tests/fixtures/charset-terminal.tsv';
const text = lines.join('\n') + '\n';
if (process.argv.includes('--generate')) await writeFile(path.join(root, fixture), text);
else assert.equal(await readFile(path.join(root, fixture), 'utf8'), text, 'Charset terminal fixture drift');
for (const [file, expected] of Object.entries(hashes)) assert.equal(sha(await readFile(path.join(root, file))), expected, `Reference changed ${file}`);
const report = { command: `node scripts/charset-terminal-oracle.mjs${process.argv.includes('--generate') ? ' --generate' : ''}`,
  node: process.version, platform: process.platform, arch: process.arch, sourceHashes: hashes,
  cases: cases.length, sourceProfiles: 3, sourceRuns: cases.length * 3 + 3, stateRuns: cases.length * 3,
  callbackRuns: 3, sourceWrites, fixture, fixtureSha256: sha(text), callbackFixture, callbackSha256: sha(callbackText), sourceLogs,
  custom: Object.fromEntries(Object.entries(custom).map(([key, value]) => [hex(key), hex(value)])),
  scope: 'Actual headless source parser/buffers/charsets/saved state/a11y under whole UTF16, split UTF16 and split UTF8. Rust native comparison and browser proof are separate.' };
await writeFile(path.join(results, 'oracle.json'), JSON.stringify(report, null, 2) + '\n');
await writeFile(receipt, JSON.stringify({ status: 'passed', fixtureSha256: report.fixtureSha256 }) + '\n');
console.log(JSON.stringify({ cases: report.cases, sourceRuns: report.sourceRuns, sourceWrites, fixtureSha256: report.fixtureSha256 }));
