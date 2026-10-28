import { createHash } from 'node:crypto';
import { readFile, readdir, lstat } from 'node:fs/promises';
import { relative, join, sep, isAbsolute } from 'node:path';
const excluded = new Set(['target', 'node_modules', '.git', 'tmp']);
async function inputFiles(directory, strict = false) {
  let entries;
  try { entries = await readdir(directory, { withFileTypes: true }); }
  catch (error) { if (error.code === 'ENOENT') return []; throw error; }
  const files = [];
  for (const item of entries) {
    const path = join(directory, item.name);
    if (item.isDirectory() && !excluded.has(item.name)) files.push(...await inputFiles(path, strict));
    else if (item.isSymbolicLink()) throw new Error(`Review symlinked quality input before hashing: ${path}`);
    else if (item.isFile()) files.push(path);
    else if (strict && !item.isDirectory()) throw new Error(`Nonregular quality input: ${path}`);
  }
  return files;
}
/** Hash fixtures/glue/tooling as well as Rust. Test data changes invalidate evidence. */
export async function snapshot(root) {
  const files = [join(root, 'Cargo.toml'), join(root, 'Cargo.lock')];
  for (const directory of ['crates', 'scripts', 'tools', 'tests', 'browser', 'examples', 'demo', 'assets', '.cargo', '.github']) {
    files.push(...await inputFiles(join(root, directory)));
  }
  for (const name of ['rust-toolchain', 'rust-toolchain.toml', '.gitattributes']) {
    try { await readFile(join(root, name)); files.push(join(root, name)); }
    catch (error) { if (error.code !== 'ENOENT') throw error; }
  }
  return hashFiles(root, files);
}

/** A scoped receipt proves only these selected inputs, never the whole workspace. */
export async function snapshotPaths(root, names) {
  if (!Array.isArray(names) || names.length === 0) throw new Error('Quality input paths are required');
  const rootStat = await lstat(root);
  if (!rootStat.isDirectory() || rootStat.isSymbolicLink()) throw new Error('Quality input root must be an unlinked directory');
  const files = new Set();
  for (const name of names) {
    if (typeof name !== 'string' || isAbsolute(name) || name.includes(':') || name.includes('\\')
      || name.split('/').some(part => !part || part === '.' || part === '..')) throw new Error(`Invalid quality input path: ${name}`);
    let current = root;
    let stat;
    for (const part of name.split('/')) {
      current = join(current, part);
      stat = await lstat(current);
      if (stat.isSymbolicLink()) throw new Error(`Linked quality input: ${name}`);
    }
    if (stat.isDirectory()) for (const file of await inputFiles(current, true)) files.add(file);
    else if (stat.isFile()) files.add(current);
    else throw new Error(`Nonregular quality input: ${name}`);
  }
  return { ...await hashFiles(root, [...files]), scope: [...new Set(names)].sort() };
}

async function hashFiles(root, files) {
  const hash = createHash('sha256');
  const unsafeCandidates = [];
  for (const file of files.sort()) {
    const bytes = await readFile(file);
    const name = relative(root, file).split(sep).join('/');
    hash.update(name + '\0').update(bytes).update('\0');
    if (!file.endsWith('.rs')) continue;
    bytes.toString('utf8').split('\n').forEach((line, index) => {
      if (/\bunsafe\b/.test(line)) unsafeCandidates.push({ path: name, line: index + 1, text: line.trim() });
    });
  }
  return { sha256: hash.digest('hex'), files: files.length, unsafeCandidates };
}
