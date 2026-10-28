import { createRequire } from 'node:module';
import { readFile } from 'node:fs/promises';
import { join } from 'node:path';
import { bootstrapSource } from './test-page.mjs';

// This is a test transport, not a distribution or benchmark bundle. Embedding
// the actual WASM bytes permits synchronous initialization without changing the
// original suites' page.goto or constructor calls.
export async function buildBootstrap({ root, referenceRoot, outputFile }) {
  const { build } = createRequire(join(referenceRoot, 'package.json'))('esbuild');
  const wasm = await readFile(join(root, 'browser/pkg/xterm_web_bg.wasm'));
  await build({ stdin: { contents: bootstrapSource(wasm.toString('base64')), resolveDir: root,
    sourcefile: 'rust-upstream-bootstrap.mjs' }, outfile: outputFile,
    bundle: true, format: 'iife', platform: 'browser', target: 'es2022', logLevel: 'silent' });
}
