// Local static and optional WebSocket echo host. No terminal engine runs here.
import { createServer } from 'node:http';
import { readFile, realpath, stat } from 'node:fs/promises';
import { createRequire } from 'node:module';
import { dirname, resolve, sep, extname } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const types = { '.html': 'text/html; charset=utf-8', '.mjs': 'text/javascript; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8', '.css': 'text/css; charset=utf-8',
  '.wasm': 'application/wasm', '.json': 'application/json; charset=utf-8' };
export async function startDemoServer({ port = 3000, timeoutMs = 15 * 60 * 1000, webSocket = false } = {}) {
  if (!Number.isInteger(port) || port < 0 || port > 65535) throw new Error('port must be an integer from0 through65535');
  if (!Number.isSafeInteger(timeoutMs) || timeoutMs <= 0) throw new Error('timeoutMs must be a positive safe integer');
  const publicRoot = await realpath(resolve(root, 'browser'));
  const requests = [];
  const server = createServer(async (request, response) => {
    const entry = { method: request.method, url: request.url, status: 0 };
    requests.push(entry);
    try {
      if (!['GET', 'HEAD'].includes(request.method)) { entry.status = 405; response.writeHead(405).end(); return; }
      const pathname = new URL(request.url, 'http://127.0.0.1').pathname;
      if (pathname === '/') { entry.status = 302; response.writeHead(302, { Location: '/demo/index.html' }).end(); return; }
      const path = resolve(publicRoot, '.' + decodeURIComponent(pathname));
      if (!path.startsWith(publicRoot + sep)) { entry.status = 403; response.writeHead(403).end(); return; }
      const canonical = await realpath(path);
      if (!canonical.startsWith(publicRoot + sep)) { entry.status = 403; response.writeHead(403).end(); return; }
      const metadata = await stat(canonical);
      if (!metadata.isFile()) { entry.status = 404; response.writeHead(404).end(); return; }
      const data = await readFile(canonical);
      entry.status = 200;
      response.writeHead(200, { 'Content-Type': types[extname(canonical)] ?? 'application/octet-stream',
        'Content-Length': data.byteLength, 'Cache-Control': 'no-store' });
      response.end(request.method === 'HEAD' ? undefined : data);
    } catch (error) {
      entry.status = error.code === 'ENOENT' ? 404 : error instanceof URIError ? 400 : 500;
      response.writeHead(entry.status, { 'Content-Type': 'text/plain; charset=utf-8' }).end(String(error.message));
    }
  });
  let socketServer;
  const socketEvents = [];
  if (webSocket) {
    // Reuse the locked reference test dependency. This optional development
    // server is not part of the distributed Rust/WASM terminal library.
    const require = createRequire(pathToFileURL(resolve(root, 'source_repo/package.json')));
    const { WebSocketServer } = require('ws');
    socketServer = new WebSocketServer({ server, path: '/socket', maxPayload: 16 * 1024 * 1024, perMessageDeflate: false });
    socketServer.on('connection', peer => {
      socketEvents.push({ type: 'open' });
      peer.on('message', (data, binary) => {
        socketEvents.push({ type: 'message', binary, bytes: data.byteLength });
        peer.send(data, { binary });
      });
      peer.on('close', () => socketEvents.push({ type: 'close' }));
      peer.on('error', error => socketEvents.push({ type: 'error', message: error.message }));
    });
  }
  await new Promise((ready, reject) => { server.once('error', reject); server.listen(port, '127.0.0.1', ready); });
  let closing;
  let timer;
  function close() {
    if (closing) return closing;
    clearTimeout(timer);
    closing = new Promise((done, reject) => {
      if (socketServer) { for (const client of socketServer.clients) client.terminate(); socketServer.close(); }
      server.closeAllConnections();
      server.close(error => error ? reject(error) : done());
    });
    return closing;
  }
  timer = setTimeout(() => { close().catch(error => console.error('Demo server shutdown failed:', error)); }, timeoutMs);
  const address = server.address();
  return { origin: `http://127.0.0.1:${address.port}`, requests, socketEvents, close };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  let port = 3000;
  let timeoutMs = 15 * 60 * 1000;
  let webSocket = false;
  const args = process.argv.slice(2);
  for (let index = 0; index < args.length; index++) {
    if (args[index] === '--port') port = Number(args[++index]);
    else if (args[index] === '--timeout-ms') timeoutMs = Number(args[++index]);
    else if (args[index] === '--websocket') webSocket = true;
    else throw new Error(`Unknown server argument ${args[index]}`);
  }
  const host = await startDemoServer({ port, timeoutMs, webSocket });
  console.log(`Demo: ${host.origin}/demo/index.html`);
  if (webSocket) console.log(`Attach echo: ${host.origin.replace('http:', 'ws:')}/socket`);
  console.log(`Stops after ${timeoutMs}ms. Press Ctrl+C for immediate shutdown.`);
  async function shutdown() {
    process.off('SIGINT', shutdown); process.off('SIGTERM', shutdown);
    await host.close();
  }
  process.once('SIGINT', shutdown); process.once('SIGTERM', shutdown);
}
