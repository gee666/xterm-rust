// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { stripTypeScriptTypes } from 'node:module';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { referenceIntegrity } from './upstream-baseline-utils.mjs';
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const source = resolve(process.env.SOURCE_ROOT ?? join(root, 'source_repo'));
const fixture = join(root, 'crates/xterm-web/tests/fixtures/attach-removal');
const output = join(root, 'results/attach-removal');
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const report = { status: 'failed', node: process.version, platform: process.platform,
  scope: 'Shared unchanged AttachAddon source methods; 36 removal traces, 42 raw errors, retained registration, literal argument calls and 12 non-callable methods, not the full upstream suite.' };
await mkdir(output, { recursive: true });
try {
  const manifest = JSON.parse(await readFile(join(root, 'docs/baseline/reference-manifest.json'), 'utf8'));
  report.before = referenceIntegrity(source, manifest);
  assert.equal(report.before.mismatches.length, 0);
  const original = await readFile(join(source, 'addons/addon-attach/src/AttachAddon.ts'), 'utf8');
  const { AttachAddon } = await import(`data:text/javascript;base64,${Buffer.from(stripTypeScriptTypes(original)).toString('base64')}`);
  const shared = await readFile(fixture + '.js', 'utf8');
  const run = Function(`${shared};return async create => ({rows:await attachRemovalCases(create),registration:await attachRetainedRegistration(create),rawErrors:await attachRawRemovalErrors(create),literalArguments:await attachLiteralArguments(create),nonCallable:await attachNonCallableMethods(create)});`)();
  const actual = await run((socket, bidirectional) => {
    const writes = [], data = [], binary = [];
    const subscribe = entries => callback => {
      entries.push(callback);
      return { dispose() { const index = entries.indexOf(callback); if (index !== -1) entries.splice(index, 1); } };
    };
    const terminal = { write(value) { writes.push(value); }, onData: subscribe(data), onBinary: subscribe(binary) };
    const addon = new AttachAddon(socket, { bidirectional });
    return { activate: () => addon.activate(terminal), dispose: () => addon.dispose(),
      input: value => { for (const callback of data.slice()) callback(value); },
      flush: async () => writes.join('') };
  });
  assert.equal(actual.rows.length, 36);
  assert.equal(actual.rawErrors, 42);
  assert.equal(actual.literalArguments, 3);
  assert.equal(actual.nonCallable, 12);
  for (const row of actual.rows) {
    assert.deepEqual(row.activeAfterSuccess, [], row.name);
    assert.deepEqual(row.trace.slice(-3), ['replacement:message', 'replacement:close', 'replacement:error']);
    assert.equal(row.text, 'A');
    assert.equal(row.repeatedText, 'AB');
  }
  if (process.argv.includes('--generate')) await writeFile(fixture + '.json', JSON.stringify(actual, null, 2) + '\n');
  else assert.deepEqual(actual, JSON.parse(await readFile(fixture + '.json', 'utf8')));
  report.counts = { discovered: 92, executed: 92, passed: 92, failed: 0, skipped: 0, unrun: 0 };
  report.sourceSha256 = hash(original);
  report.fixtureSha256 = hash(await readFile(fixture + '.json'));
  report.sharedSha256 = hash(shared);
  report.after = referenceIntegrity(source, manifest);
  assert.equal(report.after.mismatches.length, 0);
  report.status = 'passed';
} catch (error) { report.error = String(error.stack ?? error); process.exitCode = 1; }
finally { await writeFile(join(output, 'source.json'), JSON.stringify(report, null, 2) + '\n'); }
console.log(`Attach removal source ${report.status}, ${report.counts?.executed ?? 0} scenarios`);
