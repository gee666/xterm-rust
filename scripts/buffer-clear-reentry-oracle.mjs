// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Reuse the reviewed real-engine bundle produced by buffer-terminal-oracle.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { snapshot } from './terminal-snapshot.mjs';
const root = resolve(import.meta.dirname, '..');
const require = createRequire(import.meta.url);
mkdirSync(resolve(root, 'results/buffer-terminal'), { recursive: true });
writeFileSync(resolve(root, 'results/buffer-terminal/clear-reentry-source.json'), JSON.stringify({
  status: 'incomplete', overallEquivalence: 'NOT PROVEN',
}) + '\n');
const metadata = JSON.parse(readFileSync(resolve(root, 'results/buffer-terminal/oracle.json'), 'utf8'));
for (const [file, expected] of Object.entries(metadata.sourceHashes)) {
  assert.equal(createHash('sha256').update(readFileSync(resolve(root, file))).digest('hex'), expected, 'regenerate real source bundle after source drift');
}
const bundle = resolve(root, 'tmp/buffer-terminal/reference.cjs');
const { Terminal } = require(bundle);
const rows = [];
for (const action of ['read', 'scroll', 'resize', 'clear', 'reset']) {
  const terminal = new Terminal({ cols: 8, rows: 3, scrollback: 4, allowProposedApi: true });
  await new Promise(done => terminal.write('old0\r\nold1\r\nold2\r\nprompt', done));
  const marker = terminal.registerMarker();
  const events = [];
  function capture(tag) {
    const markers = terminal.markers.map(value => `${value.line},${+value.isDisposed}`).join('|') || '-';
    events.push(`${tag}#${snapshot(terminal)}#${markers}`);
  }
  terminal.onScroll(position => capture(`scroll,${position}`));
  marker.onDispose(() => {
    capture('dispose');
    if (action === 'scroll') terminal.scrollToTop();
    else if (action === 'resize') terminal.resize(4, 4);
    else if (action === 'clear') terminal.clear();
    else if (action === 'reset') terminal.reset();
    capture('callback-return');
  });
  terminal.clear();
  capture('final');
  assert.equal(marker.isDisposed, true);
  rows.push(`${action}\t${events.join('^')}`);
  terminal.dispose();
}
const fixture = resolve(root, 'crates/xterm-core/tests/fixtures/buffer-clear-reentry.tsv');
const content = rows.join('\n') + '\n';
if (process.argv.includes('--generate')) writeFileSync(fixture, content);
else assert.equal(readFileSync(fixture, 'utf8'), content, 'source clear callback fixture drift');
const report = { status: 'passed', command: `node scripts/buffer-clear-reentry-oracle.mjs${process.argv.includes('--generate') ? ' --generate' : ''}`,
  node: process.version, platform: process.platform, sourceHashes: metadata.sourceHashes,
  bundleSha256: createHash('sha256').update(readFileSync(bundle)).digest('hex'),
  cases: rows.map(row => row.split('\t')[0]), count: rows.length,
  fixture: 'crates/xterm-core/tests/fixtures/buffer-clear-reentry.tsv', fixtureSha256: createHash('sha256').update(content).digest('hex'),
  scope: 'Actual public headless clear, marker-disposal callback reads/nested scroll/resize/clear/reset and exact callback-time cells/markers/order. Rust execution separate, overall equivalence NOT PROVEN.' };
mkdirSync(resolve(root, 'results/buffer-terminal'), { recursive: true });
writeFileSync(resolve(root, 'results/buffer-terminal/clear-reentry-source.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ cases: rows.length, scope: report.scope }));
