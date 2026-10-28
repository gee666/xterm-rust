import { createServer } from 'node:http';
import { readFile, realpath } from 'node:fs/promises';
import { resolve, sep } from 'node:path';

export async function startCaptureServer(root, work) {
  const requests = [];
  const files = new Map([
    ['/reference.js', resolve(work, 'reference.js')],
    ['/reference.css', resolve(root, 'source_repo/css/xterm.css')],
    ['/xterm.css', resolve(root, 'crates/xterm-web/assets/xterm.css')],
    ['/xterm.mjs', resolve(root, 'browser/xterm.mjs')],
    ['/session.mjs', resolve(root, 'scripts/browser-capture/session.mjs')],
    ['/source-snapshot.mjs', resolve(root, 'scripts/browser-capture/source-snapshot.mjs')],
  ]);
  const packageRoot = await realpath(resolve(work, 'pkg'));
  const server = createServer(async (request, response) => {
    const pathname = new URL(request.url, 'http://127.0.0.1').pathname;
    const record = { pathname, status: 0 }; requests.push(record);
    try {
      if (pathname === '/reference.html' || pathname === '/rust.html') {
        const reference = pathname === '/reference.html';
        const html = `<!doctype html><html><head><meta charset="utf-8"><link rel="icon" href="data:,"><link rel="stylesheet" href="/${reference ? 'reference' : 'xterm'}.css"><style>html,body{margin:0;background:white}#terminal{position:absolute;left:16px;top:16px;width:700px;height:300px}</style></head><body><div id="terminal"></div>${reference ? '<script src="/reference.js"></script>' : ''}<script type="module" src="/session.mjs"></script></body></html>`;
        record.status = 200;
        response.writeHead(200, { 'Content-Type': 'text/html; charset=utf-8', 'Cache-Control': 'no-store' }).end(html);
        return;
      }
      let file = files.get(pathname);
      if (!file && pathname.startsWith('/pkg/')) {
        const candidate = resolve(packageRoot, decodeURIComponent(pathname.slice('/pkg/'.length)));
        if (!candidate.startsWith(packageRoot + sep)) throw new Error('Package path escapes its root');
        const canonical = await realpath(candidate);
        if (!canonical.startsWith(packageRoot + sep)) throw new Error('Package symlink escapes its root');
        file = canonical;
      }
      if (!file) { record.status = 404; response.writeHead(404).end('No capture asset'); return; }
      const bytes = await readFile(file);
      record.status = 200;
      const type = file.endsWith('.wasm') ? 'application/wasm' : file.endsWith('.css') ? 'text/css' : 'text/javascript';
      response.writeHead(200, { 'Content-Type': type, 'Cache-Control': 'no-store' }).end(bytes);
    } catch (error) {
      record.status = error.code === 'ENOENT' ? 404 : 500;
      response.writeHead(record.status).end(String(error.message));
    }
  });
  await new Promise((ready, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', ready); });
  const address = server.address();
  let closing;
  return { origin: `http://127.0.0.1:${address.port}`, requests,
    close() {
      if (!closing) closing = new Promise((done, reject) => {
        server.closeAllConnections(); server.close(error => error ? reject(error) : done());
      });
      return closing;
    } };
}
