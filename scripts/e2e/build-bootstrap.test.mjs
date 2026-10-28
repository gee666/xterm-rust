import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, writeFile, readFile, rm } from 'node:fs/promises';
import { resolve, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { Script } from 'node:vm';
import { buildBootstrap } from './build-bootstrap.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
test('test transport installs bindings synchronously before the classic script returns', async () => {
  await mkdir(join(root, 'tmp'), { recursive: true });
  const directory = await mkdtemp(join(root, 'tmp/e2e-bootstrap-'));
  try {
    await mkdir(join(directory, 'browser/pkg'), { recursive: true });
    const constructors = ['Terminal', 'AttachAddon', 'ClipboardAddon', 'FitAddon', 'ImageAddon',
      'ProgramStatusAddon', 'ProgressAddon', 'SearchAddon', 'SerializeAddon', 'Unicode11Addon',
      'UnicodeGraphemesAddon', 'LigaturesAddon', 'WebLinksAddon', 'WebglAddon'];
    // These are transport-test exports, not a substitute terminal implementation.
    const exports = constructors.map(name => `export class ${name} {}`).join('\n');
    await writeFile(join(directory, 'browser/pkg/xterm_web.js'),
      `export function initSync({module}) { globalThis.initialized = new WebAssembly.Instance(new WebAssembly.Module(module)); }\n${exports}`);
    await writeFile(join(directory, 'browser/xterm.mjs'), "export * from './pkg/xterm_web.js';");
    await writeFile(join(directory, 'browser/pkg/xterm_web_bg.wasm'), new Uint8Array([0, 97, 115, 109, 1, 0, 0, 0]));
    const outputFile = join(directory, 'bootstrap.js');
    await buildBootstrap({ root: directory, referenceRoot: join(root, 'source_repo'), outputFile });
    const window = {};
    const context = { window, WebAssembly, Uint8Array, atob: value => Buffer.from(value, 'base64').toString('binary') };
    new Script(await readFile(outputFile, 'utf8')).runInNewContext(context);
    assert.ok(context.initialized instanceof WebAssembly.Instance);
    for (const name of constructors) assert.equal(typeof window[name], 'function', name);
    assert.equal(window.__rustE2EBackend, 'xterm-web wasm32');
  } finally { await rm(directory, { recursive: true, force: true }); }
});
