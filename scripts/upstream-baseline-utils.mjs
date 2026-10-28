import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';

export const sha256 = bytes => crypto.createHash('sha256').update(bytes).digest('hex');

const projectRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const lockDirectory = path.join(projectRoot, 'tmp/evidence-pointer-locks');
function pointerLock(directory) {
  const canonical = fs.realpathSync(directory);
  return path.join(lockDirectory, `${sha256(process.platform === 'win32' ? canonical.toLowerCase() : canonical)}.lock`);
}
function locked(directory, callback) {
  fs.mkdirSync(lockDirectory, { recursive: true });
  const lock = pointerLock(directory);
  let fd;
  try { fd = fs.openSync(lock, 'wx'); }
  catch (error) {
    if (error.code === 'EEXIST') throw new Error(`Evidence pointer is locked: ${lock}. No retry or stale-owner deletion was attempted.`);
    throw error;
  }
  try {
    fs.writeFileSync(fd, JSON.stringify({ pid: process.pid, directory, created: new Date().toISOString() }));
    return callback();
  } finally {
    fs.closeSync(fd);
    fs.unlinkSync(lock);
  }
}
function writePointer(directory, pointer) {
  const temporary = path.join(lockDirectory, `${crypto.randomUUID()}.json`);
  fs.writeFileSync(temporary, JSON.stringify(pointer, null, 2) + '\n', { flag: 'wx' });
  try { fs.renameSync(temporary, path.join(directory, 'latest.json')); }
  finally { if (fs.existsSync(temporary)) fs.unlinkSync(temporary); }
}

/** Allocate an attempt before setup. An interrupted attempt cannot reuse success. */
export function createEvidenceRun(directory, metadata = {}) {
  const id = `${new Date().toISOString().replaceAll(/[:.]/g, '-')}-${crypto.randomUUID()}`;
  const attempt = { id, directory: path.join(directory, 'runs', id), root: directory };
  fs.mkdirSync(directory, { recursive: true });
  locked(directory, () => {
    fs.mkdirSync(attempt.directory, { recursive: true });
    fs.writeFileSync(path.join(attempt.directory, 'summary.json'), JSON.stringify({ ...metadata, status: 'incomplete', runId: id }, null, 2) + '\n');
    writePointer(directory, { runId: id, status: 'incomplete', summary: `runs/${id}/summary.json` });
  });
  return attempt;
}

export function persistEvidence(attempt, report) {
  locked(attempt.root, () => {
    const summary = path.join(attempt.directory, 'summary.json');
    fs.writeFileSync(summary, JSON.stringify({ ...report, runId: attempt.id }, null, 2) + '\n');
    const latest = JSON.parse(fs.readFileSync(path.join(attempt.root, 'latest.json'), 'utf8'));
    // A later-started attempt remains latest even if an older run finishes last.
    if (latest.runId === attempt.id) writePointer(attempt.root, {
      runId: attempt.id, status: report.status,
      summary: path.relative(attempt.root, summary).split(path.sep).join('/')
    });
  });
}

export function referenceIntegrity(directory, manifest) {
  const mismatches = [];
  for (const entry of manifest) {
    if (path.isAbsolute(entry.path) || entry.path.split(/[\\/]/).includes('..')) {
      throw new Error(`Reference manifest path escapes its root: ${entry.path}`);
    }
    const file = path.join(directory, entry.path);
    if (!fs.existsSync(file)) {
      mismatches.push({ path: entry.path, error: 'missing' });
      continue;
    }
    const bytes = fs.readFileSync(file);
    const hash = sha256(bytes);
    if (bytes.length !== entry.bytes || hash !== entry.sha256) {
      mismatches.push({ path: entry.path, bytes: bytes.length, sha256: hash });
    }
  }
  return { files: manifest.length, mismatches };
}

export function unitCounts(tests, passes, failures, pending) {
  if (tests !== passes + failures + pending) throw new Error('Mocha case counts disagree');
  return {
    discovered: tests, executed: passes + failures, passed: passes, failed: failures,
    pending, skipped: 0, unrun: 0
  };
}
