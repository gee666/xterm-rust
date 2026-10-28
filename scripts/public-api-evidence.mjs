// Shared evidence rules for the three public-API runners. Hash the production
// dependency files too, so a helper-only hash cannot endorse a changed engine.
import { createHash } from 'node:crypto';
import { readFileSync, readdirSync, statSync } from 'node:fs';

export function sha256(path) {
  return createHash('sha256').update(readFileSync(path)).digest('hex');
}

export function captureRustInputs(extraPaths, roots = ['crates/xterm-core/src']) {
  const paths = [
    'Cargo.toml', 'Cargo.lock', '.cargo/config.toml', 'crates/xterm-core/Cargo.toml',
    ...roots.flatMap(root => readdirSync(root, { recursive: true }).map(path => `${root}/${path}`)),
    ...extraPaths
  ].map(path => path.replaceAll('\\', '/')).filter(path => statSync(path).isFile());
  return Object.fromEntries([...new Set(paths)].sort().map(path => [path, sha256(path)]));
}

export function changedInputs(before, after) {
  return [...new Set([...Object.keys(before), ...Object.keys(after)])].sort()
    .filter(path => before[path] !== after[path]);
}
