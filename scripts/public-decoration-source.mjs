// Shared public API assertions run against the unchanged original Terminal.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { withArtifactServer } from './browser-artifact-server.mjs';
const reference = resolve(process.env.SOURCE_ROOT ?? 'source_repo');
const program = await readFile('scripts/public-decoration-trace.js', 'utf8');
const output = resolve('results/public-decoration');
const fixture = resolve('crates/xterm-web/tests/fixtures/public-decoration.json');
const generate = process.argv.includes('--generate');
const hash = value => createHash('sha256').update(value).digest('hex');
const hashes = {};
const map = JSON.parse(await readFile(resolve(reference, 'lib/xterm.mjs.map'), 'utf8'));
for (const file of ['lib/xterm.mjs', 'lib/xterm.mjs.map', 'src/browser/public/Terminal.ts',
  'src/common/services/DecorationService.ts', 'src/browser/decorations/BufferDecorationRenderer.ts',
  'src/browser/decorations/ColorZoneStore.ts', 'src/browser/decorations/OverviewRulerRenderer.ts']) {
  const bytes = await readFile(resolve(reference, file));
  hashes[file] = hash(bytes);
  if (file.endsWith('.ts')) {
    const index = map.sources.findIndex(value => value.endsWith(file.slice(4)));
    assert(index >= 0, `Missing source map input ${file}`);
    assert.equal(map.sourcesContent[index], bytes.toString('utf8'), `Stale bundle ${file}`);
  }
}
await mkdir(output, { recursive: true });
const report = { platform: process.platform, node: process.version, hashes, traceSha256: hash(program),
  scope: 'Public decoration milestone only. Full331 raw markers, arrays and decoration parity remain open.', browsers: [] };
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
          const other = new Terminal({ cols: 20, rows: 4 });
          const deadline = AbortSignal.timeout(30000);
          return await Promise.race([Function('terminal', 'other', program)(terminal, other),
            new Promise((_, reject) => deadline.addEventListener('abort', () => reject(new Error('Public decoration deadline')), { once: true }))]);
        }, program);
        assert.equal(entry.trace.length, 14);
        if (expected === undefined) expected = entry.trace;
        assert.deepEqual(entry.trace, expected);
        assert.deepEqual(entry.errors, []);
        entry.status = 'passed';
        console.log(`${name}: ${entry.trace.length} public decoration phases passed`);
      } catch (error) { entry.status = 'failed'; entry.error = error.stack ?? String(error); throw error; }
      finally { await browser.close(); }
    }
  });
  if (generate) await writeFile(fixture, JSON.stringify(expected, null, 2) + '\n');
} finally { await writeFile(resolve(output, 'source.json'), JSON.stringify(report, null, 2) + '\n'); }
