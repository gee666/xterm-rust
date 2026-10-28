// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Index retained evidence without interpreting a log's existence as success.
import { createHash } from 'node:crypto';
import { readdir, readFile, mkdir, writeFile, lstat } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
export const digest = bytes => createHash('sha256').update(bytes).digest('hex');

export async function inventory(directory, prefix = '') {
  if ((await lstat(directory)).isSymbolicLink()) throw new Error('Evidence root symlink requires explicit policy');
  const records = [];
  for (const entry of (await readdir(directory, { withFileTypes: true })).sort((a, b) => a.name < b.name ? -1 : a.name > b.name ? 1 : 0)) {
    const relative = prefix ? `${prefix}/${entry.name}` : entry.name;
    const absolute = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      records.push(...await inventory(absolute, relative));
    } else if (entry.isFile()) {
      const bytes = await readFile(absolute);
      records.push({ path: relative, bytes: bytes.length, sha256: digest(bytes) });
    } else {
      throw new Error(`Evidence must be a regular file or directory: ${relative}`);
    }
  }
  return records;
}

export async function verifyReference(directory, manifest) {
  const changed = [];
  const missing = [];
  const seen = new Set();
  for (const record of manifest) {
    // A retained manifest is data, never authority to read outside source_repo.
    if (typeof record.path !== 'string' || record.path.includes('\\') || record.path.includes(':') || path.posix.isAbsolute(record.path)
      || record.path.split('/').some(part => part === '..' || part === '.' || part === '')
      || !/^[a-f0-9]{64}$/.test(record.sha256) || !Number.isSafeInteger(record.bytes) || record.bytes < 0
      || seen.has(record.path)) {
      throw new Error(`Invalid or duplicate source manifest record: ${JSON.stringify(record)}`);
    }
    seen.add(record.path);
    let bytes;
    try {
      // Lexical traversal checks are not enough: a valid component may be a
      // junction to another tree. Check each component without following links.
      let component = directory;
      for (const part of ['', ...record.path.split('/')]) {
        if (part) component = path.join(component, part);
        if ((await lstat(component)).isSymbolicLink()) {
          throw new Error(`Reference symlink requires explicit policy: ${record.path}`);
        }
      }
      bytes = await readFile(path.join(directory, record.path));
    } catch (error) {
      if (error.code !== 'ENOENT') throw error;
      missing.push(record.path);
      continue;
    }
    const actual = { bytes: bytes.length, sha256: digest(bytes) };
    if (actual.bytes !== record.bytes || actual.sha256 !== record.sha256) {
      changed.push({ path: record.path, expected: { bytes: record.bytes, sha256: record.sha256 }, actual });
    }
  }
  return { listedFiles: manifest.length, checkedFiles: manifest.length - missing.length, changed, missing, matched: changed.length === 0 && missing.length === 0 };
}

export async function buildIndex(projectRoot) {
  const manifestPath = 'docs/baseline/source-manifest.json';
  const manifestBytes = await readFile(path.join(projectRoot, manifestPath));
  const manifest = JSON.parse(manifestBytes);
  if (!Array.isArray(manifest) || manifest.length === 0) throw new Error('Source manifest must be a nonempty array');
  const integrity = await verifyReference(path.join(projectRoot, 'source_repo'), manifest);
  const evidence = (await inventory(path.join(projectRoot, 'results')))
    .filter(record => record.path !== 'verification/index.json')
    .map(record => ({ ...record, path: `results/${record.path}` }));
  return {
    scope: 'Evidence file inventory and listed reference integrity only; full terminal equivalence NOT PROVEN',
    command: 'node scripts/verification-index.mjs',
    generatedAt: new Date().toISOString(),
    environment: { node: process.version, platform: process.platform, architecture: process.arch },
    reference: { provenance: 'User-supplied source_repo tree; no public revision is asserted', manifestPath, manifestSha256: digest(manifestBytes), ...integrity },
    evidence,
    interpretation: 'Hashes identify retained files, not passed checks. Consult each result and docs/verification.md. Newly added source files are not checked by the retained starting manifest.'
  };
}

export async function checkIndex(projectRoot) {
  const saved = JSON.parse(await readFile(path.join(projectRoot, 'results/verification/index.json'), 'utf8'));
  if (!Array.isArray(saved.evidence) || !saved.reference) throw new Error('Invalid saved evidence index');
  for (const record of saved.evidence) {
    if (typeof record.path !== 'string' || !record.path.startsWith('results/') || record.path === 'results/verification/index.json') {
      throw new Error('Saved evidence paths must stay under results and exclude the index');
    }
  }
  const current = await buildIndex(projectRoot);
  const files = await verifyReference(projectRoot, saved.evidence);
  const previous = new Set(saved.evidence.map(record => record.path));
  const added = current.evidence.filter(record => !previous.has(record.path)).map(record => record.path);
  const manifestMatched = saved.reference.manifestSha256 === current.reference.manifestSha256;
  return {
    scope: 'Snapshot freshness only, not passing test or equivalence certification',
    command: 'node scripts/verification-index.mjs --check',
    snapshotGeneratedAt: saved.generatedAt ?? null,
    ...files, added, manifestMatched, reference: current.reference,
    matched: files.matched && added.length === 0 && manifestMatched && current.reference.matched
  };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  if (args.length > 1 || (args.length === 1 && args[0] !== '--check')) throw new Error('Usage: node scripts/verification-index.mjs [--check]');
  if (args[0] === '--check') {
    const report = await checkIndex(root);
    console.log(JSON.stringify(report, null, 2));
    if (!report.matched) process.exitCode = 1;
  } else {
    const index = await buildIndex(root);
    await mkdir(path.join(root, 'results/verification'), { recursive: true });
    await writeFile(path.join(root, 'results/verification/index.json'), `${JSON.stringify(index, null, 2)}\n`);
    console.log(JSON.stringify({ evidenceFiles: index.evidence.length, reference: index.reference }, null, 2));
    if (!index.reference.matched) process.exitCode = 1;
  }
}
