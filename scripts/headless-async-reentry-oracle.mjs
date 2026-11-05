// Source-only directed native recursive-entry fixture. No upstream inputs change.
import { createRequire } from 'node:module';
import { mkdir, readFile, writeFile, rm } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import path from 'node:path';

const source = path.resolve(process.env.XTERM_SOURCE_ROOT ?? 'source_repo');
const require = createRequire(path.join(source, 'package.json'));
const { build } = require('esbuild');
const temporary = path.resolve('tmp', `headless-async-reentry-${process.pid}`);
const fixture = 'crates/xterm-core/tests/fixtures/headless-async-reentry.tsv';
const reportPath = 'results/headless-async-reentry/source.json';
const cases = [
  ['async-text', 'text', 'async', 'resize', 0],
  ['async-bytes', 'bytes', 'async', 'resize', 0],
  ['sync-resize', 'text', 'sync', 'resize', 0],
  ['idle-resize', 'text', 'resize', 'observe', 0],
  ['same-size', 'text', 'same', 'observe', 0],
  ['async-tail', 'text', 'async', 'resize', 1],
  ['async-grow', 'text', 'async', 'resize', 5000],
  ['async-grow-limit', 'text', 'async', 'resize', 140000],
  ['async-input', 'text', 'async', 'input', 0],
  ['async-reset', 'text', 'async', 'reset', 0],
  ['async-dispose', 'text', 'async', 'dispose', 0],
  ['nested-reset', 'text', 'async', 'nested-reset', 0],
  ['nested-dispose', 'text', 'async', 'nested-dispose', 0],
  ['append-before-resize', 'text', 'async', 'append', 0],
  ['nested-error', 'text', 'async', 'error', 0],
  ['async-unicode', 'unicode', 'async', 'resize', 0]
];
function hex(text) {
  return text.length ? Array.from({length:text.length}, (_, i) => text.charCodeAt(i).toString(16)).join(',') : '-';
}
let report = { command:'node scripts/headless-async-reentry-oracle.mjs', node:process.version,
  platform:process.platform, status:'running', discovered:cases.length, executed:0, sourceHashes:{}, outcomes:[] };
await mkdir(path.dirname(reportPath), {recursive:true});
await writeFile(reportPath, JSON.stringify(report, null, 2)+'\n');
try {
  await mkdir(temporary, {recursive:true});
  const outfile = path.join(temporary, 'source.cjs');
  const bundled = await build({ absWorkingDir:source, entryPoints:['src/headless/public/Terminal.ts'], outfile,
    bundle:true, platform:'node', format:'cjs', tsconfig:'src/headless/tsconfig.json', metafile:true, logLevel:'silent' });
  for (const file of Object.keys(bundled.metafile.inputs).sort()) {
    report.sourceHashes[file] = createHash('sha256').update(await readFile(path.resolve(source, file))).digest('hex');
  }
  const { Terminal } = require(outfile);
  const rows = [];
  for (const [name, kind, delivery, mutation, tail] of cases) {
    const term = new Terminal({cols:8, rows:3, scrollback:10, allowProposedApi:true});
    const queue = term._core._writeBuffer;
    const log = [];
    let bells = 0;
    term.onResize(e => log.push(`resize:${e.cols},${e.rows}`));
    term.onData(data => log.push(`data:${hex(data)}`));
    term.onBell(() => {
      log.push('bell');
      bells++;
      if (mutation === 'resize') term.resize(6, 3);
      if (mutation === 'input') term.input('in');
      if (mutation === 'reset') term.reset();
      if (mutation === 'dispose') term.dispose();
      if (mutation === 'nested-reset' || mutation === 'nested-dispose' || mutation === 'error') {
        if (bells === 1) term.resize(6, 3);
        else if (mutation === 'nested-reset') term.reset();
        else if (mutation === 'nested-dispose') term.dispose();
        else throw new Error('nested listener failure');
      }
      if (mutation === 'append' && bells === 1) {
        term.write('C', () => log.push('inner-callback'));
        term.resize(6, 3);
      }
    });
    let failure = '-';
    try {
      const text = kind === 'unicode' ? '界\x07\u0301Z' : 'A\x07B';
      term.write(kind === 'bytes' ? new Uint8Array(Buffer.from(text)) : text, () => log.push('callback'));
      if (tail) term.write('C'.repeat(tail), () => log.push('tail-callback'));
      if (delivery === 'sync') queue.flushSync();
      else if (delivery === 'resize') term.resize(6, 3);
      else {
        if (delivery === 'same') { term.resize(8, 3); log.push(`before:${queue._pendingData}`); }
        // The original timer callback body, with no timing guess or sleeps.
        queue._innerWrite();
      }
    } catch (error) { failure = error.message; }
    const buffer = term.buffer.active;
    const lines = Array.from({length:buffer.length}, (_, i) => hex(buffer.getLine(i).translateToString(false))).join(';');
    const result = [name, kind, delivery, mutation, tail, term.cols, term.rows, buffer.cursorX,
      buffer.cursorY, buffer.baseY, buffer.viewportY, Number(term._core._store.isDisposed),
      queue._pendingData, queue._bufferOffset, queue._writeBuffer.length,
      lines, log.join('|') || '-', failure];
    rows.push(result.join('\t'));
    report.outcomes.push({name, columns:result});
    report.executed++;
    term.dispose();
  }
  const bytes = rows.join('\n')+'\n';
  if (process.argv.includes('--generate')) await writeFile(fixture, bytes);
  else if (await readFile(fixture, 'utf8') !== bytes) throw new Error('recursive-entry source fixture drift');
  report.fixtureSha256 = createHash('sha256').update(bytes).digest('hex');
  report.status = 'passed';
  console.log(`${cases.length}/${cases.length} unchanged-source recursive-entry cases verified`);
} catch (error) {
  report.status = 'failed'; report.error = error.stack; throw error;
} finally {
  await writeFile(reportPath, JSON.stringify(report, null, 2)+'\n');
  await rm(temporary, {recursive:true, force:true});
}
