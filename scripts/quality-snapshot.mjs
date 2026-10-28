import { createHash } from 'node:crypto';
import { readFile, readdir } from 'node:fs/promises';
import { relative, join, sep } from 'node:path';
const excluded = new Set(['target', 'node_modules', '.git', 'tmp']);
async function inputFiles(directory) {
  let entries;
  try { entries = await readdir(directory, { withFileTypes: true }); }
  catch (error) { if (error.code === 'ENOENT') return []; throw error; }
  const files = [];
  for (const item of entries) {
    const path = join(directory, item.name);
    if (item.isDirectory() && !excluded.has(item.name)) files.push(...await inputFiles(path));
    else if (item.isSymbolicLink()) throw new Error(`Review symlinked quality input before hashing: ${path}`);
    else if (item.isFile()) files.push(path);
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
