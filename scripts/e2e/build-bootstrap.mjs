import { createRequire } from 'node:module';
import { readFile } from 'node:fs/promises';
import { join } from 'node:path';
import { bootstrapSource } from './test-page.mjs';

// This is test transport, not a distribution or benchmark bundle. The actual
// WASM compiles asynchronously; a fixture readiness image delays navigation
// until bindings are installed, without changing source constructor calls.
export async function buildBootstrap({ root, referenceRoot, outputFile }) {
  const { build } = createRequire(join(referenceRoot, 'package.json'))('esbuild');
  const wasm = await readFile(join(root, 'browser/pkg/xterm_web_bg.wasm'));
  await build({ stdin: { contents: bootstrapSource(wasm.toString('base64')), resolveDir: root,
    sourcefile: 'rust-upstream-bootstrap.mjs' }, outfile: outputFile,
    bundle: true, format: 'iife', platform: 'browser', target: 'es2022', logLevel: 'silent' });
}
