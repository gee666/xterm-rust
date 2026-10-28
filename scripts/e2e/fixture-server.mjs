// Original test fixture layout with only the engine bootstrap replaced by Rust.
import { createServer } from 'node:http';
import { readFile, realpath, stat } from 'node:fs/promises';
import { resolve, join, relative, isAbsolute, extname } from 'node:path';

const types = { '.js': 'text/javascript', '.mjs': 'text/javascript', '.css': 'text/css',
  '.wasm': 'application/wasm', '.ttf': 'font/ttf', '.otf': 'font/otf', '.woff': 'font/woff',
  '.woff2': 'font/woff2', '.png': 'image/png', '.jpg': 'image/jpeg', '.jpeg': 'image/jpeg',
  '.gif': 'image/gif', '.webp': 'image/webp', '.json': 'application/json' };
const sourceData = new Set(['.ttf', '.otf', '.woff', '.woff2', '.png', '.jpg', '.jpeg',
  '.gif', '.webp', '.bmp', '.txt', '.json', '.iip', '.sixel', '.bin', '.blob']);

export function rustFixtureHtml(original) {
  const bundle = /<script\s+src="\/dist\/client-bundle\.js"\s+defer><\/script>/g;
  if ([...original.matchAll(bundle)].length !== 1) throw new Error('Expected the original test fixture engine script exactly once');
  return original.replace(bundle, '<script type="module" src="/rust-test-page.mjs"></script>');
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

export async function startFixtureServer({ root, packageRoot, referenceRoot = join(root, 'source_repo'), timeoutMs = 120000 }) {
  if (!Number.isSafeInteger(timeoutMs) || timeoutMs < 1) throw new Error('A bounded server lifetime is required');
  const reference = await realpath(referenceRoot);
  const browser = await realpath(join(root, 'browser'));
  const pkg = await realpath(packageRoot);
  const fixture = rustFixtureHtml(await readFile(join(reference, 'demo/test.html'), 'utf8'));
  const files = new Map([
    ['/xterm.mjs', join(browser, 'xterm.mjs')],
    ['/xterm.css', join(root, 'crates/xterm-web/assets/xterm.css')],
    ['/rust-test-page.mjs', join(root, 'scripts/e2e/test-page.mjs')]
  ]);
  const requests = [];
  const failures = [];
  const server = createServer(async (request, response) => {
    const entry = { method: request.method, url: request.url, status: 0 };
    requests.push(entry);
    try {
      if (!['GET', 'HEAD'].includes(request.method)) { entry.status = 405; response.writeHead(405).end(); return; }
      const pathname = decodeURIComponent(new URL(request.url, 'http://127.0.0.1').pathname);
      if (pathname === '/test') {
        entry.status = 200;
        response.writeHead(200, { 'Content-Type': 'text/html; charset=utf-8', 'Cache-Control': 'no-store' })
          .end(request.method === 'HEAD' ? undefined : fixture);
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
      closing = new Promise((done, reject) => {
        server.closeAllConnections(); server.close(error => error ? reject(error) : done());
      });
    }
    return closing;
  }
  timer = setTimeout(() => { close().catch(error => failures.push(String(error))); }, timeoutMs);
  const address = server.address();
  return { origin: `http://127.0.0.1:${address.port}`, requests, failures, close };
}
