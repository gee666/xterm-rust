#!/usr/bin/env node
// Real Windows filesystem boundary probe. No Rust code is compiled.
import { mkdir, mkdtemp, writeFile, readFile, rm, realpath } from 'node:fs/promises';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
import { relocateCache } from './target-relocation.mjs';

async function main() {
  if (process.platform !== 'win32') throw new Error('Windows-only Robocopy/junction probe');
  const project = fileURLToPath(new URL('../', import.meta.url));
  await mkdir(join(project, 'tmp'), { recursive: true });
  const parent = await mkdtemp(join(project, 'tmp', 'relocation-native-probe-'));
  const root = join(parent, 'project');
  const target = join(root, 'target');
  const destination = join(parent, 'cache');
  const output = join(project, 'results', 'quality', `target-relocation-native-probe-${Date.now()}-${process.pid}`);
  await mkdir(join(root, '.cargo'), { recursive: true });
  await mkdir(join(target, '.fingerprint', 'probe'), { recursive: true });
  await writeFile(join(root, '.cargo/config.toml'), await readFile(join(project, '.cargo/config.toml')));
  await writeFile(join(root, 'Cargo.toml'), '[workspace]\n[package]\nname="xterm_target_relocation_probe"\nversion="0.0.0"\nedition="2021"\n[lib]\npath="lib.rs"\n');
  await writeFile(join(root, 'Cargo.lock'), 'version = 4\n[[package]]\nname = "xterm_target_relocation_probe"\nversion = "0.0.0"\n');
  await writeFile(join(root, 'lib.rs'), '// Metadata-only probe; never compiled.\n');
  await writeFile(join(target, 'CACHEDIR.TAG'), 'Signature: 8a477f597d28d172789f06886806bc55\n');
  await writeFile(join(target, 'compiled.bin'), Buffer.from([0, 1, 127, 255]));
  await writeFile(join(target, '.fingerprint/probe/invoked.timestamp'), '');
  const probe = async (canonical, directory, stage) => {
    const args = ['metadata', '--no-deps', '--format-version', '1', '--locked', '--offline'];
    const result = spawnSync('cargo', args, { cwd: root, env: process.env, timeout: 30_000, maxBuffer: 2_000_000 });
    await writeFile(join(directory, `metadata-${stage}.log`), Buffer.concat([result.stdout ?? Buffer.alloc(0), result.stderr ?? Buffer.alloc(0)]));
    if (result.error || result.status !== 0) throw new Error('Tiny Cargo metadata probe failed');
    const metadata = JSON.parse(result.stdout.toString('utf8'));
    const actual = await realpath(metadata.target_directory);
    if (actual.toLowerCase() !== (await realpath(canonical)).toLowerCase()) throw new Error('Metadata target does not resolve through the canonical junction');
    return { command: ['cargo', ...args], exitCode: result.status, targetDirectory: metadata.target_directory, resolvedTarget: actual };
  };
  try {
    const result = await relocateCache({ root, destination, apply: true, approvedBy: 'tiny-native-boundary-probe', directory: output,
      reserveBytes: 0n, probe });
    await writeFile(join(output, 'scope.json'), JSON.stringify({ scope: 'Real tiny Windows Robocopy, nanosecond metadata, junction, rename rollback design and Cargo metadata; no Rust app compiled', node: process.version, platform: process.platform }, null, 2) + '\n');
    console.log(JSON.stringify(result, null, 2));
    if (result.status !== 'completed') process.exitCode = 1;
  } finally { await rm(parent, { recursive: true, force: true }); }
}
await main().catch(error => { console.error(error.message ?? String(error)); process.exitCode = 1; });
