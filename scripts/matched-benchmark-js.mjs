// Actual production headless engine adapter. Setup and lossless capture are untimed.
import assert from 'node:assert/strict';
import { pathToFileURL } from 'node:url';
import { richSnapshot, observers } from './full-trace-source.mjs';

export function prepare(row) {
  assert.deepEqual(row.options.settings, { allowProposedApi: true });
  assert.deepEqual(row.options.unicode.settings, {});
  const { provider, version } = row.options.unicode;
  assert.ok((provider === 'builtin' && version === '6') ||
    (provider === 'unicode11' && version === '11') ||
    (provider === 'unicode-graphemes' && version === '15-graphemes'), 'Unsupported Unicode configuration');
  return row.operations.map(operation => {
    if (operation.kind === 'write') {
      const bytes = Buffer.from(operation.data, 'base64');
      return { ...operation, input: operation.encoding === 'utf16le' ? bytes.toString('utf16le') : Uint8Array.from(bytes) };
    }
    if (operation.kind === 'resize') return operation;
    assert.equal(operation.addon, 'serialize');
    assert.equal(operation.method, 'serialize');
    assert.deepEqual(operation.arguments, [], 'Only default serialization is measured');
    return operation;
  });
}
const units = text => Array.from({ length: text.length }, (_, i) => text.charCodeAt(i));
export async function loadEngine(artifacts) {
  const load = name => import(pathToFileURL(artifacts[name].path).href);
  const { Terminal } = await load('headless');
  const { Unicode11Addon } = await load('unicode11');
  const { UnicodeGraphemesAddon } = await load('unicode-graphemes');
  const { SerializeAddon } = await load('serialize');
  // Production bundles intentionally hide CHARSETS. Observe their actual shared
  // identities through a separate terminal, rather than rebundling a test engine.
  const charsets = {};
  const probe = new Terminal({ allowProposedApi: true });
  try {
    for (const id of '04567=ACEHKQRYZ') {
      await new Promise(done => probe.write(`\x1b(${id}`, done));
      charsets[id] = probe._core._charsetService.charset;
    }
  } finally { probe.dispose(); }
  return { Terminal, Unicode11Addon, UnicodeGraphemesAddon, SerializeAddon, charsets };
}
export async function sample(row, engine, operations = prepare(row), writeMode = 'public-callback') {
  assert.ok(['public-callback', 'synchronous-core'].includes(writeMode), 'Unsupported write timing mode');
  const { unicode, settings, ...size } = row.options;
  const terminal = new engine.Terminal({ ...size, ...settings });
  const events = [], outputs = [];
  let serializer;
  try {
    if (unicode.provider === 'unicode11') terminal.loadAddon(new engine.Unicode11Addon());
    if (unicode.provider === 'unicode-graphemes') terminal.loadAddon(new engine.UnicodeGraphemesAddon());
    terminal.unicode.activeVersion = unicode.version;
    if (operations.some(operation => operation.kind === 'addon')) {
      serializer = new engine.SerializeAddon();
      terminal.loadAddon(serializer);
    }
    observers(terminal, events);
    const start = process.hrtime.bigint();
    for (const operation of operations) {
      if (operation.kind === 'write') {
        if (writeMode === 'public-callback') await new Promise(done => terminal.write(operation.input, done));
        else terminal._core.writeSync(operation.input);
      } else if (operation.kind === 'resize') terminal.resize(operation.cols, operation.rows);
      else outputs.push(serializer.serialize());
    }
    const durationMs = Number(process.hrtime.bigint() - start) / 1e6;
    // These workloads use only builtin synchronous handlers. The optional
    // deprecated production writeSync path is explicitly an internal-core
    // comparison, never a public API or async-handler completion claim.
    assert.equal(terminal._core._writeBuffer._pendingData, 0, 'Queued writes remain');
    return { durationMs, completion: 'completed', pendingOperations: 0,
      output: { terminalTrace: `${richSnapshot(terminal, engine.charsets)}\t${events.join('/')}\n`,
        addons: serializer ? { serialize: outputs.map(units) } : {} } };
  } finally { terminal.dispose(); }
}
