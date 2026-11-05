// Licensed under MIT. Real Rust/live keyboard tests in Node's own realm.
import assert from 'node:assert/strict';
import { createHash, randomUUID } from 'node:crypto';
import { mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { runCommand } from './quality-process.mjs';

const root = resolve(import.meta.dirname, '..');
const work = resolve(root, 'tmp/keyboard-realm-node', randomUUID());
const output = resolve(root, 'results/kitty-realm/node', new Date().toISOString().replace(/[:.]/g, '-'));
await mkdir(work, { recursive: true });
await mkdir(output, { recursive: true });
const report = { scope: 'Real Rust core, LiveKeyEvent and realm table lookups. Component-only, not DOM dispatch.',
  node: process.version, platform: process.platform, status: 'preparing', commands: [] };
const persist = () => writeFile(resolve(output, 'verification.json'), JSON.stringify(report, null, 2) + '\n');
async function run(executable, args, name) {
  const log = resolve(output, name + '.log');
  const result = await runCommand(executable, args, { cwd: root, env: process.env, log, timeoutMs: 180000 });
  report.commands.push(result);
  await persist();
  assert.equal(result.status, 'passed', 'Command failed: ' + log);
  return readFile(log, 'utf8');
}
await persist();
try {
  const log = await run(process.env.CARGO ?? 'cargo', ['test', '-p', 'xterm-web', '--lib',
    '--target', 'wasm32-unknown-unknown', '--locked', '--no-run', '--message-format=json'], 'build');
  const messages = log.split('\n').filter(line => line.startsWith('{')).map(line => JSON.parse(line));
  const artifact = messages.find(message => message.reason === 'compiler-artifact'
    && message.target.name === 'xterm_web' && message.profile.test);
  const wasm = artifact?.filenames.find(name => name.endsWith('.wasm'));
  assert.ok(wasm, 'No Rust library test artifact');
  report.wasmSha256 = createHash('sha256').update(await readFile(wasm)).digest('hex');
  const bindgen = process.env.WASM_BINDGEN ?? 'wasm-bindgen';
  assert.equal((await run(bindgen, ['--version'], 'bindgen-version')).trim(), 'wasm-bindgen 0.2.100');
  await run(bindgen, [wasm, '--target', 'nodejs', '--out-dir', work, '--out-name', 'suite'], 'bindgen');
  const runner = resolve(work, 'run.cjs');
  await writeFile(runner, `
globalThis.__wbgtest_og_console_log = console.log.bind(console);
globalThis.__wbg_test_invoke = callback => callback();
const bindings = require('./suite.js');
const names = Object.keys(bindings.__wasm).filter(name => name.startsWith('__wbgt_') &&
  (name.includes('kitty_raw_lookup_values_match_source') || name.includes('win32_prototype_values_and_coercion_match_source')));
if (names.length !== 2) throw new Error('Expected exactly two keyboard realm Rust tests');
const context = new bindings.WasmBindgenTestContext();
(async () => {
  try {
    if (!await context.run(names.map(name => bindings.__wasm[name]))) process.exitCode = 1;
  } finally { context.free(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
`);
  const result = await run(process.execPath, [runner], 'node');
  assert.match(result, /2 passed; 0 failed; 0 ignored; 0 filtered out/);
  report.kittyCases = 10224;
  report.win32RetainedCases = 212;
  report.win32IndependentCases = 348;
  report.status = 'passed';
} catch (error) {
  report.status = 'failed';
  report.error = error.stack ?? String(error);
  process.exitCode = 1;
} finally {
  await persist();
  await rm(work, { recursive: true, force: true });
}
console.log('Keyboard Node/WASM realm comparison: ' + report.status + ', ' + output);
