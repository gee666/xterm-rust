// Build the actual Rust/WASM distribution used by browser/demo.
import { existsSync } from 'node:fs';
import { mkdir, readFile, writeFile, stat, statfs, copyFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { resolve, dirname, relative } from 'node:path';
import { fileURLToPath } from 'node:url';
import { runCommand } from './quality-process.mjs';
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const runId = new Date().toISOString().replace(/[:.]/g, '-');
const directory = resolve(root, 'results/demos/build', runId);
const latest = resolve(root, 'results/demos/build/latest.json');
const output = resolve(root, 'browser/pkg');
await mkdir(directory, { recursive: true });
await mkdir(output, { recursive: true });
const portable = file => relative(root, file).replaceAll('\\', '/');
const commands = [];
const report = { status: 'incomplete', command: 'node scripts/build-demo.mjs', platform: process.platform,
  node: process.version, commands, output: portable(output), artifacts: [],
  scope: 'Rust WASM production library build only, not browser runtime equivalence or matched-feature benchmark proof.' };
await writeFile(latest, `${JSON.stringify({ status: 'incomplete', report: portable(resolve(directory, 'report.json')) }, null, 2)}\n`);
async function resources() {
  const disk = await statfs(root);
  const free = Number(disk.bavail) * Number(disk.bsize);
  if (free < 1024 ** 3) throw new Error(`Only ${free} bytes free; demo build requires1GiB reserve`);
}
async function run(name, command, args) {
  await resources();
  const log = resolve(directory, `${name}.log`);
  const result = await runCommand(command, args, { cwd: root, env: process.env, log,
    timeoutMs: 240000, resourceCheck: resources });
  commands.push({ ...result, command: [command, ...args], log: portable(log) });
  if (result.status !== 'passed') throw new Error(`${name} failed; inspect ${portable(log)}`);
  return readFile(log, 'utf8');
}
try {
  const cargo = process.env.CARGO || 'cargo';
  report.rustc = (await run('rustc-version', process.env.RUSTC || 'rustc', ['--version'])).trim();
  report.cargo = (await run('cargo-version', cargo, ['--version'])).trim();
  const bundledWindowsCli = resolve(root, 'tmp/wasm-tools/wasm-bindgen-0.2.100-x86_64-pc-windows-msvc/wasm-bindgen.exe');
  const bindgen = process.env.WASM_BINDGEN || (process.platform === 'win32' && existsSync(bundledWindowsCli) ? bundledWindowsCli : 'wasm-bindgen');
  report.bindgen = (await run('bindgen-version', bindgen, ['--version'])).trim();
  if (!/^wasm-bindgen 0\.2\.100(?:\s|$)/.test(report.bindgen)) throw new Error('The locked WASM ABI requires wasm-bindgen CLI0.2.100');
  const build = await run('release-wasm', cargo, ['build', '--locked', '-p', 'xterm-web', '--lib',
    '--target', 'wasm32-unknown-unknown', '--release', '--message-format=json']);
  const messages = build.split(/\r?\n/).filter(line => line.startsWith('{')).map(line => JSON.parse(line));
  const artifact = messages.find(message => message.reason === 'compiler-artifact'
    && message.target.name === 'xterm_web' && message.filenames.some(file => file.endsWith('.wasm')));
  const wasm = artifact?.filenames.find(file => file.endsWith('.wasm'));
  if (!wasm) throw new Error('Cargo did not produce the actual xterm-web WASM library');
  await run('bindgen', bindgen, [wasm, '--target', 'web', '--out-dir', output, '--out-name', 'xterm_web']);
  await copyFile(resolve(root, 'crates/xterm-web/assets/xterm.css'), resolve(root, 'browser/xterm.css'));
  const files = ['browser/pkg/xterm_web.js', 'browser/pkg/xterm_web_bg.wasm', 'browser/xterm.mjs',
    'browser/demo/index.html', 'browser/demo/app.mjs', 'browser/demo/controls.mjs',
    'browser/demo/addons.mjs', 'browser/demo/themes.mjs', 'browser/demo/demo.css', 'browser/xterm.css'];
  for (const file of files) {
    const bytes = await readFile(resolve(root, file));
    report.artifacts.push({ path: file, bytes: (await stat(resolve(root, file))).size,
      sha256: createHash('sha256').update(bytes).digest('hex') });
  }
  report.status = 'passed';
} catch (error) { report.status = 'failed'; report.error = String(error.stack); process.exitCode = 1; }
await writeFile(resolve(directory, 'report.json'), `${JSON.stringify(report, null, 2)}\n`);
await writeFile(latest, `${JSON.stringify({ status: report.status, report: portable(resolve(directory, 'report.json')) }, null, 2)}\n`);
console.log(JSON.stringify(report, null, 2));
