import { readFile, writeFile, mkdir, lstat, realpath, link, unlink, statfs } from 'node:fs/promises';
import { createHash, randomUUID } from 'node:crypto';
import path from 'node:path';
import { regularFile } from './acceptance-validation.mjs';

const MINIMUM_FREE_BYTES = 1024n * 1024n * 1024n;
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const relative = (root, file) => path.relative(root, file).replaceAll('\\', '/');
function descendant(root, file) {
  const value = path.relative(root, file);
  return value && value !== '..' && !value.startsWith(`..${path.sep}`) && !path.isAbsolute(value);
}
async function freeBytes(root) {
  const value = await statfs(root, { bigint: true });
  return value.bavail * value.bsize;
}

/** Pin compiler-produced bytes before execution. The logical Cargo target may
 * be the coordinator's canonical junction; no source/result symlink policy is
 * relaxed. Only descendants of that Cargo cache are accepted. */
export async function pinArtifact(root, compilerArtifact, { observeFree = freeBytes } = {}) {
  const logicalTarget = path.resolve(root, 'target');
  const canonicalTarget = await realpath(logicalTarget);
  const source = path.resolve(compilerArtifact);
  const base = descendant(logicalTarget, source) ? logicalTarget : descendant(canonicalTarget, source) ? canonicalTarget : null;
  if (!base) throw new Error('compiler artifact is outside the shared Cargo target');
  let current = base;
  const parts = path.relative(base, source).split(path.sep);
  for (const [index, part] of parts.entries()) {
    current = path.join(current, part);
    const info = await lstat(current);
    if (info.isSymbolicLink() || (index < parts.length - 1 ? !info.isDirectory() : !info.isFile())) {
      throw new Error('compiler artifact has a symbolic or nonregular descendant');
    }
  }
  if (!descendant(canonicalTarget, await realpath(source))) throw new Error('compiler artifact escapes its target');
  const before = await lstat(source, { bigint: true });
  const bytes = await readFile(source);
  const after = await lstat(source, { bigint: true });
  if (before.size !== after.size || before.mtimeNs !== after.mtimeNs || before.ino !== after.ino) {
    throw new Error('compiler artifact changed while being pinned');
  }
  const sha256 = digest(bytes);
  const extension = path.extname(source).toLowerCase();
  if (extension && !['.exe', '.wasm', '.dll', '.so', '.dylib', '.rlib'].includes(extension)) throw new Error('unknown compiler artifact kind');
  const destination = path.join(root, 'results/acceptance-artifacts', sha256, `artifact${extension}`);
  const outputPath = relative(root, destination);
  try {
    const existing = await regularFile(root, outputPath);
    if (digest(await readFile(existing)) !== sha256) throw new Error('pinned artifact content was changed');
    return { path: outputPath, sha256, bytes: bytes.length };
  } catch (error) {
    if (error.code !== 'ENOENT') throw error;
  }
  if (await observeFree(root) - BigInt(bytes.length) < MINIMUM_FREE_BYTES) throw new Error('insufficient disk headroom to retain artifact bytes');
  await mkdir(path.dirname(destination), { recursive: true });
  await mkdir(path.join(root, 'tmp'), { recursive: true });
  const temporary = path.join(root, 'tmp', `artifact-pin-${randomUUID()}`);
  await writeFile(temporary, bytes, { flag: 'wx', mode: Number(before.mode & 0o777n) });
  try {
    try { await link(temporary, destination); }
    catch (error) { if (error.code !== 'EEXIST') throw error; }
    const pinned = await regularFile(root, outputPath);
    if (digest(await readFile(pinned)) !== sha256) throw new Error('concurrent artifact pin differs from its digest');
  } finally {
    await unlink(temporary);
  }
  return { path: outputPath, sha256, bytes: bytes.length };
}
