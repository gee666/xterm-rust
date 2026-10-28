// Run unchanged upstream search assertions against the built Rust browser package.
import fs from 'node:fs';
import path from 'node:path';
import { createServer } from 'node:http';
import { fileURLToPath } from 'node:url';
import { runCommand } from './quality-process.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const source = path.join(root, 'source_repo');
const pkg = path.join(root, 'browser/pkg');
const originalConfig = path.join(source, 'addons/addon-search/out-esbuild-test/playwright.config.js');
const wasm = path.join(pkg, 'xterm_web_bg.wasm');
for (const file of [wasm, path.join(pkg, 'xterm_web.js'), originalConfig]) {
  if (!fs.existsSync(file)) throw new Error(`Build prerequisite is missing: ${file}`);
}
const work = path.join(root, 'tmp/search-browser');
const results = path.join(root, 'results/search-browser');
fs.mkdirSync(work, { recursive: true }); fs.mkdirSync(results, { recursive: true });
const binary = fs.readFileSync(wasm).toString('base64');
// Synchronous WASM initialization makes globals ready at the original helper's
// page-load boundary. No original JS terminal/addon implementation is loaded.
const html = `<!doctype html><link rel="stylesheet" href="/browser/xterm.css"><div id="terminal-container"></div>
<script type="module">
import * as api from '/browser/xterm.mjs';
import {initSync} from '/browser/pkg/xterm_web.js';
initSync({module:Uint8Array.from(atob('${binary}'),c=>c.charCodeAt(0))});
for(const name of ['Terminal','SearchAddon','UnicodeGraphemesAddon']) {
 if(typeof api[name]!=='function')throw Error('Missing Rust export '+name);
 window[name]=api[name];
}
</script>`;
const server = createServer((request, response) => {
  try {
    const route = new URL(request.url, 'http://localhost').pathname;
    if (route === '/test') { response.setHeader('Content-Type', 'text/html'); response.end(html); return; }
    if (route === '/favicon.ico') { response.writeHead(204).end(); return; }
    if (!route.startsWith('/browser/')) { response.writeHead(404).end(); return; }
    const file = path.resolve(root, '.' + decodeURIComponent(route));
    const browser = path.join(root, 'browser') + path.sep;
    if (!file.startsWith(browser)) { response.writeHead(403).end(); return; }
    response.setHeader('Content-Type', file.endsWith('.wasm') ? 'application/wasm' : file.endsWith('.css') ? 'text/css' : 'text/javascript');
    response.end(fs.readFileSync(file));
  } catch (error) { response.writeHead(error.code === 'ENOENT' ? 404 : 500).end(error.message); }
});
await new Promise((resolve, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', resolve); });
try {
  const port = server.address().port;
  const config = path.join(work, 'playwright.config.cjs');
  fs.writeFileSync(config, `const loaded=require(${JSON.stringify(originalConfig)});const c=loaded.default??loaded;
module.exports={...c,testDir:${JSON.stringify(path.dirname(originalConfig))},webServer:undefined,use:{...c.use,baseURL:'http://127.0.0.1:${port}'}};\n`);
  const raw = path.join(results, 'playwright.json');
  const args = [path.join(source, 'node_modules/@playwright/test/cli.js'), 'test', '--config', config,
    '--workers=1', '--retries=0', '--reporter=json', '--output', path.join(work, 'artifacts')];
  if (process.env.BROWSER_PROJECT) args.push('--project', process.env.BROWSER_PROJECT);
  const result = await runCommand(process.execPath, args, { cwd: root,
    env: { ...process.env, PLAYWRIGHT_JSON_OUTPUT_FILE: raw }, timeoutMs: 600_000,
    log: path.join(results, 'run.log') });
  const report = fs.existsSync(raw) ? JSON.parse(fs.readFileSync(raw, 'utf8')) : null;
  console.log(JSON.stringify({ implementation: 'Rust/WASM, unchanged original search assertions', status: result.status, stats: report?.stats ?? null }));
  if (result.status !== 'passed') process.exitCode = 1;
} finally { await new Promise((resolve, reject) => server.close(error => error ? reject(error) : resolve())); }
