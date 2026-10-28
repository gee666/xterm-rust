// Prepare the unchanged reference component for paired browser assertions.
import { createRequire } from 'node:module';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const root = fileURLToPath(new URL('../', import.meta.url));
const require = createRequire(path.join(root, 'source_repo/package.json'));
const esbuild = require('esbuild');
await mkdir(path.join(root, 'tmp/core-browser-source'), { recursive: true });
try {
  await esbuild.build({
    absWorkingDir: root,
    entryPoints: ['source_repo/src/browser/services/CoreBrowserService.ts'],
    outfile: path.join(root, 'tmp/core-browser-source/reference.mjs'),
    bundle: true,
    platform: 'browser',
    format: 'esm',
    target: 'es2022',
    logLevel: 'silent',
  });
  console.log('Prepared unchanged CoreBrowserService reference for browser tests.');
} finally { esbuild.stop(); }
