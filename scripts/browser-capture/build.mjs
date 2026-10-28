// Scoped debug acceptance build. Run inside the existing WASM build lease.
import { createRequire } from 'node:module';
import { existsSync } from 'node:fs';
import { mkdir, readFile, writeFile, statfs, readdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { resolve, dirname, relative, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { runCommand } from '../quality-process.mjs';
import { assertBuildEnvironment } from '../quality-policy.mjs';
const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
if (process.argv.length !== 4 || process.argv[2] !== '--output') throw new Error('Usage: build.mjs --output tmp/DIRECTORY');
const output = resolve(root, process.argv[3]);
if (!output.startsWith(resolve(root, 'tmp') + sep)) throw new Error('Acceptance output must be under repository tmp/');
await mkdir(output, { recursive: true });
const packageDir = resolve(output, 'pkg');
await mkdir(packageDir, { recursive: true });
const report = { status: 'incomplete', platform: process.platform, node: process.version, commands: [], sourceHashes: [], artifacts: [] };
const portable = value => relative(root, value).split(sep).join('/');
const sha256 = value => createHash('sha256').update(value).digest('hex');
async function resources() {
  const disk = await statfs(root);
  if (Number(disk.bavail) * Number(disk.bsize) < 1024 ** 3) throw new Error('Browser acceptance build requires1GiB headroom');
}
async function run(name, command, args) {
  const log = resolve(output, `${name}.log`);
  await resources();
  const result = await runCommand(command, args, { cwd: root, env: process.env, log,
    timeoutMs: 180000, resourceCheck: resources });
  report.commands.push({ ...result, argv: [command, ...args], log: portable(log) });
  if (result.status !== 'passed') throw new Error(`${name} failed; inspect ${portable(log)}`);
  return readFile(log, 'utf8');
}
try {
  assertBuildEnvironment(process.env);
  const cargo = process.env.CARGO || 'cargo';
  report.rustc = (await run('rust-version', process.env.RUSTC || 'rustc', ['--version'])).trim();
  const built = await run('cargo-build', cargo, ['build', '--locked', '-p', 'xterm-web', '--lib',
    '--target', 'wasm32-unknown-unknown', '--features', 'acceptance-hooks', '--message-format=json']);
  const messages = built.split(/\r?\n/).filter(line => line.startsWith('{')).map(line => JSON.parse(line));
  const artifact = messages.find(value => value.reason === 'compiler-artifact' && value.target.name === 'xterm_web'
    && value.filenames.some(file => file.endsWith('.wasm')));
  const wasm = artifact?.filenames.find(file => file.endsWith('.wasm'));
  if (!wasm) throw new Error('No actual Rust acceptance WASM artifact');
  const windowsCli = resolve(root, 'tmp/wasm-tools/wasm-bindgen-0.2.100-x86_64-pc-windows-msvc/wasm-bindgen.exe');
  const bindgen = process.env.WASM_BINDGEN || (process.platform === 'win32' && existsSync(windowsCli) ? windowsCli : 'wasm-bindgen');
  report.bindgen = (await run('bindgen-version', bindgen, ['--version'])).trim();
  if (!/^wasm-bindgen 0\.2\.100(?:\s|$)/.test(report.bindgen)) throw new Error('Locked browser ABI requires bindgen0.2.100');
  await run('bindgen', bindgen, [wasm, '--target', 'web', '--out-dir', packageDir, '--out-name', 'xterm_web']);
  const entry = resolve(output, 'reference-entry.ts');
  const modulePath = value => JSON.stringify(relative(output, resolve(root, value)).split(sep).join('/').replace(/^(?!\.)/, './'));
  await writeFile(entry, `export { Terminal } from ${modulePath('source_repo/src/browser/public/Terminal.ts')};\nexport { CHARSETS } from ${modulePath('source_repo/src/common/data/Charsets.ts')};\n`);
  const require = createRequire(resolve(root, 'source_repo/package.json'));
  const esbuild = require('esbuild');
  const result = await esbuild.build({ absWorkingDir: root, entryPoints: [entry], outfile: resolve(output, 'reference.js'),
    bundle: true, format: 'iife', globalName: 'XtermReference', platform: 'browser', target: 'es2022',
    tsconfig: resolve(root, 'source_repo/src/browser/tsconfig.json'), metafile: true, logLevel: 'silent' });
  report.esbuild = esbuild.version;
  report.sourceHashes = await Promise.all(Object.keys(result.metafile.inputs).sort().map(async file => ({
    path: file.split(sep).join('/'), sha256: sha256(await readFile(resolve(root, file)))
  })));
  async function artifacts(directory) {
    for (const entry of await readdir(directory, { withFileTypes: true })) {
      const file = resolve(directory, entry.name);
      if (entry.isSymbolicLink()) throw new Error('Generated bindings must not contain symlinks');
      if (entry.isDirectory()) await artifacts(file);
      else if (entry.isFile()) report.artifacts.push({ path: portable(file), sha256: sha256(await readFile(file)) });
      else throw new Error('Generated binding is not a regular file');
    }
  }
  await artifacts(packageDir);
  report.reference = { path: portable(resolve(output, 'reference.js')), sha256: sha256(await readFile(resolve(output, 'reference.js'))) };
  report.status = 'passed';
} catch (error) { report.status = 'failed'; report.error = String(error.stack); process.exitCode = 1; }
await writeFile(resolve(output, 'build.json'), `${JSON.stringify(report, null, 2)}\n`);
console.log(JSON.stringify(report, null, 2));
