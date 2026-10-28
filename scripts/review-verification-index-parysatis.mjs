// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Independent regression for review finding VI-P1. No reference files change.
import assert from 'node:assert/strict';
import { mkdir, mkdtemp, writeFile, symlink, rm } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { digest, verifyReference, inventory } from './verification-index.mjs';
const root = fileURLToPath(new URL('../', import.meta.url));
await mkdir(path.join(root, 'tmp/review-parysatis'), { recursive: true });
const dir = await mkdtemp(path.join(root, 'tmp/review-parysatis/index-symlink-'));
try {
  await mkdir(path.join(dir, 'reference'));
  await mkdir(path.join(dir, 'outside'));
  await writeFile(path.join(dir, 'outside/source.ts'), 'same');
  await symlink(path.join(dir, 'outside'), path.join(dir, 'reference/linked'), process.platform === 'win32' ? 'junction' : 'dir');
  await assert.rejects(verifyReference(path.join(dir, 'reference'), [
    { path: 'linked/source.ts', bytes: 4, sha256: digest(Buffer.from('same')) }
  ]), /symbolic|symlink|junction|outside|regular|linked/i);
  await assert.rejects(inventory(path.join(dir, 'reference/linked')), /symbolic|symlink|junction|outside|regular|linked/i);
  console.log(JSON.stringify({ reviewer: 'Parysatis', control: 'VI-P1/VI-P2', result: 'reference component and evidence root links rejected', node: process.version, platform: process.platform }));
} finally {
  await rm(dir, { recursive: true });
}
