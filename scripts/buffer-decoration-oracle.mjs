// MIT license. Observe the unchanged upstream renderer in real browser DOM.
import { createServer } from 'node:http';
import { createHash } from 'node:crypto';
import { mkdir, readFile, writeFile, statfs } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
const root = resolve(import.meta.dirname, '..');
const generate = process.argv.includes('--generate');
if (process.argv.slice(2).some(arg => arg !== '--generate')) throw new Error('Usage: node scripts/buffer-decoration-oracle.mjs [--generate]');
const work = resolve(root, 'tmp/buffer-decoration-oracle'); const output = resolve(root, 'results/browser-decoration');
await mkdir(work, { recursive: true }); await mkdir(output, { recursive: true });
const disk = await statfs(root); if (disk.bavail * disk.bsize < 1024 ** 3) throw new Error('1GiB disk reserve required');
const { build } = await import(pathToFileURL(resolve(root, 'source_repo/node_modules/esbuild/lib/main.js')));
const built = await build({ absWorkingDir: root, stdin: { contents: `
export { BufferDecorationRenderer } from './source_repo/src/browser/decorations/BufferDecorationRenderer';
export { DecorationService } from './source_repo/src/common/services/DecorationService';
export { BufferService } from './source_repo/src/common/services/BufferService';
export { OptionsService } from './source_repo/src/common/services/OptionsService';
export { Emitter } from './source_repo/src/common/Event';`, resolveDir: root, loader: 'ts' },
  outfile: resolve(work, 'source.mjs'), bundle: true, platform: 'browser', format: 'esm', metafile: true,
  tsconfig: 'source_repo/src/browser/tsconfig.json' });
const sha = value => createHash('sha256').update(value).digest('hex'); const hashes = {};
for (const path of Object.keys(built.metafile.inputs).filter(path => path.startsWith('source_repo/src/'))) hashes[path] = sha(await readFile(resolve(root, path)));
const code = await readFile(resolve(work, 'source.mjs'));
const server = createServer((request, response) => {
  response.setHeader('Content-Type', request.url === '/source.mjs' ? 'text/javascript' : 'text/html');
  response.end(request.url === '/source.mjs' ? code : '<!doctype html><title>Original buffer decoration oracle</title>');
});
await new Promise((done, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', done); });
let browser;
try {
  const playwright = await import(pathToFileURL(resolve(root, 'source_repo/node_modules/playwright/index.mjs')));
  browser = await playwright.chromium.launch({ headless: true, timeout: 30_000 });
  const page = await browser.newPage(); page.setDefaultTimeout(60_000);
  await page.goto(`http://127.0.0.1:${server.address().port}/`);
  const snapshots = await page.evaluate(async () => {
    const { BufferDecorationRenderer, BufferService, OptionsService, DecorationService, Emitter } = await import('/source.mjs');
    const buffer = new BufferService(new OptionsService({ cols: 80, rows: 24 }), { debug() {}, info() {}, warn() {}, error() {} });
    const decorations = new DecorationService({ debug() {}, info() {}, warn() {}, error() {} }, buffer);
    const viewport = new Emitter(), dimensions = new Emitter(), dpr = new Emitter(); const callbacks = [];
    const render = { dimensions: { css: { cell: { width: 8.25, height: 16.5 } } },
      onRenderedViewportChange: viewport.event, onDimensionsChange: dimensions.event,
      addRefreshCallback(callback) { callbacks.push(callback); return callbacks.length; } };
    const screen = document.createElement('div'); document.body.appendChild(screen);
    const renderer = new BufferDecorationRenderer(screen, buffer, { mainDocument: document, onDprChange: dpr.event }, decorations, render);
    const flush = () => { for (const callback of callbacks.splice(0)) callback(0); };
    const style = decoration => {
      const element = decoration.element; if (!element) return null;
      return { className: element.className, width: element.style.width, height: element.style.height,
        top: element.style.top, lineHeight: element.style.lineHeight, left: element.style.left,
        right: element.style.right, display: element.style.display, connected: element.isConnected };
    };
    const observed = []; const snapshots = [];
    const record = (name, decoration) => snapshots.push({ name, style: style(decoration), renderEvents: observed.slice() });
    buffer.buffer.ydisp = 2;
    const decoration = decorations.registerDecoration({ marker: buffer.buffer.addMarker(5), x: 3, width: 2, height: 2, anchor: 'right', layer: 'top' });
    decoration.onRender(() => observed.push(style(decoration))); flush(); record('created', decoration);
    Object.assign(decoration.options, { width: 0, height: 0, x: 0, anchor: 'left', layer: 'bottom' });
    dimensions.fire(); flush(); record('zero-defaults', decoration);
    Object.assign(decoration.options, { width: 3, height: 3, x: 2 });
    dimensions.fire(); flush(); record('anchor-update', decoration);
    buffer.buffer.ydisp = 7; viewport.fire(); record('off-viewport', decoration);
    buffer.buffers.activateAltBuffer(); viewport.fire(); record('alternate', decoration);
    buffer.buffers.activateNormalBuffer(); buffer.buffer.ydisp = 0; viewport.fire(); record('normal', decoration);
    decoration.marker.dispose(); record('disposed-marker', decoration);
    const pending = decorations.registerDecoration({ marker: buffer.buffer.addMarker(1) });
    renderer.dispose(); flush(); record('detached-pending-frame', pending);
    decorations.dispose(); buffer.dispose(); screen.remove();
    return snapshots;
  });
  const fixture = resolve(root, 'crates/xterm-web/tests/fixtures/buffer-decoration.json'); const bytes = JSON.stringify(snapshots, null, 2) + '\n';
  let drift = false; if (generate) await writeFile(fixture, bytes); else { drift = await readFile(fixture, 'utf8') !== bytes; if (drift) await writeFile(resolve(work, 'actual.json'), bytes); }
  const report = { command: 'node scripts/buffer-decoration-oracle.mjs', mode: generate ? 'generate' : 'verify', node: process.version, platform: process.platform, browser: browser.version(), hashes,
    scenarios: snapshots.length, originalTestFunctions: 0, note: 'No upstream BufferDecorationRenderer.test.ts exists; these are actual-source DOM observations, not invented original test counts.',
    fixtureSha256: sha(await readFile(fixture)), generatedSha256: sha(bytes), fixtureDrift: drift, overallEquivalence: 'NOT PROVEN' };
  await writeFile(resolve(output, 'source.json'), JSON.stringify(report, null, 2) + '\n'); console.log(JSON.stringify(report, null, 2));
  if (drift) process.exitCode = 1;
} finally { if (browser) await browser.close(); server.closeAllConnections(); await new Promise(done => server.close(done)); }
