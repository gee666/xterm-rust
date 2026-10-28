import { readFile, writeFile, mkdir, lstat } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

export function validateRegistry(value) {
  if (!value || !Array.isArray(value.criteria) || value.criteria.length === 0) throw new Error('criteria must be a nonempty array');
  const ids = new Set();
  return value.criteria.map(criterion => {
    if (!Number.isSafeInteger(criterion.id) || criterion.id < 1 || ids.has(criterion.id)) {
      throw new Error(`invalid or duplicate criterion id: ${criterion.id}`);
    }
    ids.add(criterion.id);
    for (const key of ['group', 'title', 'body', 'verification', 'source']) {
      if (typeof criterion[key] !== 'string' || !criterion[key].trim()) {
        throw new Error(`criterion ${criterion.id}: missing ${key}`);
      }
    }
    if (!Array.isArray(criterion.taskIds) || criterion.taskIds.some(id => !Number.isSafeInteger(id) || id < 1)) {
      throw new Error(`criterion ${criterion.id}: invalid taskIds`);
    }
    const text = [criterion.source, criterion.body, criterion.verification].join('\n');
    const documents = [...new Set(text.match(/docs\/[A-Za-z0-9_./-]+\.md/g) ?? [])].sort();
    const sourceFiles = [...new Set(criterion.source.match(/source_repo\/[A-Za-z0-9_./-]+\.(?:tsx?|css|json)/g) ?? [])].sort();
    for (const relative of [...documents, ...sourceFiles]) {
      if (relative.split('/').includes('..') || path.posix.isAbsolute(relative)) {
        throw new Error(`criterion ${criterion.id}: unsafe reference ${relative}`);
      }
    }
    return { id: criterion.id, group: criterion.group, title: criterion.title, taskIds: criterion.taskIds, documents, sourceFiles, aggregate: criterion.id <= 10 };
  });
}

export async function auditRegistry(root, value, fileStat = lstat) {
  const criteria = validateRegistry(value);
  const missing = [];
  for (const criterion of criteria) {
    for (const relative of [...criterion.documents, ...criterion.sourceFiles]) {
      try {
        const entry = await fileStat(path.join(root, relative));
        if (!entry.isFile()) missing.push({ criterionId: criterion.id, path: relative, reason: 'not a regular file' });
      } catch (error) {
        if (error.code !== 'ENOENT') throw error;
        missing.push({ criterionId: criterion.id, path: relative, reason: 'missing' });
      }
    }
  }
  return {
    scope: 'Metadata snapshot audit only, not behavior coverage or execution proof',
    requiresRefreshAfterGraphChange: true,
    criteriaCount: criteria.length,
    unlinked: criteria.filter(criterion => criterion.taskIds.length === 0).map(criterion => criterion.id),
    focusedWithoutChecklistDocument: criteria.filter(criterion => !criterion.aggregate && criterion.documents.length === 0).map(criterion => criterion.id),
    missingReferences: missing,
    criteria,
  };
}

async function main() {
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
  const raw = await readFile(path.join(root, 'results/acceptance/criteria-index.json'));
  const report = await auditRegistry(root, JSON.parse(raw));
  report.registrySha256 = createHash('sha256').update(raw).digest('hex');
  await mkdir(path.join(root, 'results/acceptance'), { recursive: true });
  await writeFile(path.join(root, 'results/acceptance/audit.json'), JSON.stringify(report, null, 2) + '\n');
  console.log(JSON.stringify({ criteria: report.criteriaCount, unlinked: report.unlinked, missingReferences: report.missingReferences, checklistDocumentMissing: report.focusedWithoutChecklistDocument }));
  if (report.unlinked.length || report.missingReferences.length) process.exitCode = 1;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await main();
