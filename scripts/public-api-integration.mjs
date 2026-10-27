// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
// Event-time observations deliberately distinguish parser-time delivery from a
// queue drained after parsing. The production source and assertions are untouched.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
process.chdir(resolve(import.meta.dirname, '..'));
const require = createRequire(resolve('source_repo/package.json'));
const { build } = require('esbuild');
const directory = 'results/public-api';
mkdirSync(directory, { recursive: true });
mkdirSync('tmp/public-api', { recursive: true });
const attempt = `integration-${Date.now()}`;
const summaryPath = `${directory}/integration-summary.json`;
writeFileSync(summaryPath, JSON.stringify({status:'running', attempt}) + '\n');
const bundled = await build({
  absWorkingDir: process.cwd(), entryPoints: ['source_repo/src/headless/public/Terminal.ts'],
  outfile: 'tmp/public-api/headless.cjs', bundle: true, platform: 'node', format: 'cjs',
  target: 'node24', tsconfig: 'source_repo/src/headless/tsconfig.json', metafile: true, logLevel: 'silent'
});
const sourceHashes = Object.fromEntries(Object.keys(bundled.metafile.inputs).sort().map(path =>
  [path, createHash('sha256').update(readFileSync(path)).digest('hex')]));
const { Terminal } = require(resolve('tmp/public-api/headless.cjs'));
function hex(value) { return Array.from({length:value.length}, (_,i) => value.charCodeAt(i).toString(16)).join(','); }
function snapshot(terminal) {
  const buffer = terminal.buffer.active;
  return `${buffer.cursorX},${buffer.cursorY}|${hex(buffer.getLine(0).translateToString(true))}`;
}
const cases = [
  ['bell_observe', 'onBell', 'A\x07B', false],
  ['bell_reset', 'onBell', 'A\x07B', true],
  ['data_observe', 'onData', 'A\x1b[5nB', false],
  ['linefeed_observe', 'onLineFeed', 'A\nB', false],
  ['title_observe', 'onTitleChange', 'A\x1b]2;title\x07B', false]
];
const rows = [];
for (const [name, event, input, reset] of cases) {
  const terminal = new Terminal({cols:5, rows:3});
  const observations = [];
  terminal[event](() => { observations.push(snapshot(terminal)); if (reset) terminal.reset(); });
  await new Promise(done => terminal.write(input, done));
  rows.push(`${name}\t${hex(input)}\t${observations.join('/')}\t${snapshot(terminal)}`);
  terminal.dispose();
}
const fixture = 'crates/xterm-core/tests/fixtures/public-api-integration.tsv';
const text = rows.join('\n') + '\n';
if (process.argv.includes('--generate')) writeFileSync(fixture, text);
else assert.equal(readFileSync(fixture, 'utf8'), text, 'event-time source fixture drift');
const command = 'cargo test -p xterm-core --test public_api_integration';
const cargo = process.env.CARGO ?? (process.platform === 'win32' ? `${process.env.USERPROFILE}/.cargo/bin/cargo.exe` : 'cargo');
const run = spawnSync(cargo, ['test','-p','xterm-core','--test','public_api_integration'], {encoding:'utf8', timeout:180000, maxBuffer:8*1024*1024});
for (const name of ['integration-cargo', attempt]) {
  writeFileSync(`${directory}/${name}.stdout.log`,run.stdout ?? '');
  writeFileSync(`${directory}/${name}.stderr.log`,run.stderr ?? '');
}
const report = {status:run.status === 0 ? 'passed' : 'failed', attempt, command, sourceHashes,
  cases:cases.length, observations:rows, node:process.version, platform:`${process.platform}-${process.arch}`,
  exitCode:run.status, error:run.error?.message ?? null,
  scope:'PA04 event-time state and callback-triggered reset through actual source/Rust terminal'};
for (const [path, hash] of Object.entries(sourceHashes)) assert.equal(createHash('sha256').update(readFileSync(path)).digest('hex'), hash);
for (const path of [summaryPath, `${directory}/${attempt}.json`]) writeFileSync(path, JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(report,null,2));
process.exitCode=run.status===0 ? 0 : 1;
