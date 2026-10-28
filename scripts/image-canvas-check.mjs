// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
// Compile the unchanged reference alongside real Rust/WASM canvas assertions.
import { createRequire } from 'node:module';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { mkdir, readFile, writeFile, copyFile, readdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const reference = resolve(process.env.SOURCE_REPO ?? resolve(root, 'source_repo'));
const dependencies = resolve(process.env.REFERENCE_NODE_MODULES ?? resolve(reference, 'node_modules'));
const require = createRequire(resolve(dependencies, '../package.json'));
const work = resolve(root, 'tmp/image-canvas');
await mkdir(work, { recursive: true });
await mkdir(resolve(root, 'results/image-canvas'), { recursive: true });
const verificationPath = resolve(root, 'results/image-canvas/verification.json');
await writeFile(verificationPath, JSON.stringify({ status: 'unrun', fullImageAddonEquivalence: 'NOT PROVEN' }) + '\n');
const esbuild = require('esbuild');
const oracle = resolve(work, 'source-renderer.js');
const entry = resolve(reference, 'addons/addon-image/src/ImageRenderer.ts');
const build = await esbuild.build({ entryPoints: [entry], bundle: true, write: false,
  platform: 'browser', format: 'iife', globalName: 'SourceImageRendererBundle',
  alias: { common: resolve(reference, 'src/common') }, nodePaths: [dependencies], metafile: true });
await writeFile(oracle, build.outputFiles[0].text + '\nglobalThis.SourceImageRenderer = SourceImageRendererBundle.ImageRenderer;\n');
const inputs = [];
for (const name of Object.keys(build.metafile.inputs)) {
  const path = resolve(name);
  inputs.push({ path, sha256: createHash('sha256').update(await readFile(path)).digest('hex') });
}
await mkdir(resolve(root, 'results/image-canvas'), { recursive: true });
await writeFile(resolve(root, 'results/image-canvas/source-inputs.json'), JSON.stringify({ inputs, esbuild: esbuild.version, componentOnly: true }, null, 2) + '\n');
process.env.WASM_TEST = 'image_canvas_browser';
process.env.WASM_PRELUDE = oracle;
process.env.CARGO_TARGET_DIR ??= resolve(root, 'target');
if (process.argv.includes('--isolated-component')) {
  // Explicit fallback while the shared browser crate has unresolved producers.
  // Compile the exact production files, not copies or simplified substitutes.
  const crate = resolve(work, 'component');
  await mkdir(resolve(crate, 'src/image'), { recursive: true });
  await copyFile(resolve(root, 'crates/xterm-web/src/image/placeholder.js'), resolve(crate, 'src/image/placeholder.js'));
  const slash = p => p.replaceAll('\\', '/');
  await writeFile(resolve(crate, 'lib.rs'), `#[path = ${JSON.stringify(slash(resolve(root, 'crates/xterm-web/src/image/mod.rs')))}]\npub mod image;\n`);
  await writeFile(resolve(crate, 'Cargo.toml'), `[package]\nname = "xterm-web-image-check"\nversion = "0.1.0"\nedition = "2021"\n[workspace]\n[lib]\nname = "xterm_web"\npath = "lib.rs"\n[[test]]\nname = "image_canvas_browser"\npath = ${JSON.stringify(slash(resolve(root, 'crates/xterm-web/tests/image_canvas_browser.rs')))}\n[dependencies]\nxterm-core = { path = ${JSON.stringify(slash(resolve(root, 'crates/xterm-core')))} }\nxterm-addons = { path = ${JSON.stringify(slash(resolve(root, 'crates/xterm-addons')))} }\nxterm-wasm-support = { path = ${JSON.stringify(slash(resolve(root, 'crates/xterm-wasm-support')))} }\nwasm-bindgen = "=0.2.100"\njs-sys = "=0.3.77"\nweb-sys = { version = "=0.3.77", features = ["Window", "Document", "Element", "HtmlElement", "Node", "console"] }\nwasm-bindgen-futures = "=0.4.50"\n[dev-dependencies]\nwasm-bindgen-test = "=0.3.50"\n[profile.dev]\ndebug = 0\nincremental = false\n[profile.test]\ndebug = 0\nincremental = false\n`);
  // Seed resolution from the retained repository lockfile; Cargo prunes only
  // unreachable workspace members and records the isolated root package.
  await copyFile(resolve(root, 'Cargo.lock'), resolve(crate, 'Cargo.lock'));
  process.env.WASM_MANIFEST = resolve(crate, 'Cargo.toml');
  process.env.WASM_PACKAGE = 'xterm-web-image-check';
}
await writeFile(verificationPath, JSON.stringify({ status: 'running', fullImageAddonEquivalence: 'NOT PROVEN' }) + '\n');
try { await import('./browser-host-check.mjs'); }
catch (error) {
  await writeFile(verificationPath, JSON.stringify({ status: 'failed', fullImageAddonEquivalence: 'NOT PROVEN', error: error.message }, null, 2) + '\n');
  throw error;
}
const rustInputs = [];
for (const directory of ['crates/xterm-web/src/image', 'crates/xterm-web/tests/image_canvas_support']) {
  for (const file of await readdir(resolve(root, directory))) {
    const name = `${directory}/${file}`;
    rustInputs.push({ path: name, sha256: createHash('sha256').update(await readFile(resolve(root, name))).digest('hex') });
  }
}
for (const name of ['crates/xterm-web/tests/image_canvas_browser.rs', 'Cargo.lock', 'scripts/image-canvas-check.mjs', 'scripts/browser-host-check.mjs']) {
  rustInputs.push({ path: name, sha256: createHash('sha256').update(await readFile(resolve(root, name))).digest('hex') });
}
const browsers = JSON.parse(await readFile(resolve(root, 'results/browser-host/image_canvas_browser-results.json'), 'utf8'));
await writeFile(verificationPath, JSON.stringify({
  status: browsers.browsers.every(browser => browser.status === 'passed') ? 'passed-component-only' : 'failed',
  fullImageAddonEquivalence: 'NOT PROVEN', isolatedComponent: process.argv.includes('--isolated-component'),
  seed: '0x272389', seededDrawAndExtractPairsPerBrowser: 128,
  rustInputs, sourceInputs: inputs, browserReport: 'results/browser-host/image_canvas_browser-results.json',
  remaining: ['127 terminal open/font/renderer lifecycle hooks', '38 complete addon/parser/browser integration', 'workspace build/test/lint/WASM producers and native OS matrix']
}, null, 2) + '\n');
