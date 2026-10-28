import { createServer } from 'node:http';
import { readFile, lstat, realpath } from 'node:fs/promises';
import { resolve, sep } from 'node:path';

/** Serve one pinned generated bundle and always close the owned socket. */
export async function withArtifactServer({ directory, css, html, port = 0 }, use) {
  const root = resolve(directory);
  const canonical = await realpath(root);
  const server = createServer(async (request, response) => {
    try {
      const pathname = new URL(request.url, 'http://localhost').pathname;
      if (pathname === '/') { response.setHeader('Content-Type', 'text/html; charset=utf-8'); response.end(html); return; }
      if (pathname === '/xterm.css') { response.setHeader('Content-Type', 'text/css; charset=utf-8'); response.end(css); return; }
      const path = resolve(root, '.' + decodeURIComponent(pathname));
      if (!path.startsWith(root + sep)) { response.writeHead(403).end(); return; }
      const info = await lstat(path);
      if (!info.isFile() || info.isSymbolicLink() || !(await realpath(path)).startsWith(canonical + sep)) {
        response.writeHead(403).end(); return;
      }
      const bytes = await readFile(path);
      response.setHeader('Content-Type', path.endsWith('.wasm') ? 'application/wasm' : 'text/javascript');
      response.end(bytes);
    } catch (error) { response.writeHead(error.code === 'ENOENT' ? 404 : 500).end(error.message); }
  });
  let failure;
  try {
    await new Promise((ready, reject) => { server.once('error', reject); server.listen(port, '127.0.0.1', ready); });
    return await use(`http://127.0.0.1:${server.address().port}/`);
  } catch (error) {
    failure = error;
    throw error;
  } finally {
    if (server.listening) {
      try { await new Promise((done, reject) => server.close(error => error ? reject(error) : done())); }
      catch (cleanup) { throw failure ? new AggregateError([failure, cleanup], 'Browser server execution and cleanup failed') : cleanup; }
    }
  }
}
