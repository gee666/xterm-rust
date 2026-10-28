import test from 'node:test';
import assert from 'node:assert/strict';
import { get } from 'node:http';
import { mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { withArtifactServer } from './browser-artifact-server.mjs';

async function fixture(use) {
  await mkdir(resolve('tmp'), { recursive: true });
  const directory = await mkdtemp(resolve('tmp/browser-server-test-'));
  try { return await use(directory); } finally { await rm(directory, { recursive: true, force: true }); }
}
function request(url) {
  return new Promise((done, reject) => {
    const call = get(url, { headers: { connection: 'close' } }, response => {
      const chunks = [];
      response.on('data', chunk => chunks.push(chunk));
      response.on('end', () => done({ status: response.statusCode, headers: response.headers, body: Buffer.concat(chunks).toString() }));
      response.on('error', reject);
    });
    call.on('error', reject);
  });
}

test('serves pinned bundle and CSS, refusing missing and escaping files', { timeout: 5000 }, async () => fixture(async directory => {
  await writeFile(resolve(directory, 'suite.js'), 'export const value = 1;');
  await writeFile(resolve(directory, 'suite_bg.wasm'), 'wasm fixture');
  await withArtifactServer({ directory, css: '.xterm{}', html: '<pre id="output"></pre>' }, async url => {
    assert.equal((await request(url)).status, 200);
    assert.equal((await request(new URL('xterm.css', url))).body, '.xterm{}');
    const wasm = await request(new URL('suite_bg.wasm', url));
    assert.equal(wasm.headers['content-type'], 'application/wasm');
    assert.equal(wasm.body, 'wasm fixture');
    assert.equal((await request(new URL('missing.js', url))).status, 404);
    assert.equal((await request(new URL('%2e%2e%2foutside', url))).status, 403);
  });
}));

test('closes the owned socket when setup inside the bound server throws', { timeout: 5000 }, async () => fixture(async directory => {
  let address;
  const original = new Error('artifact read failed after listen');
  await assert.rejects(withArtifactServer({ directory, css: '', html: '' }, async url => {
    address = url;
    assert.equal((await request(url)).status, 200);
    throw original;
  }), error => error === original);
  await assert.rejects(request(address), /ECONNREFUSED/);
}));

test('invalid listen configuration never enters the body', { timeout: 5000 }, async () => fixture(async directory => {
  let entered = false;
  await assert.rejects(withArtifactServer({ directory, css: '', html: '', port: -1 }, async () => { entered = true; }), /port/i);
  assert.equal(entered, false);
}));
