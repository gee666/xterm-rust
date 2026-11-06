import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, writeFile, readFile, rm } from 'node:fs/promises';
import { resolve, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { Script } from 'node:vm';
import { buildBootstrap } from './build-bootstrap.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const minimalWasm = new Uint8Array([0, 97, 115, 109, 1, 0, 0, 0]);
async function fixture(body) {
  await mkdir(join(root, 'tmp'), { recursive: true });
  const directory = await mkdtemp(join(root, 'tmp/e2e-bootstrap-'));
  try {
    await mkdir(join(directory, 'browser/pkg'), { recursive: true });
    // These exports test transport ordering, not terminal behavior.
    await writeFile(join(directory, 'browser/pkg/xterm_web.js'), `
export default async function initialize({module_or_path}) {
  await globalThis.compileReady;
  globalThis.initialized = (await WebAssembly.instantiate(await module_or_path.arrayBuffer())).instance;
}
export class Terminal {}
export class FitAddon {}
`);
    await writeFile(join(directory, 'browser/xterm.mjs'), "export * from './pkg/xterm_web.js';");
    await writeFile(join(directory, 'browser/pkg/xterm_web_bg.wasm'), minimalWasm);
    const outputFile = join(directory, 'bootstrap.js');
    await buildBootstrap({ root: directory, referenceRoot: join(root, 'source_repo'), outputFile });
    await body(await readFile(outputFile, 'utf8'));
  } finally { await rm(directory, { recursive: true, force: true }); }
}

test('missing WASM fails before emitting a bootstrap', async () => {
  await mkdir(join(root, 'tmp'), { recursive: true });
  const directory = await mkdtemp(join(root, 'tmp/e2e-missing-wasm-'));
  const outputFile = join(directory, 'bootstrap.js');
  try {
    await assert.rejects(buildBootstrap({ root: directory, referenceRoot: join(root, 'source_repo'), outputFile }),
      error => error.code === 'ENOENT' && error.path.endsWith('xterm_web_bg.wasm'));
    await assert.rejects(readFile(outputFile), { code: 'ENOENT' });
  } finally { await rm(directory, { recursive: true, force: true }); }
});

function context(window, extra) {
  return { window, WebAssembly, Uint8Array, console,
    document: { currentScript: { dataset: { rustReady: 'token' } } },
    // Kept for the old transport negative control. New bundles must not use it.
    atob: value => Buffer.from(value, 'base64').toString('binary'), ...extra };
}

test('fetches exact package WASM before publishing constructors and releasing navigation', async () => fixture(async bundle => {
  let release;
  const window = {}, requests = [];
  const vm = context(window, {
    compileReady: new Promise(resolve => { release = resolve; }),
    fetch: async (url, options) => {
      requests.push({ url, options });
      if (url === '/pkg/xterm_web_bg.wasm') {
        assert.equal(window.Terminal, undefined);
        return { ok: true, arrayBuffer: async () => minimalWasm };
      }
      assert.equal(typeof window.Terminal, 'function');
      assert.ok(vm.initialized instanceof WebAssembly.Instance);
      return { ok: true };
    }
  });
  new Script(bundle).runInNewContext(vm);
  assert.equal(window.Terminal, undefined);
  assert.equal(requests.length, 1, 'The WASM request must precede initialization');
  assert.equal(requests[0].url, '/pkg/xterm_web_bg.wasm');
  release(); await window.__rustE2EReady;
  assert.ok(vm.initialized instanceof WebAssembly.Instance);
  assert.equal(typeof window.Terminal, 'function'); assert.equal(typeof window.FitAddon, 'function');
  assert.equal(window.ImageAddon, undefined);
  assert.ok(window.__rustE2EMissingBindings.includes('ImageAddon'));
  assert.equal(window.__rustE2EBackend, 'xterm-web wasm32');
  assert.equal(requests.length, 2);
  assert.equal(requests[1].options.method, 'POST');
  const notified = new URL(requests[1].url, 'http://fixture');
  assert.equal(notified.searchParams.get('token'), 'token');
  assert.ok(notified.searchParams.get('missing').split(',').includes('ImageAddon'));
  assert.equal(window.__rustE2EInitializationError, undefined);
  assert.doesNotMatch(bundle, /atob\(|Uint8Array\.from\(/);
}));

test('fetch rejection, HTTP failure and invalid WASM fail navigation without constructors', async () => fixture(async bundle => {
  for (const failure of ['fetch', 'http', 'wasm']) {
    const window = {}, requests = [], errors = [];
    const vm = context(window, {
      compileReady: Promise.resolve(),
      console: { error: error => errors.push(String(error)) },
      fetch: async (url, options) => {
        requests.push({ url, options });
        if (url === '/pkg/xterm_web_bg.wasm') {
          if (failure === 'fetch') throw new Error('WASM transport rejected');
          return { ok: failure !== 'http', status: 404, arrayBuffer: async () => new Uint8Array([1]) };
        }
        return { ok: true };
      }
    });
    new Script(bundle).runInNewContext(vm);
    await window.__rustE2EReady;
    assert.equal(window.Terminal, undefined);
    assert.equal(window.__rustE2EBackend, undefined);
    assert.match(window.__rustE2EInitializationError,
      failure === 'fetch' ? /WASM transport rejected/ : failure === 'http' ? /WASM fetch failed: 404/ : /WebAssembly|CompileError/);
    assert.deepEqual(requests.map(request => request.url), ['/pkg/xterm_web_bg.wasm', '/rust-ready?token=token&failed=1']);
    assert.equal(requests[1].options.method, 'POST');
    assert.equal(errors.length, 1);
  }
}));
