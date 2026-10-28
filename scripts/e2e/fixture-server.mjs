// Original test fixture layout with only the engine bootstrap replaced by Rust.
import { createServer } from 'node:http';
import { randomUUID } from 'node:crypto';
import { readFile, realpath, stat } from 'node:fs/promises';
import { resolve, join, relative, isAbsolute, extname } from 'node:path';

const types = { '.js': 'text/javascript', '.mjs': 'text/javascript', '.css': 'text/css',
  '.wasm': 'application/wasm', '.ttf': 'font/ttf', '.otf': 'font/otf', '.woff': 'font/woff',
  '.woff2': 'font/woff2', '.png': 'image/png', '.jpg': 'image/jpeg', '.jpeg': 'image/jpeg',
  '.gif': 'image/gif', '.webp': 'image/webp', '.json': 'application/json' };
const sourceData = new Set(['.ttf', '.otf', '.woff', '.woff2', '.png', '.jpg', '.jpeg',
  '.gif', '.webp', '.bmp', '.txt', '.json', '.iip', '.sixel', '.bin', '.blob']);

export function rustFixtureHtml(original, token) {
  if (!/^[a-f0-9-]{36}$/.test(token)) throw new Error('Invalid fixture readiness token');
  const bundle = /<script\s+src="\/dist\/client-bundle\.js"\s+defer><\/script>/g;
  if ([...original.matchAll(bundle)].length !== 1) throw new Error('Expected the original test fixture engine script exactly once');
  return original.replace(bundle, `<img src="/rust-ready?token=${token}" hidden alt=""><script src="/rust-test-page.js" data-rust-ready="${token}" defer></script>`);
}

function inside(root, file) {
  const path = relative(root, file);
  return path && path !== '..' && !path.startsWith('../') && !path.startsWith('..\\') && !isAbsolute(path);
}
async function fromDirectory(root, name) {
  const file = await realpath(resolve(root, name));
  if (!inside(root, file) || !(await stat(file)).isFile()) throw new Error('Asset is outside its declared root');
  return { file, bytes: await readFile(file) };
}

export async function startFixtureServer({ root, packageRoot, bootstrapFile, referenceRoot = join(root, 'source_repo'), timeoutMs = 120000 }) {
  if (!Number.isSafeInteger(timeoutMs) || timeoutMs < 1) throw new Error('A bounded server lifetime is required');
  if (typeof bootstrapFile !== 'string') throw new Error('A compiled Rust bootstrap file is required');
  await readFile(bootstrapFile);
  const reference = await realpath(referenceRoot);
  const browser = await realpath(join(root, 'browser'));
  const pkg = await realpath(packageRoot);
  const original = await readFile(join(reference, 'demo/test.html'), 'utf8');
  const gates = new Map();
  const pixel = Buffer.from('R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7', 'base64');
  function finishGate(token, gate) {
    if (gate.state === 'pending' || !gate.response) return;
    clearTimeout(gate.timer);
    gate.entry.status = gate.state === 'ready' ? 200 : 500;
    gate.response.writeHead(gate.entry.status, { 'Content-Type': 'image/gif', 'Cache-Control': 'no-store' }).end(pixel);
    gates.delete(token);
  }
  const files = new Map([
    ['/xterm.mjs', join(browser, 'xterm.mjs')],
    ['/xterm.css', join(root, 'crates/xterm-web/assets/xterm.css')],
    ['/rust-test-page.js', bootstrapFile]
  ]);
  const requests = [];
  const failures = [];
  const runtimes = [];
  const server = createServer(async (request, response) => {
    const entry = { method: request.method, url: request.url, status: 0 };
    requests.push(entry);
    try {
      const url = new URL(request.url, 'http://127.0.0.1');
      const pathname = decodeURIComponent(url.pathname);
      if (pathname === '/rust-ready') {
        const token = url.searchParams.get('token'), gate = gates.get(token);
        if (!gate) { entry.status = 404; response.writeHead(404).end('Unknown readiness token'); return; }
        if (request.method === 'POST') {
          gate.state = url.searchParams.has('failed') ? 'failed' : 'ready';
          if (gate.state === 'failed') failures.push('Rust bootstrap initialization failed');
          runtimes.push({ state: gate.state, missingBindings: (url.searchParams.get('missing') ?? '').split(',').filter(Boolean) });
          finishGate(token, gate);
          entry.status = 204; response.writeHead(204).end(); return;
        }
        if (request.method === 'GET' && !gate.response) {
          gate.response = response; gate.entry = entry; finishGate(token, gate); return;
        }
        entry.status = 405; response.writeHead(405).end(); return;
      }
      if (!['GET', 'HEAD'].includes(request.method)) { entry.status = 405; response.writeHead(405).end(); return; }
      if (pathname === '/test') {
        if (gates.size >= 64) throw new Error('Too many pending Rust fixture navigations');
        const token = randomUUID();
        const gate = { state: 'pending', response: undefined, timer: undefined };
        gate.timer = setTimeout(() => {
          failures.push('Rust fixture initialization timed out'); gate.state = 'failed';
          finishGate(token, gate); gates.delete(token);
        }, Math.min(timeoutMs, 30000));
        gates.set(token, gate);
        entry.status = 200;
        response.writeHead(200, { 'Content-Type': 'text/html; charset=utf-8', 'Cache-Control': 'no-store' })
          .end(request.method === 'HEAD' ? undefined : rustFixtureHtml(original, token));
        return;
      }
      let asset;
      if (files.has(pathname)) {
        const file = files.get(pathname);
        asset = { file, bytes: await readFile(file) };
      } else if (pathname.startsWith('/pkg/')) asset = await fromDirectory(pkg, pathname.slice(5));
      else if (pathname.startsWith('/demo/')) asset = await fromDirectory(await realpath(join(browser, 'demo')), pathname.slice(6));
      else if (pathname.startsWith('/fonts/') && sourceData.has(extname(pathname))) {
        asset = await fromDirectory(await realpath(join(reference, 'demo/fonts')), pathname.slice(7));
      } else {
        const match = pathname.match(/^\/addons\/(addon-[a-z0-9-]+)\/fixture\/(.+)$/);
        if (match && sourceData.has(extname(pathname))) {
          asset = await fromDirectory(await realpath(join(reference, 'addons', match[1], 'fixture')), match[2]);
        }
      }
      if (!asset) { entry.status = 404; response.writeHead(404).end('No Rust E2E asset'); return; }
      entry.status = 200;
      response.writeHead(200, { 'Content-Type': types[extname(asset.file)] ?? 'application/octet-stream',
        'Content-Length': asset.bytes.length, 'Cache-Control': 'no-store' });
      response.end(request.method === 'HEAD' ? undefined : asset.bytes);
    } catch (error) {
      entry.status = error.code === 'ENOENT' ? 404 : error instanceof URIError ? 400 : 403;
      if (entry.status === 403) failures.push(String(error));
      response.writeHead(entry.status).end(String(error.message));
    }
  });
  await new Promise((ready, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', ready); });
  let closing;
  let timer;
  function close() {
    if (!closing) {
      clearTimeout(timer);
      for (const gate of gates.values()) clearTimeout(gate.timer);
      gates.clear();
      closing = new Promise((done, reject) => {
        server.closeAllConnections(); server.close(error => error ? reject(error) : done());
      });
    }
    return closing;
  }
  timer = setTimeout(() => { close().catch(error => failures.push(String(error))); }, timeoutMs);
  const address = server.address();
  return { origin: `http://127.0.0.1:${address.port}`, requests, failures, runtimes, close };
}
