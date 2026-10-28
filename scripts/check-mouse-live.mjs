#!/usr/bin/env node
// Run through build-slot.mjs. Captures the actual component source snapshot,
// unchanged-source fixture and immutable three-browser result paths.
import { readFile, writeFile, mkdir, readdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { runCommand } from './quality-process.mjs';
const root = fileURLToPath(new URL('../', import.meta.url));
const result = path.join(root, 'results/mouse-live', new Date().toISOString().replace(/[:.]/g, '-'));
await mkdir(result, { recursive: true });
const fixed = ['Cargo.toml', 'Cargo.lock', '.cargo/config.toml', 'crates/xterm-core/Cargo.toml',
  'crates/xterm-web/Cargo.toml', 'crates/xterm-addons/Cargo.toml',
  'crates/xterm-web/src/dom_listener.rs', 'crates/xterm-web/src/js_error.rs',
  'crates/xterm-web/src/input/mouse_coords.rs', 'crates/xterm-web/src/input/mouse_policy.rs', 'crates/xterm-web/src/input/mouse_wheel.rs',
  'crates/xterm-web/tests/wasm_mouse.rs', 'crates/xterm-web/tests/support/mouse_dom.rs',
  'crates/xterm-web/tests/support/input_dom.rs', 'crates/xterm-web/tests/support/input_mouse_host.rs',
  'crates/xterm-web/tests/fixtures/mouse-live.tsv', 'scripts/mouse-live-oracle.mjs', 'scripts/check-mouse-live.mjs'];
const groups = [
  ['crates/xterm-web/src/input/dom', /^mouse.*\.rs$/], ['crates/xterm-web/tests/cases', /^mouse.*\.rs$/],
  ['crates/xterm-core/src', /^(mouse.*|event.*|lifecycle.*)\.rs$/],
];
async function snapshot() {
  const files = [...fixed];
  for (const [directory, pattern] of groups) {
    for (const name of await readdir(path.join(root, directory))) if (pattern.test(name)) files.push(`${directory}/${name}`);
  }
  return Object.fromEntries(await Promise.all([...new Set(files)].sort().map(async file => [file,
    createHash('sha256').update(await readFile(path.join(root, file))).digest('hex')])));
}
async function directories() {
  try { return await readdir(path.join(root, 'results/browser-host/wasm_mouse')); }
  catch (error) { if (error.code === 'ENOENT') return []; throw error; }
}
const commands = [];
const report = { command: 'node scripts/check-mouse-live.mjs through wasm build lease', platform: process.platform,
  node: process.version, startedAt: new Date().toISOString(), status: 'in_progress', componentOnly: true,
  originalSuiteExecuted: 0, commands, browsers: [], changedFiles: [] };
try {
  report.before = await snapshot();
  const beforeDirectories = new Set(await directories());
  for (const [name, script, environment] of [
    ['source', 'scripts/mouse-live-oracle.mjs', process.env],
    ['browser', 'scripts/browser-host-check.mjs', { ...process.env, WASM_TEST: 'wasm_mouse', BROWSERS: 'chromium,firefox,webkit' }],
  ]) {
    const command = await runCommand(process.execPath, [script], { cwd: root, env: environment,
      log: path.join(result, `${name}.log`), timeoutMs: name === 'source' ? 30000 : 240000 });
    commands.push(command);
    if (command.status !== 'passed') throw new Error(`${name} stage failed`);
  }
  const created = (await directories()).filter(name => !beforeDirectories.has(name));
  if (created.length !== 1) throw new Error(`Expected one owned browser attempt, observed ${created.length}`);
  report.browserReport = `results/browser-host/wasm_mouse/${created[0]}/wasm_mouse-results.json`;
  const browser = JSON.parse(await readFile(path.join(root, report.browserReport), 'utf8'));
  report.browsers = browser.browsers.map(entry => ({ name: entry.name, version: entry.version,
    status: entry.status, counts: entry.counts, tests: entry.tests.length }));
  if (report.browsers.length !== 3 || report.browsers.some(entry => entry.status !== 'passed')) throw new Error('Three-browser assertions did not all pass');
  report.artifacts = browser.artifacts;
  report.status = 'passed';
} catch (error) {
  report.status = 'failed'; report.error = error.message;
} finally {
  report.after = await snapshot();
  report.changedFiles = [...new Set([...Object.keys(report.before ?? {}), ...Object.keys(report.after)])]
    .filter(file => report.before?.[file] !== report.after[file]);
  if (report.changedFiles.length && report.status === 'passed') report.status = 'source_changed';
  report.finishedAt = new Date().toISOString();
  const json = JSON.stringify(report, null, 2) + '\n';
  await writeFile(path.join(result, 'result.json'), json);
  await writeFile(path.join(root, 'results/mouse-live/latest.json'), json);
}
console.log(JSON.stringify({ status: report.status, report: path.relative(root, path.join(result, 'result.json')) }));
if (report.status !== 'passed') process.exitCode = 1;
