// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
import assert from 'node:assert/strict';
import { lstat, readdir } from 'node:fs/promises';
import path from 'node:path';
export const EXCLUDED_NAMES = Object.freeze(['.git', 'node_modules', 'out', 'out-test', 'out-esbuild', 'out-esbuild-test', 'lib', 'coverage']);
export const EXCLUDED_PATHS = Object.freeze(['source_repo/demo/dist', 'source_repo/headless/lib-headless']);

/** Exact generated-output exclusions, testable without modifying source_repo. */
export async function listReferenceFiles(root) {
  async function walk(dir) {
    const directory = await lstat(path.join(root, dir));
    assert(directory.isDirectory() && !directory.isSymbolicLink(), `Reference directory must not be linked: ${dir}`);
    const files = [];
    for (const entry of await readdir(path.join(root, dir), { withFileTypes: true })) {
      const name = `${dir}/${entry.name}`;
      if (entry.isSymbolicLink()) throw new Error(`Reference symlink requires explicit policy: ${name}`);
      if (entry.isDirectory()) {
        if (!EXCLUDED_NAMES.includes(entry.name) && !EXCLUDED_PATHS.includes(name)) files.push(...await walk(name));
      } else if (entry.isFile()) files.push(name);
    }
    return files.sort();
  }
  return walk('source_repo');
}
