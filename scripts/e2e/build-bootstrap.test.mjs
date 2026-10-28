import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, writeFile, readFile, rm } from 'node:fs/promises';
import { resolve, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { Script } from 'node:vm';
import { buildBootstrap } from './build-bootstrap.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
test('transport releases navigation only after async compilation and real binding publication', async () => {
  await mkdir(join(root, 'tmp'), { recursive: true });
  const directory = await mkdtemp(join(root, 'tmp/e2e-bootstrap-'));
  try {
    await mkdir(join(directory, 'browser/pkg'), { recursive: true });
    // These are transport-test exports, not terminal behavior evidence.
    await writeFile(join(directory, 'browser/pkg/xterm_web.js'), `
export default async function initialize({module_or_path}) {
  await globalThis.compileReady;
  globalThis.initialized = (await WebAssembly.instantiate(module_or_path)).instance;
}
export class Terminal {}
export class FitAddon {}
`);
    await writeFile(join(directory, 'browser/xterm.mjs'), "export * from './pkg/xterm_web.js';");
    await writeFile(join(directory, 'browser/pkg/xterm_web_bg.wasm'), new Uint8Array([0, 97, 115, 109, 1, 0, 0, 0]));
    const outputFile = join(directory, 'bootstrap.js');
    await buildBootstrap({ root: directory, referenceRoot: join(root, 'source_repo'), outputFile });
    let release;
    const window = {}, requests = [];
    const context = { window, WebAssembly, Uint8Array, console,
      document: { currentScript: { dataset: { rustReady: 'token' } } },
      compileReady: new Promise(resolve => { release = resolve; }),
      atob: value => Buffer.from(value, 'base64').toString('binary'),
      fetch: async (url, options) => {
        assert.equal(typeof window.Terminal, 'function');
        requests.push({ url, options }); return { ok: true };
      } };
    new Script(await readFile(outputFile, 'utf8')).runInNewContext(context);
    assert.equal(window.Terminal, undefined); assert.equal(requests.length, 0);
    release(); await window.__rustE2EReady;
    assert.ok(context.initialized instanceof WebAssembly.Instance);
    assert.equal(typeof window.Terminal, 'function'); assert.equal(typeof window.FitAddon, 'function');
    assert.equal(window.ImageAddon, undefined);
    assert.ok(window.__rustE2EMissingBindings.includes('ImageAddon'));
    assert.equal(window.__rustE2EBackend, 'xterm-web wasm32');
    assert.equal(requests.length, 1);
    assert.equal(requests[0].options.method, 'POST');
    const notified = new URL(requests[0].url, 'http://fixture');
    assert.equal(notified.searchParams.get('token'), 'token');
    assert.ok(notified.searchParams.get('missing').split(',').includes('ImageAddon'));
    const failedRequests = [], failureWindow = {}, errors = [];
    const failedContext = { ...context, window: failureWindow,
      compileReady: Promise.reject(new Error('broken WASM module')),
      console: { error: error => errors.push(String(error)) },
      fetch: async url => { failedRequests.push(url); return { ok: true }; } };
    new Script(await readFile(outputFile, 'utf8')).runInNewContext(failedContext);
    await failureWindow.__rustE2EReady;
    assert.equal(failureWindow.Terminal, undefined);
    assert.match(failureWindow.__rustE2EInitializationError, /broken WASM module/);
    assert.deepEqual(failedRequests, ['/rust-ready?token=token&failed=1']);
    assert.equal(errors.length, 1);
  } finally { await rm(directory, { recursive: true, force: true }); }
});
