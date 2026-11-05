// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
// Bundle the unchanged comparator, then execute the full Rust browser crate.
import { createRequire } from 'node:module';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { mkdir, readFile, writeFile, readdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const test = process.env.WASM_TEST ?? 'image_canvas_browser';
if (!['image_canvas_browser', 'image_lifecycle_browser'].includes(test)) throw new Error('Invalid image test suite');
if (process.argv.length > 2) throw new Error('Image checks now compile the full browser crate; isolated-component mode was retired');
const reference = resolve(process.env.SOURCE_REPO ?? resolve(root, 'source_repo'));
const dependencies = resolve(process.env.REFERENCE_NODE_MODULES ?? resolve(reference, 'node_modules'));
const require = createRequire(resolve(dependencies, '../package.json'));
const work = resolve(root, 'tmp/image-canvas');
const results = resolve(root, test === 'image_canvas_browser' ? 'results/image-canvas' : 'results/image-lifecycle');
await mkdir(work, { recursive: true });
await mkdir(results, { recursive: true });
const verificationPath = resolve(results, 'verification.json');
const persist = report => writeFile(verificationPath, JSON.stringify({
  fullImageAddonEquivalence: 'NOT PROVEN', test, ...report
}, null, 2) + '\n');
await persist({ status: 'preparing' });
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
try {
  const esbuild = require('esbuild');
  const oracle = resolve(work, 'source-renderer.js');
  const entry = resolve(work, 'image-comparators.ts');
  await writeFile(entry, ['ImageRenderer', 'IIPHandler'].map(name =>
    `export { ${name} } from ${JSON.stringify(resolve(reference, `addons/addon-image/src/${name}.ts`))};`
  ).join('\n'));
  const build = await esbuild.build({
    entryPoints: [entry],
    bundle: true, write: false, platform: 'browser', format: 'iife',
    globalName: 'SourceImageRendererBundle',
    alias: { common: resolve(reference, 'src/common'),
      'xterm-wasm-parts': resolve(dependencies, 'xterm-wasm-parts') },
    nodePaths: [dependencies], metafile: true
  });
  await writeFile(oracle, build.outputFiles[0].text + '\nglobalThis.SourceImageRenderer = SourceImageRendererBundle.ImageRenderer;\nglobalThis.SourceIIPHandler = SourceImageRendererBundle.IIPHandler;\n');
  const sourceInputs = [];
  for (const name of Object.keys(build.metafile.inputs)) {
    const path = resolve(name);
    sourceInputs.push({ path, sha256: sha256(await readFile(path)) });
  }
  await writeFile(resolve(results, 'source-inputs.json'), JSON.stringify({
    inputs: sourceInputs, esbuild: esbuild.version
  }, null, 2) + '\n');
  process.env.WASM_TEST = test;
  process.env.WASM_PRELUDE = oracle;
  await persist({ status: 'running' });
  await import('./browser-host-check.mjs');
  const browserReport = `results/browser-host/${test}-results.json`;
  const browsers = JSON.parse(await readFile(resolve(root, browserReport), 'utf8'));
  const rustInputs = [];
  for (const directory of ['crates/xterm-web/src/image', 'crates/xterm-web/tests/image_canvas_support', 'crates/xterm-web/tests/image_lifecycle_support', 'crates/xterm-addons/src/image/qoi', 'crates/xterm-addons/src/image/iip']) {
    for (const file of await readdir(resolve(root, directory))) {
      const path = `${directory}/${file}`;
      rustInputs.push({ path, sha256: sha256(await readFile(resolve(root, path))) });
    }
  }
  for (const path of [`crates/xterm-web/tests/${test}.rs`, 'Cargo.lock',
    'scripts/image-canvas-check.mjs', 'scripts/browser-host-check.mjs']) {
    rustInputs.push({ path, sha256: sha256(await readFile(resolve(root, path))) });
  }
  const passed = browsers.status === 'passed';
  await persist({
    status: passed ? 'passed-renderer-component' : 'failed',
    isolatedComponent: false, browserReport, rustInputs, sourceInputs,
    seed: test === 'image_canvas_browser' ? '0x272389' : null,
    seededDrawAndExtractPairsPerBrowser: test === 'image_canvas_browser' ? 128 : 0,
    remaining: ['complete ImageAddon/parser/browser integration and original image suites', 'native OS and full workspace gates']
  });
  if (!passed) process.exitCode = 1;
} catch (error) {
  await persist({ status: 'failed', error: error.stack ?? error.message });
  throw error;
}
