// Run the unchanged upstream production ESM builds, then count their shipped files.
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, lstatSync, mkdirSync, readFileSync, writeFileSync, copyFileSync, symlinkSync, unlinkSync, rmSync } from 'node:fs';
import { dirname, resolve, join, relative, isAbsolute } from 'node:path';
import { fileURLToPath } from 'node:url';
import { inventory as sourceInventory } from './count-loc.mjs';
import { inventory as sizes } from './distribution-size.mjs';

const project = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const options = {};
for (let i = 2; i < process.argv.length; i += 2) {
  const key = process.argv[i];
  if (!['--source', '--dependencies', '--staging', '--output'].includes(key) || options[key] || !process.argv[i + 1] || process.argv[i + 1].startsWith('--')) throw new Error('Usage: --source DIR --dependencies DIR --staging NEW-DIR --output DIR');
  options[key] = resolve(process.argv[i + 1]);
}
if (Object.keys(options).length !== 4) throw new Error('Usage: --source DIR --dependencies DIR --staging NEW-DIR --output DIR');
const source = options['--source'], staging = options['--staging'], output = options['--output'];
for (const folder of [source, options['--dependencies']]) {
  const info = lstatSync(folder);
  if (!info.isDirectory() || info.isSymbolicLink()) throw new Error(`Expected real directory: ${folder}`);
}
const within = (parent, child) => { const path = relative(parent, child); return !isAbsolute(path) && path !== '..' && !path.startsWith('..' + (process.platform === 'win32' ? '\\' : '/')); };
if (within(staging, source) || within(source, staging) || within(staging, output) || within(source, output)) throw new Error('Source, staging and durable output must be disjoint');
if (existsSync(staging)) throw new Error('Staging must be a new owned directory');
const dependencyLink = join(source, 'node_modules');
if (existsSync(dependencyLink)) throw new Error('Use an isolated reference copy without node_modules; existing dependencies will not be replaced');
mkdirSync(output, { recursive: true });
// A failed rebuild must not leave an old successful size receipt beside it.
for (const name of ['original-manifest.json', 'original-sizes.json']) rmSync(join(output, name), { force: true });
const report = { node: process.version, platform: process.platform, commands: [], status: 'failed',
  scope: 'Original production ESM core only. No addons, Rust sizes, timing or functional equivalence claim.' };
let ownLink = false, ownStaging = false;
try {
  const rules = JSON.parse(readFileSync(join(project, 'scripts/loc-rules.json')));
  report.sourceSha256 = sourceInventory(source, 'source', rules).inventorySha256;
  report.esbuild = JSON.parse(readFileSync(join(options['--dependencies'], 'esbuild/package.json'))).version;
  report.lockSha256 = createHash('sha256').update(readFileSync(join(source, 'package-lock.json'))).digest('hex');
  symlinkSync(options['--dependencies'], dependencyLink, process.platform === 'win32' ? 'junction' : 'dir');
  ownLink = true;
  for (const args of [['bin/esbuild.mjs', '--prod'], ['bin/esbuild.mjs', '--prod', '--headless']]) {
    const result = spawnSync(process.execPath, args, { cwd: source, encoding: 'utf8', timeout: 60000, maxBuffer: 8 * 1024 ** 2 });
    report.commands.push({ command: ['node', ...args], cwd: 'source_repo', status: result.status, signal: result.signal,
      error: result.error?.message, stdout: result.stdout, stderr: result.stderr });
    if (result.error || result.status !== 0) throw new Error(`Original production build failed: ${args.join(' ')}`);
  }
  unlinkSync(dependencyLink);
  ownLink = false;
  if (sourceInventory(source, 'source', rules).inventorySha256 !== report.sourceSha256) throw new Error('Reference inventory changed during build');
  mkdirSync(staging);
  ownStaging = true;
  const distributions = [];
  for (const [id, kind, bundle, directory] of [
    ['original-web-core-esm', 'web', 'lib/xterm.mjs', 'web'],
    ['original-headless-core-esm', 'headless-js', 'headless/lib-headless/xterm-headless.mjs', 'headless']
  ]) {
    const destination = join(staging, directory);
    mkdirSync(destination);
    const name = bundle.split('/').at(-1);
    const files = [{ path: name, role: 'production ESM bundle' }, { path: 'LICENSE', role: 'license' }];
    copyFileSync(join(source, bundle), join(destination, name));
    copyFileSync(join(source, 'LICENSE'), join(destination, 'LICENSE'));
    if (kind === 'web') {
      copyFileSync(join(source, 'css/xterm.css'), join(destination, 'xterm.css'));
      files.push({ path: 'xterm.css', role: 'required CSS' });
    }
    distributions.push({ id, directory, kind, profile: 'release', scope: 'Core-only original ESM distribution, all fourteen optional addons excluded', addons: [], files,
      build: { commands: report.commands.map(row => row.command.join(' ')), sourceSha256: report.sourceSha256,
        toolchain: `Node ${process.version}; esbuild ${report.esbuild}; lock SHA256 ${report.lockSha256}`,
        flags: 'Unchanged bin/esbuild.mjs --prod: minify=true, treeShaking=true, target=es2021, legalComments=none; emitted sourcemaps excluded from download totals; no native LTO/strip flags apply' } });
  }
  const manifest = { schemaVersion: 1, distributions };
  const result = sizes(manifest, staging);
  writeFileSync(join(output, 'original-manifest.json'), JSON.stringify(manifest, null, 2) + '\n');
  writeFileSync(join(output, 'original-sizes.json'), JSON.stringify(result, null, 2) + '\n');
  report.status = 'passed';
} catch (error) { report.error = String(error.stack); process.exitCode = 1; }
finally {
  if (ownLink) unlinkSync(dependencyLink);
  if (ownStaging) rmSync(staging, { recursive: true });
  writeFileSync(join(output, 'original-build.json'), JSON.stringify(report, null, 2) + '\n');
}
console.log(JSON.stringify(report, null, 2));
