// Explicit shipped-file accounting. No dependency or build-directory guessing.
import { createHash } from 'node:crypto';
import { lstatSync, readdirSync, readFileSync, mkdirSync, writeFileSync } from 'node:fs';
import { dirname, resolve, relative, isAbsolute, join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { gzipSync, gunzipSync, brotliCompressSync, brotliDecompressSync, constants } from 'node:zlib';

const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
const slash = value => value.replaceAll('\\', '/');
export const compression = Object.freeze({ gzip: { level: 9 }, brotli: { quality: 11, mode: 'generic', lgwin: 22 } });
function fail(message) { throw new Error(message); }
function text(value, field) {
  if (typeof value !== 'string' || !value.trim()) fail(`${field} must be nonempty text`);
  return value;
}
function local(value, field, allowRoot = false) {
  text(value, field);
  if (allowRoot && value === '.') return value;
  if (value.includes('\\') || value.includes(':') || value.includes('\0') || value.startsWith('/') ||
      value.split('/').some(part => !part || part === '.' || part === '..')) fail(`${field} must be a normalized relative path`);
  return value;
}
function directory(path) {
  const info = lstatSync(path);
  if (info.isSymbolicLink() || !info.isDirectory()) fail(`Not a real directory: ${path}`);
}
function descend(root, path) {
  let current = root;
  for (const part of path.split('/')) {
    current = join(current, part);
    const info = lstatSync(current);
    if (info.isSymbolicLink()) fail(`Linked entry is not a shipped file: ${path}`);
  }
  return current;
}
function files(root, prefix = '') {
  const result = [];
  for (const name of readdirSync(root).sort()) {
    const path = prefix ? `${prefix}/${name}` : name;
    const absolute = join(root, name), info = lstatSync(absolute);
    if (info.isSymbolicLink()) fail(`Linked entry is not a shipped file: ${path}`);
    if (info.isDirectory()) result.push(...files(absolute, path));
    else if (info.isFile()) result.push(path);
    else fail(`Nonregular distribution entry: ${path}`);
  }
  return result;
}
export function compressed(bytes) {
  const gzip = gzipSync(bytes, { level: compression.gzip.level });
  const brotli = brotliCompressSync(bytes, { params: {
    [constants.BROTLI_PARAM_QUALITY]: compression.brotli.quality,
    [constants.BROTLI_PARAM_MODE]: constants.BROTLI_MODE_GENERIC,
    [constants.BROTLI_PARAM_LGWIN]: compression.brotli.lgwin
  } });
  if (!gunzipSync(gzip).equals(bytes) || !brotliDecompressSync(brotli).equals(bytes)) fail('Compression round trip failed');
  return { gzip: gzip.length, brotli: brotli.length };
}
export function inventory(manifest, root) {
  directory(root);
  if (manifest.schemaVersion !== 1 || !Array.isArray(manifest.distributions) || !manifest.distributions.length) fail('Expected schemaVersion1 and distributions');
  const ids = new Set();
  const distributions = manifest.distributions.map(distribution => {
    const id = text(distribution.id, 'id');
    if (ids.has(id)) fail(`Repeated distribution id: ${id}`);
    ids.add(id);
    if (!['web', 'native', 'headless-js'].includes(distribution.kind)) fail(`Unknown distribution kind: ${id}`);
    if (distribution.profile !== 'release') fail(`Only release distributions can be measured: ${id}`);
    text(distribution.scope, 'scope');
    if (!Array.isArray(distribution.addons) || distribution.addons.some(addon => typeof addon !== 'string')) fail(`Expected explicit addons: ${id}`);
    if (!distribution.build || !Array.isArray(distribution.build.commands) || !distribution.build.commands.length ||
        distribution.build.commands.some(command => typeof command !== 'string' || !command.trim())) fail(`Expected build commands: ${id}`);
    text(distribution.build.toolchain, 'build.toolchain');
    text(distribution.build.flags, 'build.flags');
    if (!/^[a-f0-9]{64}$/.test(distribution.build.sourceSha256)) fail(`Expected build source SHA256: ${id}`);
    const folder = local(distribution.directory, 'directory', true);
    const location = folder === '.' ? root : descend(root, folder);
    directory(location);
    if (!Array.isArray(distribution.files) || !distribution.files.length) fail(`No shipped files: ${id}`);
    const selected = new Map();
    for (const entry of distribution.files) {
      const path = local(entry.path, 'file.path'), role = text(entry.role, 'file.role');
      if (selected.has(path) && selected.get(path) !== role) fail(`Conflicting repeated file: ${path}`);
      selected.set(path, role);
    }
    const excluded = new Map();
    for (const entry of distribution.excluded ?? []) {
      const path = local(entry.path, 'excluded.path');
      if (selected.has(path) || excluded.has(path)) fail(`Conflicting excluded file: ${path}`);
      excluded.set(path, text(entry.reason, 'excluded.reason'));
    }
    const discovered = files(location);
    for (const path of discovered) if (!selected.has(path) && !excluded.has(path)) fail(`Undeclared distribution asset: ${id}/${path}`);
    for (const path of [...selected.keys(), ...excluded.keys()]) if (!discovered.includes(path)) fail(`Missing distribution file: ${id}/${path}`);
    const web = distribution.kind !== 'native';
    const entries = [...selected].sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0).map(([path, role]) => {
      const bytes = readFileSync(descend(location, path));
      return { path, role, sha256: sha256(bytes), raw: bytes.length, ...(web ? compressed(bytes) : {}) };
    });
    const totals = entries.reduce((sum, entry) => {
      for (const key of web ? ['raw', 'gzip', 'brotli'] : ['raw']) sum[key] += entry[key];
      return sum;
    }, web ? { raw: 0, gzip: 0, brotli: 0 } : { raw: 0 });
    return { ...distribution, files: entries, excluded: [...excluded].sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0)
      .map(([path, reason]) => ({ path, reason })), totals };
  }).sort((a, b) => a.id < b.id ? -1 : a.id > b.id ? 1 : 0);
  return { schemaVersion: 1, tool: 'distribution-size.mjs v1', node: process.version,
    codecs: { zlib: process.versions.zlib, brotli: process.versions.brotli }, compression,
    method: 'Unique normalized file paths per distribution; sum independently compressed files, not archive bytes. Native raw bytes are not JS download bytes.',
    status: 'Measured declared artifacts only. Manifest provenance is caller-supplied; neither equivalent functionality nor matched-feature coverage is established by sizes.',
    distributions };
}
export function main(args = process.argv.slice(2)) {
  const options = {};
  for (let i = 0; i < args.length; i += 2) {
    if (!['--manifest', '--root', '--output'].includes(args[i]) || options[args[i]] || !args[i + 1] || args[i + 1].startsWith('--')) fail('Usage: --manifest FILE --root DIRECTORY --output FILE');
    options[args[i]] = resolve(args[i + 1]);
  }
  if (Object.keys(options).length !== 3) fail('Usage: --manifest FILE --root DIRECTORY --output FILE');
  const within = relative(options['--root'], options['--output']);
  if (!isAbsolute(within) && within !== '..' && !within.startsWith(`..${process.platform === 'win32' ? '\\' : '/'}`)) fail('Output must be outside the distribution root');
  const report = inventory(JSON.parse(readFileSync(options['--manifest'], 'utf8')), options['--root']);
  report.manifestSha256 = sha256(readFileSync(options['--manifest']));
  report.invocation = { manifest: slash(options['--manifest']), root: slash(options['--root']) };
  mkdirSync(dirname(options['--output']), { recursive: true });
  writeFileSync(options['--output'], JSON.stringify(report, null, 2) + '\n');
  console.log(JSON.stringify(report, null, 2));
  return report;
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  try { main(); } catch (error) { console.error(error.message); process.exitCode = 1; }
}
