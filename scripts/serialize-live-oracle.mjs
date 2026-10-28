// Compare actual source serialization and restoration, including source losses.
import { createRequire } from 'node:module';
import assert from 'node:assert/strict';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import path from 'node:path';
const root = path.resolve(import.meta.dirname, '..');
const require = createRequire(path.join(root, 'source_repo/package.json'));
const { build } = require('esbuild');
const temporary = path.join(root, 'tmp/serialize-live-oracle');
await mkdir(temporary, { recursive: true });
const bundle = path.join(temporary, 'reference.cjs');
const built = await build({
  stdin: { resolveDir: root, sourcefile: 'serialize-live-oracle.ts', contents: [
    "export { Terminal } from './source_repo/src/browser/public/Terminal.ts';",
    "export { SerializeAddon } from './source_repo/addons/addon-serialize/src/SerializeAddon.ts';"
  ].join('\n') },
  absWorkingDir: root, bundle: true, platform: 'node', format: 'cjs', target: 'node24',
  tsconfig: 'source_repo/src/browser/tsconfig.json',
  alias: { common: path.join(root, 'source_repo/src/common'), browser: path.join(root, 'source_repo/src/browser') },
  outfile: bundle, metafile: true, logLevel: 'silent'
});
const sourceHashes = {};
for (const file of Object.keys(built.metafile.inputs).filter(file => file.startsWith('source_repo/'))) {
  sourceHashes[file] = createHash('sha256').update(await readFile(path.join(root, file))).digest('hex');
}
const { Terminal, SerializeAddon } = require(bundle);
const units = text => Array.from({ length: text.length }, (_, i) => text.charCodeAt(i));
const write = (terminal, input) => new Promise(resolve => terminal.write(input, resolve));
function buffer(value) {
  return {
    cursorX: value.cursorX, cursorY: value.cursorY, baseY: value.baseY, viewportY: value.viewportY,
    lines: Array.from({ length: value.length }, (_, row) => {
      const line = value.getLine(row);
      return { wrapped: line.isWrapped, cells: Array.from({ length: line.length }, (_, col) => {
        const cell = line.getCell(col);
        return { chars: units(cell.getChars()), width: cell.getWidth(), code: cell.getCode(),
          fg: cell.fg >>> 0, bg: cell.bg >>> 0, underlineStyle: cell.getUnderlineStyle(),
          underlineColor: cell.getUnderlineColor(), underlineColorMode: cell.getUnderlineColorMode() };
      }) };
    })
  };
}
function state(terminal) {
  const active = terminal._core.buffer;
  return { normal: buffer(terminal.buffer.normal), alternate: buffer(terminal.buffer.alternate),
    active: terminal.buffer.active.type, modes: terminal.modes,
    scrollTop: active.scrollTop, scrollBottom: active.scrollBottom };
}
const inputs = [
  'hello\r\nworld', '\x1b[31mred\x1b[0m plain', 'a\x1b[3Cz\x1b[2D',
  'abcdefghijwrap中é', '0\r\n1\r\n2\r\n3\r\n4',
  'normal\x1b[?1049h\x1b[Halt', '\x1b[?1h\x1b[?66h\x1b[?2004h\x1b[4h\x1b[?25lhello',
  '\x1b[2;4r\x1b[3;3Hmargin', '\x1b[4:3;58:2::255:128:64mtest\x1b[24m',
  '0\r\n1\r\n2\r\n3\r\n4\r\n5\r\n6\r\n7\r\n8'
];
const cases = [];
for (const input of inputs) {
  const original = new Terminal({ cols: 10, rows: 5, scrollback: 20, allowProposedApi: true });
  const addon = new SerializeAddon(); original.loadAddon(addon);
  await write(original, input);
  const before = state(original);
  const serialized = addon.serialize();
  const restored = new Terminal({ cols: 10, rows: 5, scrollback: 20, allowProposedApi: true });
  await write(restored, serialized);
  const after = state(restored);
  const adjusted = structuredClone(before);
  adjusted.normal.cursorX = after.normal.cursorX;
  adjusted.normal.cursorY = after.normal.cursorY;
  assert.deepEqual(adjusted, after, 'Unexpected source round-trip loss beyond the known cursor change');
  cases.push({ input: units(input), serialized: units(serialized), before, after,
    losslessForObservedState: JSON.stringify(before) === JSON.stringify(after) });
  original.dispose(); restored.dispose();
}
const reentrantTerminal = new Terminal({ cols: 10, rows: 2, scrollback: 20, allowProposedApi: true });
const reentrantAddon = new SerializeAddon(); reentrantTerminal.loadAddon(reentrantAddon);
await write(reentrantTerminal, 'normal');
// Public writes are asynchronous in JS. The header getter performs the same
// already-synchronous input-handler operation used by Rust's native write API.
reentrantTerminal.options.theme = { get foreground() {
  reentrantTerminal._core._inputHandler.parse('\x1b[?1049halt');
  reentrantTerminal.resize(12, 3);
  reentrantTerminal.options.allowProposedApi = false;
  return undefined;
} };
const reentrantHtml = reentrantAddon.serializeAsHTML({ includeGlobalBackground: true });
await writeFile(path.join(root, 'crates/xterm-core/tests/fixtures/serialize-html-reentrant.tsv'), units(reentrantHtml).map(u => u.toString(16)).join(',') + '\n');
reentrantTerminal.dispose();
const orderedTerminal = new Terminal({ cols: 10, rows: 2, allowProposedApi: true });
const orderedAddon = new SerializeAddon(); orderedTerminal.loadAddon(orderedAddon);
const orderedTrace = [];
const orderedValue = (label, text) => ({ [Symbol.toPrimitive](hint) {
  orderedTrace.push(`${label}:${hint}`);
  return hint === 'default' ? text : hint === 'number' ? 15 : 'wrong-string';
} });
const foregroundValue = orderedValue('fg', '#112233');
const backgroundValue = orderedValue('bg', '#445566');
const replacementTheme = { get background() { orderedTrace.push('new:background'); return backgroundValue; } };
orderedTerminal.options.theme = {
  get foreground() { orderedTrace.push('old:foreground'); orderedTerminal.options.theme = replacementTheme; return foregroundValue; },
  get background() { orderedTrace.push('old:background'); return undefined; }
};
orderedTerminal.options.fontFamily = orderedValue('font', 'fancy');
orderedTerminal.options.fontSize = orderedValue('size', 'weird');
orderedTrace.length = 0;
const orderedHtml = orderedAddon.serializeAsHTML({ includeGlobalBackground: true });
assert.deepEqual(orderedTrace, ['old:foreground', 'new:background', 'fg:default', 'bg:default', 'font:default', 'size:default']);
assert.equal(orderedHtml, "<html><body><!--StartFragment--><pre><div style='color: #112233; background-color: #445566; font-family: fancy; font-size: weirdpx;'><div><span>          </span></div><div><span>          </span></div></div></pre><!--EndFragment--></body></html>");
orderedTerminal.dispose();
for (const [file, expected] of Object.entries(sourceHashes)) {
  const current = createHash('sha256').update(await readFile(path.join(root, file))).digest('hex');
  if (current !== expected) throw new Error(`Source changed while running: ${file}`);
}
const result = { command: 'node scripts/serialize-live-oracle.mjs', node: process.version,
  platform: process.platform, sourceHashes, cases, reentrantHtml: units(reentrantHtml),
  headerOrder: { trace: orderedTrace, html: units(orderedHtml) },
  scope: 'Ten deterministic source engine serialize/restore traces. A source loss is an expected compatibility result, not permission to change Rust output. Rust integration tests must retain corresponding assertions.' };
await writeFile(path.join(root, 'results/serialize/live-source.json'), JSON.stringify(result, null, 2) + '\n');
const hex = units => units.map(unit => unit.toString(16)).join(',');
await writeFile(path.join(root, 'crates/xterm-core/tests/fixtures/serialize-live.tsv'), cases.map((c, id) => [
  id, hex(c.input), hex(c.serialized), c.before.normal.cursorX, c.before.normal.cursorY,
  c.after.normal.cursorX, c.after.normal.cursorY, +c.losslessForObservedState
].join('\t')).join('\n') + '\n');
console.log(JSON.stringify(cases.map((c, id) => ({ id, lossless: c.losslessForObservedState,
  beforeCursor: [c.before.normal.cursorX, c.before.normal.cursorY], afterCursor: [c.after.normal.cursorX, c.after.normal.cursorY] }))));
