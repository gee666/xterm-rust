import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdir, mkdtemp, writeFile, rm, symlink } from 'node:fs/promises';
import { resolve, join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { rustFixtureHtml, startFixtureServer } from './fixture-server.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const original = '<html><head><link rel="stylesheet" href="/xterm.css"></head><body><div id="terminal-container"></div><script src="/dist/client-bundle.js" defer></script></body></html>';
async function fixture(callback) {
  await mkdir(join(root, 'tmp'), { recursive: true });
  const directory = await mkdtemp(join(root, 'tmp/e2e-server-'));
  const files = { 'reference/demo/test.html': original, 'reference/demo/fonts/demo.ttf': 'font',
    'reference/addons/addon-image/fixture/image.png': 'image',
    'browser/xterm.mjs': 'export class Terminal {}', 'browser/demo/app.mjs': 'demo',
    'crates/xterm-web/assets/xterm.css': '.xterm{}', 'scripts/e2e/test-page.mjs': 'rust-only',
    'pkg/xterm_web.js': 'Rust binding', 'pkg/xterm_web_bg.wasm': 'wasm', 'private.txt': 'not public' };
  for (const [name, text] of Object.entries(files)) {
    const file = join(directory, name);
    await mkdir(dirname(file), { recursive: true }); await writeFile(file, text);
  }
  let server;
  try {
    server = await startFixtureServer({ root: directory, packageRoot: join(directory, 'pkg'),
      referenceRoot: join(directory, 'reference'), timeoutMs: 10000 });
    await callback({ directory, server });
  } finally { if (server) await server.close(); await rm(directory, { recursive: true, force: true }); }
}

test('changes only the original fixture engine script', () => {
  const replacement = '<script type="module" src="/rust-test-page.mjs"></script>';
  assert.equal(rustFixtureHtml(original), original.replace('<script src="/dist/client-bundle.js" defer></script>', replacement));
  assert.throws(() => rustFixtureHtml('<html></html>'), /exactly once/);
  assert.throws(() => rustFixtureHtml(original + original), /exactly once/);
});
test('serves Rust module, binary, canonical CSS, fonts and original image data', async () => fixture(async ({ server }) => {
  for (const [route, body, type] of [
    ['/pkg/xterm_web_bg.wasm', 'wasm', 'application/wasm'], ['/xterm.mjs', 'export class Terminal {}', 'text/javascript'],
    ['/xterm.css', '.xterm{}', 'text/css'], ['/fonts/demo.ttf', 'font', 'font/ttf'],
    ['/addons/addon-image/fixture/image.png', 'image', 'image/png']
  ]) {
    const response = await fetch(server.origin + route);
    assert.equal(response.status, 200, route); assert.equal(response.headers.get('content-type'), type);
    assert.equal(await response.text(), body);
  }
  assert.equal(await (await fetch(server.origin + '/test')).text(), rustFixtureHtml(original));
  const head = await fetch(server.origin + '/pkg/xterm_web_bg.wasm', { method: 'HEAD' });
  assert.equal(head.status, 200); assert.equal(head.headers.get('content-length'), '4'); assert.equal(await head.text(), '');
}));
test('does not serve the reference engine, arbitrary project files or source code', async () => fixture(async ({ server }) => {
  for (const route of ['/dist/client-bundle.js', '/source_repo/lib/xterm.js', '/private.txt',
    '/addons/addon-image/fixture/engine.js', '/fonts/engine.js', '/pkg/missing.js']) {
    assert.equal((await fetch(server.origin + route)).status, 404, route);
  }
  assert.equal((await fetch(server.origin + '/test', { method: 'POST' })).status, 405);
  assert.equal((await fetch(server.origin + '/pkg/%ZZ')).status, 400);
}));
test('rejects a package symlink that escapes its declared asset root', async () => fixture(async ({ directory, server }) => {
  await mkdir(join(directory, 'pkg/escape'));
  await symlink(join(directory), join(directory, 'pkg/escape/root'), process.platform === 'win32' ? 'junction' : 'dir');
  assert.equal((await fetch(server.origin + '/pkg/escape/root/private.txt')).status, 403);
  assert.equal(server.failures.length, 1);
}));
test('closing the server is idempotent and disconnects the listener', async () => fixture(async ({ server }) => {
  await server.close(); await server.close();
  await assert.rejects(fetch(server.origin + '/test'));
}));
test('refuses unbounded or invalid server lifetime', async () => {
  for (const timeoutMs of [0, -1, Infinity, NaN]) {
    await assert.rejects(startFixtureServer({ root, packageRoot: root, timeoutMs }), /bounded server lifetime/);
  }
});
