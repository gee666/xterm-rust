// Build and measure real release artifacts. No terminal behavior is delegated.
import { createHash } from 'node:crypto';
import { freemem } from 'node:os';
import { existsSync, lstatSync, readdirSync, readFileSync, mkdirSync, copyFileSync, writeFileSync, rmSync, statfsSync } from 'node:fs';
import { dirname, resolve, relative, isAbsolute, join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { inventory } from './distribution-size.mjs';
import { sourceIdentity } from './e2e/build-identity.mjs';
import { runCommand } from './quality-process.mjs';

const project = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const slash = value => value.replaceAll('\\', '/');
const within = (parent, child) => {
  const local = relative(parent, child);
  return local !== '' && !isAbsolute(local) && local !== '..' && !local.startsWith('..' + (process.platform === 'win32' ? '\\' : '/'));
};
// These are Cargo's ordinary release defaults, made explicit rather than tuned
// differently between the native and WASM builds to improve a comparison.
export const releaseProfile = Object.freeze({
  CARGO_BUILD_JOBS: '2', CARGO_PROFILE_RELEASE_OPT_LEVEL: '3', CARGO_PROFILE_RELEASE_LTO: 'false',
  CARGO_PROFILE_RELEASE_CODEGEN_UNITS: '16', CARGO_PROFILE_RELEASE_DEBUG: '0',
  CARGO_PROFILE_RELEASE_STRIP: 'none', CARGO_PROFILE_RELEASE_PANIC: 'unwind',
  CARGO_PROFILE_RELEASE_INCREMENTAL: 'false', CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS: 'false',
  CARGO_PROFILE_RELEASE_DEBUG_ASSERTIONS: 'false'
});
function regular(file) {
  const entry = lstatSync(file);
  if (!entry.isFile() || entry.isSymbolicLink()) throw new Error(`Expected regular artifact: ${file}`);
}
function regularAt(root, local) {
  let current = root;
  for (const part of local.split('/')) {
    current = join(current, part);
    if (lstatSync(current).isSymbolicLink()) throw new Error(`Linked artifact ancestor: ${local}`);
  }
  regular(current);
}
export function runtimeFiles(root) {
  const files = [], excluded = [];
  function walk(local) {
    const absolute = join(root, local), entry = lstatSync(absolute);
    if (entry.isSymbolicLink()) throw new Error(`Linked runtime entry: ${local}`);
    if (entry.isDirectory()) {
      for (const name of readdirSync(absolute).sort()) walk(`${local}/${name}`);
    } else if (entry.isFile()) {
      if (local.endsWith('.d.ts')) excluded.push({ path: local, reason: 'Type declarations are development metadata, not runtime downloads' });
      else if (/\.(?:js|wasm)$/.test(local)) files.push({ path: local, role: local.endsWith('.wasm') ? 'compiled WASM engine and addons' : 'generated binding or required snippet JS' });
      else throw new Error(`Unreviewed generated distribution asset: ${local}`);
    } else throw new Error(`Nonregular runtime entry: ${local}`);
  }
  walk('browser/pkg');
  for (const name of ['browser/pkg/xterm_web.js', 'browser/pkg/xterm_web_bg.wasm']) {
    if (!files.some(file => file.path === name)) throw new Error(`Missing mandatory artifact: ${name}`);
  }
  for (const [path, role] of [['browser/xterm.mjs', 'public bootstrap'], ['browser/xterm.css', 'required CSS'], ['LICENSE', 'port license']]) {
    regular(join(root, path)); files.push({ path, role });
  }
  // Ship the project's complete notice set. This is an explicit distribution
  // choice, not a claim that every optional font fixture is embedded in WASM.
  for (const name of readdirSync(join(root, 'licenses')).sort()) {
    if (/\.txt$/.test(name)) { regular(join(root, 'licenses', name)); files.push({ path: `licenses/${name}`, role: 'shipped third-party license or notice' }); }
  }
  return { files, excluded };
}
export function stageFiles(root, destination, files) {
  for (const file of files) {
    const input = resolve(root, file.path), output = resolve(destination, file.path);
    if (!within(root, input) || !within(destination, output)) throw new Error(`Escaped artifact path: ${file.path}`);
    regularAt(root, file.path); mkdirSync(dirname(output), { recursive: true }); copyFileSync(input, output);
  }
}
export function nativeArtifacts(root, output) {
  const artifacts = output.split(/\r?\n/).filter(line => line.startsWith('{')).map(line => JSON.parse(line))
    .filter(message => message.reason === 'compiler-artifact' && !message.profile.test);
  const executable = artifacts.filter(message => message.target.name === 'native' && message.target.kind.includes('example') && message.executable);
  const library = artifacts.filter(message => message.target.name === 'xterm_core' && message.target.kind.includes('lib'));
  if (executable.length !== 1 || library.length !== 1) throw new Error('Build did not uniquely identify native example and core library');
  for (const message of [...executable, ...library]) {
    if (message.profile.opt_level !== '3' || message.profile.debug_assertions !== false || message.profile.overflow_checks !== false) throw new Error('Native compiler artifact is not the declared release profile');
  }
  const rlibs = library[0].filenames.filter(file => file.endsWith('.rlib'));
  if (rlibs.length !== 1) throw new Error('Build did not uniquely identify the core rlib');
  return [executable[0].executable, rlibs[0]].map(file => {
    const absolute = resolve(file);
    if (!within(join(root, 'target/release'), absolute)) throw new Error(`Native artifact escaped private release target: ${file}`);
    regularAt(root, slash(relative(root, absolute)));
    return { path: slash(relative(root, absolute)), role: absolute.endsWith('.rlib') ? 'core rlib, not a standalone linked SDK or executable' : 'interactive native core example executable' };
  });
}
export async function main(args = process.argv.slice(2)) {
  const options = {};
  for (let index = 0; index < args.length; index += 2) {
    const key = args[index];
    if (!['--staging', '--output'].includes(key) || options[key] || !args[index + 1] || args[index + 1].startsWith('--')) throw new Error('Usage: --staging NEW-DIR --output DIR');
    options[key] = resolve(args[index + 1]);
  }
  if (Object.keys(options).length !== 2) throw new Error('Usage: --staging NEW-DIR --output DIR');
  const staging = options['--staging'], output = options['--output'];
  if (existsSync(staging) || staging === output || within(staging, output) || within(output, staging) || staging === project || within(staging, project) || (within(project, staging) && !within(join(project, 'tmp'), staging))) throw new Error('Staging must be a new owned scratch directory, disjoint from results and sources');
  if (!within(join(project, 'results'), output)) throw new Error('Durable output must be a child of this checkout results directory');
  mkdirSync(output, { recursive: true });
  for (const name of ['rust-manifest.json', 'rust-sizes.json']) rmSync(join(output, name), { force: true });
  const env = { ...process.env, ...releaseProfile };
  const report = { schemaVersion: 1, status: 'failed', node: process.version, platform: process.platform, architecture: process.arch,
    profile: releaseProfile, commands: [], scope: 'Actual optimized artifacts only. Not final parity, whole-workspace quality or native Linux/macOS runtime evidence.' };
  let ownStaging = false;
  function resources() {
    const disk = statfsSync(project), free = Number(disk.bavail) * Number(disk.bsize);
    if (free < 10 * 1024 ** 3 || freemem() < 8 * 1024 ** 3) throw new Error('Release measurement requires at least10GiB disk and8GiB available RAM');
  }
  async function run(id, program, args, timeoutMs = 600000) {
    resources();
    const log = join(output, `${id}.log`);
    const result = await runCommand(program, args, { cwd: project, env, log, timeoutMs, resourceCheck: resources });
    report.commands.push({ id, program, args, ...result, log: slash(relative(project, log)) });
    if (result.status !== 'passed') throw new Error(`${id} failed; inspect ${log}`);
    return readFileSync(log, 'utf8');
  }
  try {
    for (const key of Object.keys(process.env)) {
      if (/RUSTFLAGS|^CARGO_PROFILE_RELEASE_|^CARGO_TARGET_DIR$|^CARGO_BUILD_TARGET$/.test(key) && process.env[key]) throw new Error(`Remove build override before measuring: ${key}`);
    }
    const pkg = join(project, 'browser/pkg');
    if (existsSync(pkg)) throw new Error('Use a fresh owned checkout without browser/pkg; prior generated snippets must not inflate a new distribution');
    report.source = await sourceIdentity(project);
    report.rustc = (await run('rustc', process.env.RUSTC ?? 'rustc', ['-vV'], 30000)).trim();
    report.cargo = (await run('cargo', process.env.CARGO ?? 'cargo', ['--version'], 30000)).trim();
    await run('wasm-build', process.execPath, ['scripts/build-demo.mjs']);
    const pointer = JSON.parse(readFileSync(join(project, 'results/demos/build/latest.json')));
    if (pointer.status !== 'passed' || !/^results\/demos\/build\/[a-zA-Z0-9-]+\/report\.json$/.test(pointer.report)) throw new Error('No valid successful WASM build receipt');
    const build = JSON.parse(readFileSync(join(project, pointer.report)));
    if (build.profile !== 'release' || build.status !== 'passed' || build.source.sha256 !== report.source.sha256) throw new Error('WASM receipt is not this release source');
    report.wasmBuild = { report: pointer.report, sha256: hash(readFileSync(join(project, pointer.report))), bindgen: build.bindgen };
    const nativeLog = await run('native-build', process.env.CARGO ?? 'cargo', ['build', '--locked', '--release', '-p', 'xterm-core', '--lib', '--example', 'native', '--message-format=json', '-j2']);
    const native = nativeArtifacts(project, nativeLog), web = runtimeFiles(project);
    for (const artifact of build.artifacts) {
      if (hash(readFileSync(join(project, artifact.path))) !== artifact.sha256) throw new Error(`WASM build artifact changed: ${artifact.path}`);
    }
    const executable = resolve(project, native.find(file => !file.path.endsWith('.rlib')).path);
    const sample = 'crates/xterm-core/tests/fixtures/native-demo.txt';
    const expected = readFileSync(join(project, sample), 'utf8').split(/\r?\n/).filter(line => line.startsWith('assert-')).length;
    const checked = await run('native-script', executable, ['--plain', '--script', sample], 30000);
    if (expected < 20 || !checked.includes(`Assertions: ${expected}; disposed: true`)) throw new Error('Release example did not complete its source-derived interactive assertions');
    report.nativeAssertions = expected;
    if ((await sourceIdentity(project)).sha256 !== report.source.sha256) throw new Error('Build inputs changed during release measurement');
    mkdirSync(staging); ownStaging = true;
    const runtime = join(staging, 'rust-web');
    stageFiles(project, runtime, web.files);
    const metadata = { commands: report.commands.filter(row => row.id.endsWith('build')).map(row => [row.program, ...row.args].join(' ')),
      sourceSha256: report.source.sha256, toolchain: `${report.rustc}; ${report.cargo}; ${build.bindgen}`,
      flags: JSON.stringify(releaseProfile) + '; locked default features, acceptance-hooks excluded; wasm-bindgen --target web; no wasm-opt or post-link stripping' };
    const distributions = [{ id: 'rust-web-monolithic', directory: 'rust-web', kind: 'web', profile: 'release',
      scope: 'Default monolithic browser engine with all14 addon bindings; not a core-only web build and not proven fully equivalent to those addons.',
      addons: ['attach', 'clipboard', 'fit', 'image', 'ligatures', 'program-status', 'progress', 'search', 'serialize', 'unicode-graphemes', 'unicode11', 'web-fonts', 'web-links', 'webgl'],
      files: web.files, build: metadata }];
    for (const artifact of native) {
      const id = artifact.path.endsWith('.rlib') ? 'rust-native-core-rlib' : 'rust-native-core-example';
      const files = [artifact, ...web.files.filter(file => file.role.includes('license') || file.role.includes('notice'))];
      stageFiles(project, join(staging, id), files);
      distributions.push({ id, directory: id, kind: 'native', profile: 'release', addons: [], files, build: metadata,
        scope: artifact.role + '; raw artifact size only, not directly comparable to a JavaScript download; platform runtime prerequisites not embedded' });
    }
    const manifest = { schemaVersion: 1, distributions };
    const sizes = inventory(manifest, staging);
    report.notShipped = web.excluded;
    report.status = 'passed';
    writeFileSync(join(output, 'rust-manifest.json'), JSON.stringify(manifest, null, 2) + '\n');
    writeFileSync(join(output, 'rust-sizes.json'), JSON.stringify(sizes, null, 2) + '\n');
  } catch (error) { report.error = String(error.stack); process.exitCode = 1; }
  finally {
    if (ownStaging) {
      try { rmSync(staging, { recursive: true }); }
      catch (error) { report.status = 'failed'; report.cleanupError = String(error.stack); process.exitCode = 1; }
    }
    report.stagingRemoved = ownStaging && !existsSync(staging);
    if (report.status !== 'passed') for (const name of ['rust-manifest.json', 'rust-sizes.json']) rmSync(join(output, name), { force: true });
    writeFileSync(join(output, 'rust-build.json'), JSON.stringify(report, null, 2) + '\n');
  }
  console.log(JSON.stringify(report, null, 2));
  return report;
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  try { await main(); } catch (error) { console.error(error.stack); process.exitCode = 1; }
}
