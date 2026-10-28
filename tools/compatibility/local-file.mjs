// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
import assert from 'node:assert/strict';
import { lstat, readFile } from 'node:fs/promises';
import path from 'node:path';

/** Receipts/configuration are data, never authority to follow an external link. */
export async function validateLocalFile(root, relative, kind = 'mapped') {
  assert(typeof relative === 'string' && !relative.includes('\\') && !relative.includes(':')
    && !path.posix.isAbsolute(relative) && relative.split('/').every(part => part && part !== '.' && part !== '..'), `Invalid ${kind} path: ${relative}`);
  const label = kind[0].toUpperCase() + kind.slice(1);
  let current = root;
  const directory = await lstat(current);
  assert(directory.isDirectory() && !directory.isSymbolicLink(), `${label} root must not be linked`);
  let info;
  for (const part of relative.split('/')) {
    current = path.join(current, part);
    info = await lstat(current);
    assert(!info.isSymbolicLink(), `${label} file must not be linked: ${relative}`);
  }
  assert(info.isFile(), `${label} path must be a regular file: ${relative}`);
}

export async function readLocalFile(root, relative, kind = 'mapped') {
  await validateLocalFile(root, relative, kind);
  return readFile(path.join(root, relative));
}
