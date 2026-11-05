// Preserve the unchanged reference component for paired browser assertions.
import { createRequire } from 'node:module';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const root = fileURLToPath(new URL('../', import.meta.url));
const args = process.argv.slice(2);
if (args.length > 1 || args.length === 1 && args[0] !== '--check') throw new Error('Usage: node scripts/prepare-core-browser-source.mjs [--check]');
const check = args.length === 1;
const require = createRequire(path.join(root, 'source_repo/package.json'));
const esbuild = require('esbuild');
const directory = path.join(root, 'crates/xterm-web/tests/reference');
const output = path.join(directory, 'core-browser-source.mjs');
const manifestPath = path.join(directory, 'core-browser-source.json');
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
try {
  const result = await esbuild.build({
    absWorkingDir: root,
    entryPoints: ['source_repo/src/browser/services/CoreBrowserService.ts'],
    outfile: output,
    bundle: true,
    write: false,
    metafile: true,
    platform: 'browser',
    format: 'esm',
    target: 'es2022',
    logLevel: 'silent',
    banner: { js: '// Copyright (c) 2014-2026 The xterm.js authors. MIT license.\n// Generated unchanged test-only oracle, never production code.\n// Source hashes and license provenance are recorded in core-browser-source.json.' },
  });
  if (result.outputFiles.length !== 1) throw new Error('Expected one self-contained reference module');
  const bytes = result.outputFiles[0].contents;
  const sourceSha256 = {};
  for (const file of [...Object.keys(result.metafile.inputs), 'source_repo/LICENSE'].sort()) {
    const relative = file.replaceAll('\\', '/');
    if (!relative.startsWith('source_repo/')) throw new Error(`Unexpected non-reference input: ${file}`);
    sourceSha256[relative] = sha256(await readFile(path.join(root, file)));
  }
  const manifest = JSON.stringify({ generator: 'node scripts/prepare-core-browser-source.mjs',
    esbuild: esbuild.version, scope: 'Unchanged original CoreBrowserService and imported dependencies, test use only.',
    sourceSha256, generatedSha256: sha256(bytes) }, null, 2) + '\n';
  if (check) {
    if (!Buffer.from(bytes).equals(await readFile(output)) || manifest !== await readFile(manifestPath, 'utf8')) {
      throw new Error('Tracked CoreBrowserService reference is stale; regenerate from the unchanged source');
    }
  } else {
    await mkdir(directory, { recursive: true });
    await writeFile(output, bytes);
    await writeFile(manifestPath, manifest);
  }
  console.log(`${check ? 'Verified' : 'Prepared'} unchanged CoreBrowserService reference for browser tests.`);
} finally { esbuild.stop(); }
