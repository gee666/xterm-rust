// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Original engine baseline only. Never counts as Rust browser coverage.
import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {mkdir, mkdtemp, readFile, writeFile, rm} from 'node:fs/promises';
import {resolve, join} from 'node:path';
import {runCommand} from './quality-process.mjs';
import {configForRust, testCounts} from './e2e/upstream-config.mjs';

const root = resolve(import.meta.dirname, '..'), source = join(root, 'source_repo');
const require = createRequire(join(source, 'package.json'));
const manifest = JSON.parse(await readFile(join(root, 'docs/baseline/reference-manifest.json'), 'utf8'));
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
async function integrity() {
  for (const entry of manifest) {
    const bytes = await readFile(join(source, entry.path));
    assert.equal(bytes.length, entry.bytes, entry.path);
    assert.equal(sha(bytes), entry.sha256, entry.path);
  }
}
await integrity();
await mkdir(join(root, 'tmp'), {recursive: true});
const scratch = await mkdtemp(join(root, 'tmp/dsr-original-'));
const output = join(root, 'results/dsr-option/original-browser');
await mkdir(output, {recursive: true});
await writeFile(join(output, 'summary.json'), JSON.stringify({status: 'running', engine: 'original JavaScript'}) + '\n');
const esbuild = require('esbuild');
let server;
try {
  const bundle = join(scratch, 'source.js');
  await esbuild.build({stdin: {contents: `import {Terminal} from ${JSON.stringify(join(source, 'src/browser/public/Terminal.ts'))}; globalThis.Terminal = Terminal;`,
    resolveDir: source, sourcefile: 'dsr-original-bootstrap.ts'}, outfile: bundle, bundle: true,
    platform: 'browser', format: 'iife', tsconfig: join(source, 'src/browser/tsconfig.json'), logLevel: 'silent'});
  const files = new Map([
    ['/test', {bytes: await readFile(join(source, 'demo/test.html')), type: 'text/html'}],
    ['/xterm.css', {bytes: await readFile(join(source, 'css/xterm.css')), type: 'text/css'}],
    ['/dist/client-bundle.js', {bytes: await readFile(bundle), type: 'text/javascript'}]
  ]);
  server = createServer((request, response) => {
    const file = files.get(new URL(request.url, 'http://localhost').pathname);
    if (!file) {response.writeHead(404).end(); return;}
    response.writeHead(200, {'Content-Type': file.type, 'Cache-Control': 'no-store'}).end(file.bytes);
  });
  await new Promise((ready, reject) => {server.once('error', reject); server.listen(0, '127.0.0.1', ready);});
  const origin = `http://127.0.0.1:${server.address().port}`;
  const config = join(scratch, 'playwright.cjs');
  // This existing helper preserves original testDir/projects/assertions and only
  // replaces webServer/baseURL. Despite its name, it does not choose an engine.
  await writeFile(config, configForRust(join(source, 'test/playwright/playwright.config.ts'), origin));
  const rawFile = join(output, 'playwright.json');
  const args = [join(source, 'node_modules/@playwright/test/cli.js'), 'test', `--config=${config}`,
    '--reporter=json', '--workers=2', '--retries=0', '--global-timeout=179500',
    '--grep=DSR: Device Status Report', `--output=${join(scratch, 'artifacts')}`];
  const result = await runCommand(process.execPath, args, {cwd: source, log: join(output, 'playwright.log'),
    timeoutMs: 180000, env: {...process.env, TMP: scratch, TEMP: scratch, TMPDIR: scratch,
      CI: '1', FORCE_COLOR: '0', PLAYWRIGHT_JSON_OUTPUT_FILE: rawFile}});
  const raw = JSON.parse(await readFile(rawFile, 'utf8'));
  const counts = testCounts(raw);
  const report = {engine: 'original JavaScript browser Terminal', command: [process.execPath, ...args],
    node: process.version, platform: process.platform, counts, process: result,
    bundleSha256: sha(await readFile(bundle)), referenceFilesVerifiedBeforeAndAfter: manifest.length,
    fixtures: Object.fromEntries([...files].map(([path, {bytes}]) => [path, {bytes: bytes.length, sha256: sha(bytes)}])),
    scope: 'Six unchanged original InputHandler DSR/DECDSR assertions in three engines only. Not Rust coverage.'};
  assert.equal(result.status, 'passed');
  assert.equal(counts.discovered, 18);
  assert.equal(counts.passed, 18);
  assert.equal(counts.failed, 0);
  assert.equal(counts.skipped, 0);
  assert.equal(counts.unrun, 0);
  assert.equal(raw.errors.length, 0);
  await integrity();
  report.status = 'passed';
  await writeFile(join(output, 'summary.json'), JSON.stringify(report, null, 2) + '\n');
  console.log(JSON.stringify({engine: report.engine, counts, reference: manifest.length}));
} catch (error) {
  await writeFile(join(output, 'summary.json'), JSON.stringify({status: 'failed', engine: 'original JavaScript',
    error: String(error.stack)}, null, 2) + '\n');
  throw error;
} finally {
  esbuild.stop();
  if (server?.listening) {
    const closed = new Promise((done, reject) => server.close(error => error ? reject(error) : done()));
    server.closeAllConnections(); await closed;
  }
  await rm(scratch, {recursive: true, force: true});
}
