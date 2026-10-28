// Executes actual oracle startup bodies in isolated roots with stale-pass sentinels.
// Only root/helper import paths change. No production source or shared report moves.
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir, rm } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { pathToFileURL, fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import path from 'node:path';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const definitions = [
  { id: 'reflow', source: 'scripts/buffer-reflow-oracle.mjs', output: 'buffer-reflow', temp: 'buffer-reflow-oracle',
    receipt: 'report.json', expected: 'incomplete', rootLine: "const root = resolve(import.meta.dirname, '..');" },
  { id: 'serialize', source: 'scripts/serialize-oracle.mjs', output: 'serialize', temp: 'serialize-oracle',
    receipt: 'last-attempt.json', expected: 'failed' },
  { id: 'charset', source: 'scripts/charset-terminal-oracle.mjs', output: 'charset-terminal', temp: 'charset-terminal',
    receipt: 'last-attempt.json', expected: 'failed' }
];
const selected = process.argv.slice(2).map(value => value.replace(/^--/, ''));
assert.ok(selected.every(id => definitions.some(definition => definition.id === id)), 'Unknown oracle selector');
const records = [];
for (const definition of definitions.filter(definition => !selected.length || selected.includes(definition.id))) {
  const source = await readFile(path.join(root, definition.source), 'utf8');
  const sourceSha256 = createHash('sha256').update(source).digest('hex');
  const rootLine = definition.rootLine || "const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');";
  assert.equal(source.split(rootLine).length, 2, 'Expected exactly one root assignment');
  for (const scenario of ['missing-reference-or-dependency', 'blocked-temp-directory']) {
    const fakeRoot = path.join(root, 'tmp/oracle-startup', definition.id, scenario);
    await rm(fakeRoot, { recursive: true, force: true });
    const out = path.join(fakeRoot, 'results', definition.output);
    await mkdir(out, { recursive: true });
    await mkdir(path.join(fakeRoot, 'tmp'), { recursive: true });
    // A broken package main prevents fallback to a globally installed esbuild.
    // These tests must fail before starting any compiler or bundler service.
    const packageDir = path.join(fakeRoot, 'source_repo/node_modules/esbuild');
    await mkdir(packageDir, { recursive: true });
    await writeFile(path.join(packageDir, 'package.json'), JSON.stringify({ main: 'missing-entry.js' }));
    await writeFile(path.join(out, definition.receipt), JSON.stringify({ status: 'passed', sentinel: 'stale previous run' }));
    if (scenario === 'blocked-temp-directory') {
      await writeFile(path.join(fakeRoot, 'tmp', definition.temp), 'regular file, not a directory');
    }
    const probe = source.replace(rootLine, `const root = ${JSON.stringify(fakeRoot)};`)
      .replace(/(from\s+)(['"])(\.\/[^'"]+\.mjs)\2/g, (_, prefix, quote, module) =>
        prefix + JSON.stringify(pathToFileURL(path.join(root, 'scripts', module.slice(2))).href));
    const file = path.join(fakeRoot, 'probe.mjs');
    await writeFile(file, probe);
    const run = spawnSync(process.execPath, [file], { cwd: root, encoding: 'utf8', timeout: 10000, maxBuffer: 1024 ** 2 });
    const receipt = JSON.parse(await readFile(path.join(out, definition.receipt), 'utf8'));
    const passed = Number.isInteger(run.status) && run.status !== 0 && !run.signal && !run.error && receipt.status === definition.expected;
    records.push({ oracle: definition.id, source: definition.source, sourceSha256, scenario,
      status: run.status, signal: run.signal, error: run.error?.message, stderr: run.stderr,
      receipt, expectedReceiptStatus: definition.expected, passed });
  }
}
const report = { command: `node scripts/check-oracle-startup.mjs${selected.map(id => ` --${id}`).join('')}`,
  node: process.version, platform: process.platform, arch: process.arch,
  method: 'Actual source body with only root/import paths redirected into tmp. Nonzero exit and invalidated prior pass both required; timeout/crash is not success.',
  executed: records.length, passed: records.filter(record => record.passed).length,
  failed: records.filter(record => !record.passed).length, records };
await mkdir(path.join(root, 'results/oracle-startup'), { recursive: true });
const reportPath = path.join(root, 'results/oracle-startup', selected.length ? selected.join('-') + '.json' : 'all.json');
await writeFile(reportPath, JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ executed: report.executed, passed: report.passed, failed: report.failed }));
assert.equal(report.failed, 0, 'Oracle left stale passing receipt; inspect results/oracle-startup/');
