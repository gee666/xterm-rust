// Lossless reference projection for the native full_trace acceptance protocol.
import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import assert from 'node:assert/strict';
import { snapshot as baseSnapshot, hex } from './terminal-snapshot.mjs';
import { referenceIntegrity } from './upstream-baseline-utils.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const require = createRequire(path.join(root, 'source_repo/package.json'));
const digest = data => createHash('sha256').update(data).digest('hex');
import { decodeUnits, fromUnits } from './full-trace-codec.mjs';
export { decodeUnits, fromUnits } from './full-trace-codec.mjs';

function mapId(map, charsets) {
  if (map === undefined) return '-';
  const id = [...'04567=ACEHKQRYZ'].find(id => charsets[id] === map);
  if (id !== undefined) return id;
  throw new Error('Custom charset capture requires an explicit identity mapping');
}
function saved(buffer, charsets) {
  const attr = buffer.savedCurAttrData;
  return [[buffer.savedX, buffer.savedY].join(','),
    [attr.fg >>> 0, attr.bg >>> 0, attr.extended.ext >>> 0, attr.extended.urlId].join(','),
    mapId(buffer.savedCharset, charsets),
    Array.from(buffer.savedCharsets, map => mapId(map, charsets)).join(','), buffer.savedGlevel,
    [+buffer.savedOriginMode, +buffer.savedWraparoundMode].join(','),
    Object.keys(buffer.tabs).filter(key => buffer.tabs[key]).map(Number).sort((a, b) => a - b).join(','),
    buffer.lines.maxLength].join('~');
}
export function richSnapshot(terminal, charsets) {
  const core = terminal._core, state = core._charsetService;
  const modes = core.coreService.decPrivateModes, handler = core._inputHandler;
  const meta = [mapId(state.charset, charsets), Array.from(state.charsets, map => mapId(map, charsets)).join(','), state.glevel,
    hex(handler._windowTitle), hex(handler._iconName), modes.cursorStyle ?? '-',
    modes.cursorBlink === undefined ? '-' : +modes.cursorBlink, +modes.colorSchemeUpdates, +modes.win32InputMode,
    +core.coreService.isCursorInitialized, +core.coreService.isCursorHidden,
    hex(terminal.options.cursorStyle), +terminal.options.cursorBlink, hex(core.unicodeService.activeVersion)].join('~');
  return ['RICH2', baseSnapshot(terminal), saved(core.buffers.normal, charsets), saved(core.buffers.alt, charsets),
    meta, +(core.buffers.active === core.buffers.alt)].join('|');
}
function observers(terminal, events) {
  const core = terminal._core, input = core._inputHandler;
  terminal.onBell(() => events.push('B'));
  terminal.onCursorMove(() => events.push('C'));
  terminal.onLineFeed(() => events.push('L'));
  terminal.onScroll(y => events.push(`S${y}`));
  terminal.onData(value => events.push(`D${hex(value)}`));
  terminal.onBinary(value => events.push(`BIN${hex(value)}`));
  terminal.onTitleChange(value => events.push(`T${hex(value)}`));
  terminal.onResize(({ cols, rows }) => events.push(`SIZE${cols}:${rows}`));
  core.onRender(({ start, end }) => events.push(`R${start}:${end}`));
  input.onRequestReset(() => events.push('RESET'));
  input.onRequestSyncScrollBar(() => events.push('SYNC'));
  input.onRequestSendFocus(() => events.push('FOCUS'));
  input.onRequestColorSchemeQuery(() => events.push('SCHEME'));
  input.onRequestWindowsOptionsReport(kind => events.push(`WIN${kind}`));
  input.onA11yChar(value => events.push(`A${hex(value)}`));
  input.onA11yTab(amount => events.push(`TAB${amount}`));
}
function optionValue(kind, data) {
  if (kind === 'b') {
    if (!['0', '1'].includes(data)) throw new Error('Invalid boolean encoding');
    return data === '1';
  }
  if (kind === 'n') return Number(data);
  if (kind === 's') return fromUnits(decodeUnits(data));
  if (kind === 'm') return Object.fromEntries(data ? data.split(',').map(item => {
    const [key, flag] = item.split('=');
    if (!key || !['0', '1'].includes(flag)) throw new Error('Invalid option map encoding');
    return [key, flag === '1'];
  }) : []);
  throw new Error(`Unknown option kind: ${kind}`);
}
export async function runSource(protocol, implementation) {
  let terminal;
  const output = [], events = [];
  try {
    const lines = protocol.split(/\r?\n/);
    if (lines.at(-1) === '') lines.pop();
    for (const line of lines) {
      const fields = line.split('\t'), kind = fields[0];
      if (kind === 'N') {
        terminal?.dispose();
        terminal = new implementation.Terminal({ cols: Number(fields[1]), rows: Number(fields[2]),
          scrollback: Number(fields[3]), allowProposedApi: true });
        observers(terminal, events);
        continue;
      }
      if (!terminal) throw new Error('Operation before construction');
      events.length = 0;
      if (kind === 'W' || kind === 'B') {
        const units = decodeUnits(fields[1]);
        const input = kind === 'W' ? fromUnits(units) : Uint8Array.from(units);
        await new Promise(done => terminal.write(input, done));
      } else if (kind === 'R') terminal.resize(Number(fields[1]), Number(fields[2]));
      else if (kind === 'S') terminal.scrollLines(Number(fields[1]));
      else if (kind === 'C') terminal.clear();
      else if (kind === 'Q') terminal.reset();
      else if (kind === 'O') terminal.options[fields[1]] = optionValue(fields[2], fields[3]);
      else throw new Error(`Unknown trace operation: ${kind}`);
      output.push(`${richSnapshot(terminal, implementation.CHARSETS)}\t${events.join('/')}`);
    }
  } finally { terminal?.dispose(); }
  return output.join('\n') + '\n';
}
function checkOriginal() {
  const manifest = JSON.parse(fs.readFileSync(path.join(root, 'docs/baseline/reference-manifest.json'), 'utf8'));
  const observation = referenceIntegrity(path.join(root, 'source_repo'), manifest);
  assert.equal(observation.mismatches.length, 0,
    `Original source changed during the terminal trace: ${JSON.stringify(observation.mismatches)}`);
  return observation;
}
export async function loadReference(directory) {
  checkOriginal();
  fs.mkdirSync(directory, { recursive: true });
  const file = path.join(directory, 'reference.cjs');
  const { build } = require('esbuild');
  const terminal = path.join(root, 'source_repo/src/headless/public/Terminal.ts').replaceAll('\\', '/');
  const charsets = path.join(root, 'source_repo/src/common/data/Charsets.ts').replaceAll('\\', '/');
  const result = await build({ absWorkingDir: root,
    stdin: { contents: `export { Terminal } from ${JSON.stringify(terminal)}; export { CHARSETS } from ${JSON.stringify(charsets)};`,
      resolveDir: root, sourcefile: 'reference-entry.ts' },
    outfile: file, bundle: true, platform: 'node', format: 'cjs', target: 'node24',
    tsconfig: 'source_repo/src/headless/tsconfig.json', metafile: true, logLevel: 'silent' });
  checkOriginal();
  const retained = new Map(JSON.parse(fs.readFileSync(path.join(root, 'docs/baseline/reference-manifest.json'), 'utf8'))
    .map(entry => [`source_repo/${entry.path}`, entry]));
  const referenceRoot = fs.realpathSync(path.join(root, 'source_repo'));
  const hashes = Object.keys(result.metafile.inputs).filter(input => input !== 'reference-entry.ts').sort().map(input => {
    const physical = fs.realpathSync(path.resolve(root, input));
    const relative = path.relative(referenceRoot, physical).replaceAll('\\', '/');
    assert.ok(relative && !relative.startsWith('../') && !path.isAbsolute(relative), `Reference input escaped its root: ${input}`);
    const logical = `source_repo/${relative}`;
    const original = retained.get(logical);
    assert.ok(original, `Unexpected reference bundle input outside the retained source: ${input}`);
    const sha256 = digest(fs.readFileSync(physical));
    assert.equal(sha256, original.sha256, `Reference bundle input drift: ${input}`);
    return { path: logical, sha256 };
  });
  return { implementation: require(file), sourceHashes: hashes };
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [input, output, hashes, temporary] = process.argv.slice(2);
  if (!input || !output || !hashes || !temporary) throw new Error('Usage: full-trace-source INPUT OUTPUT HASHES TMPDIR');
  const reference = await loadReference(path.resolve(temporary));
  const result = await runSource(fs.readFileSync(input, 'utf8'), reference.implementation);
  const integrity = checkOriginal();
  fs.writeFileSync(path.join(path.dirname(hashes), 'full-trace-integrity.json'), JSON.stringify(integrity, null, 2) + '\n');
  fs.writeFileSync(output, result);
  fs.writeFileSync(hashes, JSON.stringify(reference.sourceHashes, null, 2) + '\n');
}
