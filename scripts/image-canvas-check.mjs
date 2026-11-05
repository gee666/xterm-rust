// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
// Bundle the unchanged comparator, then execute the full Rust browser crate.
import { createRequire } from 'node:module';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { mkdir, readFile, writeFile, readdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const test = process.env.WASM_TEST ?? 'image_canvas_browser';
const terminalTest = ['image_addon_browser', 'image_iip_terminal_browser', 'image_sixel_terminal_browser', 'image_kitty_terminal_browser'].includes(test);
if (!['image_addon_browser', 'image_canvas_browser', 'image_lifecycle_browser', 'image_iip_terminal_browser', 'image_sixel_terminal_browser', 'image_kitty_terminal_browser'].includes(test)) throw new Error('Invalid image test suite');
if (process.argv.length > 2) throw new Error('Image checks now compile the full browser crate; isolated-component mode was retired');
const reference = resolve(process.env.SOURCE_REPO ?? resolve(root, 'source_repo'));
const dependencies = resolve(process.env.REFERENCE_NODE_MODULES ?? resolve(reference, 'node_modules'));
const require = createRequire(resolve(dependencies, '../package.json'));
const work = resolve(root, 'tmp/image-canvas');
const resultNames = { image_addon_browser: 'image-addon', image_canvas_browser: 'image-canvas', image_lifecycle_browser: 'image-lifecycle', image_iip_terminal_browser: 'image-iip-terminal', image_sixel_terminal_browser: 'image-sixel-terminal', image_kitty_terminal_browser: 'image-kitty-terminal' };
const results = resolve(root, 'results', resultNames[test]);
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
  const exports = [...(test === 'image_addon_browser' ? ['ImageAddon'] : []), 'ImageRenderer', 'IIPHandler', ...(terminalTest ? ['ImageStorage', 'IIPImageStorage'] : []),
    ...(test === 'image_sixel_terminal_browser' ? ['SixelHandler', 'SixelImageStorage'] : [])].map(name =>
    `export { ${name} } from ${JSON.stringify(resolve(reference, `addons/addon-image/src/${name}.ts`))};`
  );
  if (test === 'image_kitty_terminal_browser') {
    for (const name of ['KittyGraphicsHandler', 'KittyImageStorage']) exports.push(
      `export { ${name} } from ${JSON.stringify(resolve(reference, `addons/addon-image/src/kitty/${name}.ts`))};`
    );
  }
  if (terminalTest) exports.push(`export { Terminal } from ${JSON.stringify(resolve(reference, 'src/browser/public/Terminal.ts'))};`);
  await writeFile(entry, exports.join('\n'));
  const build = await esbuild.build({
    entryPoints: [entry],
    bundle: true, write: false, platform: 'browser', format: 'iife',
    globalName: 'SourceImageRendererBundle',
    alias: { common: resolve(reference, 'src/common'),
      'xterm-wasm-parts': resolve(dependencies, 'xterm-wasm-parts') },
    nodePaths: [dependencies], metafile: true
  });
  await writeFile(oracle, build.outputFiles[0].text + '\nObject.keys(SourceImageRendererBundle).forEach(name => { globalThis[`Source${name}`] = SourceImageRendererBundle[name]; });\n');
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
  for (const directory of ['crates/xterm-web/src/image', 'crates/xterm-web/tests/image_canvas_support', 'crates/xterm-web/tests/image_lifecycle_support', 'crates/xterm-addons/src/image/qoi', 'crates/xterm-addons/src/image/iip', 'crates/xterm-addons/src/image/sixel', 'crates/xterm-addons/src/image/kitty']) {
    async function collect(directory) {
      for (const entry of await readdir(resolve(root, directory), { withFileTypes: true })) {
        const path = `${directory}/${entry.name}`;
        if (entry.isDirectory()) await collect(path);
        else if (entry.isFile()) rustInputs.push({ path, sha256: sha256(await readFile(resolve(root, path))) });
        else throw new Error(`Unexpected linked image input ${path}`);
      }
    }
    await collect(directory);
  }
  for (const path of [`crates/xterm-web/tests/${test}.rs`, 'Cargo.lock',
    'scripts/image-canvas-check.mjs', 'scripts/browser-host-check.mjs',
    ...(terminalTest ? ['crates/xterm-web/tests/image_terminal_support/mod.rs', 'crates/xterm-web/tests/image_terminal_support/patch.rs', 'crates/xterm-web/tests/image_terminal_support/snapshot.rs'] : []),
    ...(test === 'image_addon_browser' ? ['crates/xterm-web/tests/image_addon_support/mod.rs', 'crates/xterm-web/tests/image_addon_support/bootstrap.rs', 'crates/xterm-web/tests/image_addon_support/pending.rs', 'crates/xterm-web/tests/image_addon_support/inspection.rs', 'crates/xterm-web/tests/image_addon_support/inspection_activation.rs', 'crates/xterm-web/tests/image_addon_support/handlers.rs', 'crates/xterm-web/tests/image_addon_support/options.rs', 'crates/xterm-web/tests/image_addon_support/constructor_order.rs', 'crates/xterm-web/tests/image_addon_support/raw_placeholder.rs', 'crates/xterm-web/tests/image_addon_support/raw_storage_limit.rs', 'crates/xterm-web/tests/image_addon_support/raw_sixel_limit.rs', 'crates/xterm-web/tests/image_addon_support/raw_kitty_limit.rs', 'crates/xterm-web/tests/image_addon_support/raw_iip_limit.rs', 'crates/xterm-web/tests/image_addon_support/visibility.rs', 'crates/xterm-addons/src/image/storage/mod.rs', 'crates/xterm-addons/src/image/storage/budget.rs', 'crates/xterm-addons/tests/image_storage_cases/budget.rs', 'crates/xterm-web/tests/image_addon_support/render_dimensions.rs', 'crates/xterm-web/src/terminal/internal_render.rs', 'crates/xterm-web/src/terminal/internal_render.js', 'crates/xterm-web/src/terminal/internal_api.rs', 'crates/xterm-web/src/terminal/browser_open.rs', 'crates/xterm-web/src/terminal/mod.rs', 'browser/xterm.mjs'] : []),
    ...(test === 'image_sixel_terminal_browser' ? ['crates/xterm-web/tests/image_sixel_terminal_support/mod.rs', 'crates/xterm-web/tests/image_sixel_terminal_support/failures.rs'] : []),
    ...(test === 'image_kitty_terminal_browser' ? ['crates/xterm-web/tests/image_kitty_terminal_support/mod.rs', 'crates/xterm-web/tests/image_kitty_terminal_support/failures.rs', 'crates/xterm-web/tests/image_kitty_terminal_support/async_cases.rs', 'crates/xterm-web/tests/image_kitty_terminal_support/lifetime.rs', 'crates/xterm-core/src/public_api/parser.rs'] : []),
    ...(test === 'image_iip_terminal_browser' ? ['crates/xterm-web/tests/image_iip_terminal_support/mod.rs', 'crates/xterm-web/tests/image_iip_terminal_support/failures.rs'] : [])]) {
    rustInputs.push({ path, sha256: sha256(await readFile(resolve(root, path))) });
  }
  const passed = browsers.status === 'passed';
  await persist({
    status: passed ? (test === 'image_addon_browser' ? 'passed-directed-addon-cases' : terminalTest ? `passed-${test}-producer` : 'passed-renderer-component') : 'failed',
    isolatedComponent: false, browserReport, rustInputs, sourceInputs,
    seed: test === 'image_canvas_browser' ? '0x272389' : test === 'image_iip_terminal_browser' ? '0x1337365' : test === 'image_sixel_terminal_browser' ? '0x51370370' : test === 'image_kitty_terminal_browser' ? '0x4b373' : null,
    seededDrawAndExtractPairsPerBrowser: test === 'image_canvas_browser' ? 128 : 0,
    remaining: ['complete ImageAddon/parser/browser integration and original image suites', 'native OS and full workspace gates']
  });
  if (!passed) process.exitCode = 1;
} catch (error) {
  await persist({ status: 'failed', error: error.stack ?? error.message });
  throw error;
}
