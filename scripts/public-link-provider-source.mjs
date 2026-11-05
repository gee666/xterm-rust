// Original opened Terminal is the oracle for public custom-provider behavior.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { withArtifactServer } from './browser-artifact-server.mjs';

const reference = resolve(process.env.SOURCE_ROOT ?? 'source_repo');
const output = resolve('results/public-link-provider');
const fixture = resolve('crates/xterm-web/tests/fixtures/public-link-provider.json');
const program = await readFile('scripts/public-link-provider-trace.js', 'utf8');
const generate = process.argv.includes('--generate');
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const sourceMap = JSON.parse(await readFile(resolve(reference, 'lib/xterm.mjs.map'), 'utf8'));
const hashes = {};
for (const file of ['lib/xterm.mjs', 'lib/xterm.mjs.map', 'src/browser/Linkifier.ts',
  'src/browser/services/LinkProviderService.ts', 'src/browser/public/Terminal.ts']) {
  const bytes = await readFile(resolve(reference, file));
  hashes[file] = hash(bytes);
  if (file.endsWith('.ts')) {
    const index = sourceMap.sources.findIndex(name => name.endsWith(file.slice(4)));
    assert(index >= 0, `Bundle source map must contain ${file}`);
    assert.equal(sourceMap.sourcesContent[index], bytes.toString('utf8'), `Stale source bundle for ${file}`);
  }
}
await mkdir(output, { recursive: true });
const report = { platform: process.platform, node: process.version, hashes, traceSha256: hash(program),
  scope: 'Directed public custom link-provider trace, not full link equivalence.', browsers: [] };
const playwright = await import(pathToFileURL(process.env.PLAYWRIGHT_MODULE ?? resolve(reference, 'node_modules/playwright/index.mjs')));
let expected = generate ? undefined : JSON.parse(await readFile(fixture, 'utf8'));
try {
  await withArtifactServer({ directory: resolve(reference, 'lib'), css: await readFile(resolve(reference, 'css/xterm.css')),
    html: '<!doctype html><link rel="stylesheet" href="/xterm.css">' }, async url => {
    for (const name of (process.env.BROWSERS ?? 'chromium,firefox,webkit').split(',')) {
      assert(['chromium', 'firefox', 'webkit'].includes(name));
      const browser = await playwright[name].launch({ headless: true, timeout: 30000 });
      const entry = { name, version: browser.version(), status: 'running', errors: [] };
      report.browsers.push(entry);
      try {
        const page = await browser.newPage();
        page.on('pageerror', error => entry.errors.push(error.message));
        await page.goto(url);
        entry.trace = await page.evaluate(async program => {
          const { Terminal } = await import('/xterm.mjs');
          const terminal = new Terminal({ cols: 20, rows: 4 });
          const deadline = AbortSignal.timeout(30000);
          return await Promise.race([Function('terminal', program)(terminal), new Promise((_, reject) => {
            deadline.addEventListener('abort', () => reject(new Error('Public provider trace deadline')), { once: true });
          })]);
        }, program);
        if (expected === undefined) expected = entry.trace;
        assert.deepEqual(entry.trace, expected);
        assert.equal(entry.trace.length, 20);
        assert.deepEqual(entry.errors, []);
        entry.status = 'passed';
        console.log(`${name}: ${entry.trace.length} source provider phases passed`);
      } catch (error) { entry.status = 'failed'; entry.error = error.stack ?? String(error); throw error; }
      finally { await browser.close(); }
    }
  });
  if (generate) { assert(expected); await writeFile(fixture, JSON.stringify(expected, null, 2) + '\n'); }
} finally { await writeFile(resolve(output, 'source.json'), JSON.stringify(report, null, 2) + '\n'); }
