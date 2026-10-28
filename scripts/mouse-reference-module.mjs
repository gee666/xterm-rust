// Copyright (c) 2019-2026 The xterm.js authors. All rights reserved. MIT license.
// Shared unchanged-source loading for mouse probes; no production engine use.
import { createRequire } from 'node:module';
import { readFileSync, realpathSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import vm from 'node:vm';

export function loadMouseReference() {
  const root = fileURLToPath(new URL('../', import.meta.url));
  const require = createRequire(new URL('../source_repo/package.json', import.meta.url));
  const esbuild = require('esbuild');
  let built;
  try {
    built = esbuild.buildSync({ absWorkingDir: root,
      entryPoints: ['source_repo/src/common/services/MouseStateService.ts'],
      bundle: true, platform: 'node', format: 'cjs', write: false, metafile: true, logLevel: 'silent' });
  } finally { esbuild.stop(); }
  const module = { exports: {} };
  vm.runInNewContext(built.outputFiles[0].text, { module, exports: module.exports });
  const hash = bytes => createHash('sha256').update(bytes).digest('hex');
  const referenceRoot = realpathSync(path.join(root, 'source_repo'));
  const sourceHashes = Object.fromEntries(Object.keys(built.metafile.inputs).map(file => {
    // esbuild returns absolute inputs when an older worktree has a junction.
    const actual = realpathSync(path.resolve(root, file));
    const relative = path.relative(referenceRoot, actual).replaceAll('\\', '/');
    if (!relative || relative.startsWith('../') || path.isAbsolute(relative)) {
      throw new Error(`Mouse oracle input escaped the source reference: ${file}`);
    }
    return [`source_repo/${relative}`, hash(readFileSync(actual))];
  }));
  return {
    ...module.exports, esbuild: esbuild.version, sourceHashes,
    moduleSha256: hash(built.outputFiles[0].text),
  };
}
