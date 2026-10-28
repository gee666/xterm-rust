// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Reuse the wasm-bindgen-test runner; add an ephemeral real WebSocket peer.
import { createRequire } from 'node:module';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
const require = createRequire(pathToFileURL(resolve('source_repo/package.json')));
const { WebSocketServer } = require('ws');
const server = new WebSocketServer({ host: '127.0.0.1', port: 0 });
await new Promise((done, reject) => { server.once('listening', done); server.once('error', reject); });
server.on('connection', socket => { socket.send('foo'); socket.send(Buffer.from([102, 111, 111])); });
process.env.UTILITY_SOCKET_URL = `ws://127.0.0.1:${server.address().port}`;
process.env.WASM_TEST ??= 'wasm_utility_addons';
try { await import('./browser-host-check.mjs'); }
finally {
  for (const client of server.clients) client.terminate();
  await new Promise((done, reject) => server.close(error => error ? reject(error) : done()));
}
