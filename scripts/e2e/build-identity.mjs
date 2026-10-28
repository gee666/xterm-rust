import { readdir, readFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { join } from 'node:path';

const execute = promisify(execFile);
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
export async function artifactIdentity(root, paths) {
  return Promise.all(paths.map(async path => {
    const bytes = await readFile(join(root, path));
    return { path, bytes: bytes.length, sha256: digest(bytes) };
  }));
}

// Hash on-disk build inputs, including uncommitted changes, rather than claiming
// that a commit ID alone identifies the source Cargo actually compiled.
export async function sourceIdentity(root) {
  const paths = ['Cargo.toml', 'Cargo.lock', '.cargo/config.toml'];
  async function walk(path) {
    for (const entry of await readdir(join(root, path), { withFileTypes: true })) {
      const child = `${path}/${entry.name}`;
      if (entry.isSymbolicLink()) throw new Error(`Linked build input is not supported: ${child}`);
      if (entry.isDirectory()) { if (entry.name !== 'target') await walk(child); }
      else if (entry.isFile()) paths.push(child);
    }
  }
  await walk('crates'); paths.sort();
  const files = await artifactIdentity(root, paths);
  const { stdout } = await execute('git', ['rev-parse', 'HEAD'], { cwd: root, timeout: 10000, windowsHide: true });
  return { revision: stdout.trim(), sha256: digest(JSON.stringify(files)), files: files.length };
}

export async function verifyBuiltPackage(root, source) {
  const latest = JSON.parse(await readFile(join(root, 'results/demos/build/latest.json'), 'utf8'));
  if (latest.status !== 'passed') throw new Error('No passing Rust build is available');
  if (typeof latest.report !== 'string' || !/^results\/demos\/build\/[a-zA-Z0-9-]+\/report\.json$/.test(latest.report)) {
    throw new Error('Invalid Rust build report path');
  }
  const build = JSON.parse(await readFile(join(root, latest.report), 'utf8'));
  if (build.status !== 'passed' || build.source?.sha256 !== source.sha256) throw new Error('Rust package is stale or has no matching source identity; rebuild it');
  const mandatory = ['browser/pkg/xterm_web.js', 'browser/pkg/xterm_web_bg.wasm'];
  const actual = await artifactIdentity(root, mandatory);
  for (const artifact of actual) {
    const recorded = build.artifacts?.find(item => item.path === artifact.path);
    if (recorded?.sha256 !== artifact.sha256) throw new Error(`Rust build artifact changed: ${artifact.path}`);
  }
  return { report: latest.report, source: build.source, artifacts: actual };
}
