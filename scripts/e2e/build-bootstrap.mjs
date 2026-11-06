import { createRequire } from 'node:module';
import { stat } from 'node:fs/promises';
import { join } from 'node:path';
import { bootstrapSource } from './test-page.mjs';

// This is test transport, not a distribution or benchmark bundle. The actual
// WASM compiles asynchronously; a fixture readiness image delays navigation
// until bindings are installed, without changing source constructor calls.
export async function buildBootstrap({ root, referenceRoot, outputFile }) {
  const { build } = createRequire(join(referenceRoot, 'package.json'))('esbuild');
  const wasm = await stat(join(root, 'browser/pkg/xterm_web_bg.wasm'));
  if (!wasm.isFile()) throw new Error('E2E WASM package must be a regular file');
  await build({ stdin: { contents: bootstrapSource(), resolveDir: root,
    sourcefile: 'rust-upstream-bootstrap.mjs' }, outfile: outputFile,
    // Private addon contracts inspect real binding constructor names. Transport
    // bundling must not rename those classes a second time.
    bundle: true, keepNames: true, format: 'iife', platform: 'browser', target: 'es2022', logLevel: 'silent' });
}
