// Public source oracle. Kept independent of the frozen teardown545 producer.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { withArtifactServer } from './browser-artifact-server.mjs';
const source = resolve(process.env.SOURCE_ROOT ?? 'source_repo');
const output = resolve('results/selection-live-geometry');
const fixture = resolve('crates/xterm-web/tests/fixtures/selection-live-geometry.json');
const program = await readFile('scripts/selection-live-geometry-trace.js', 'utf8');
const generate = process.argv.includes('--generate');
assert(process.argv.slice(2).every(arg => arg === '--generate'), 'Unknown argument');
const playwright = await import(pathToFileURL(process.env.PLAYWRIGHT_MODULE ?? resolve(source, 'node_modules/playwright/index.mjs')));
const hash = value => createHash('sha256').update(value).digest('hex');
const map = JSON.parse(await readFile(resolve(source, 'lib/xterm.mjs.map'), 'utf8'));
const report = { platform: process.platform, node: process.version, status: 'running',
  scope: 'Eight custom public geometry/dedup cases, not the full original selection suite.',
  hashes: { 'scripts/selection-live-geometry-trace.js': hash(program) }, browsers: [] };
for (const file of ['lib/xterm.mjs', 'lib/xterm.mjs.map', 'src/browser/services/SelectionService.ts',
  'src/browser/selection/SelectionModel.ts', 'src/browser/CoreBrowserTerminal.ts', 'src/browser/public/Terminal.ts']) {
  const bytes = await readFile(resolve(source, file));
  report.hashes[file] = hash(bytes);
  if (file.endsWith('.ts')) {
    const index = map.sources.findIndex(name => name.endsWith(file.slice(4)));
    assert(index >= 0, `Missing embedded source: ${file}`);
    assert.equal(map.sourcesContent[index], bytes.toString('utf8'), `Stale original bundle: ${file}`);
  }
}
let expected = generate ? undefined : JSON.parse(await readFile(fixture, 'utf8'));
await mkdir(output, { recursive: true });
try {
  await withArtifactServer({ directory: resolve(source, 'lib'), css: await readFile(resolve(source, 'css/xterm.css')),
    html: '<!doctype html><link rel="stylesheet" href="/xterm.css">' }, async url => {
    for (const name of (process.env.BROWSERS ?? 'chromium,firefox,webkit').split(',')) {
      assert(['chromium', 'firefox', 'webkit'].includes(name));
      const browser = await playwright[name].launch({ headless: true, timeout: 30000 });
      const entry = { name, version: browser.version(), status: 'running', errors: [], trace: [] };
      report.browsers.push(entry);
      try {
        const page = await browser.newPage();
        page.on('pageerror', error => entry.errors.push(error.stack ?? error.message));
        await page.goto(url);
        let timer;
        try {
          entry.trace = await Promise.race([page.evaluate(async program => {
            const { Terminal } = await import('/xterm.mjs');
            const trace = Function('terminal,action,phase', program), result = [];
            for (const phase of ['mousemove', 'mouseup']) for (const action of ['shrink', 'grow', 'shrink-throw', 'grow-throw']) {
              result.push(await trace(new Terminal({ cols: 20, rows: 4 }), action, phase));
            }
            return result;
          }, program), new Promise((_, reject) => {
            timer = setTimeout(() => reject(new Error('Selection geometry matrix deadline')), 60000);
          })]);
        } finally { clearTimeout(timer); }
        assert.equal(entry.trace.length, 8);
        for (const item of entry.trace) {
          const throwing = item.action.endsWith('throw');
          const end = item.action.startsWith('shrink') ? { x: 3, y: 2 } : { x: 23, y: 0 };
          assert.equal(item.error, throwing ? 'marker' : 'none');
          assert.equal(item.initialEvents, throwing ? 0 : 1);
          assert.equal(item.events.length, 1, 'repeated identical selection emits no second event');
          assert.deepEqual(item.repeated.position, { start: { x: 8, y: 0 }, end });
          if (throwing) assert.equal(item.after.position, null);
          else assert.deepEqual(item.after, item.repeated);
        }
        if (expected === undefined) expected = entry.trace;
        assert.deepEqual(entry.trace, expected);
        assert.deepEqual(entry.errors, []);
        entry.status = 'passed';
        console.log(`${name}: eight public live selection geometry cases passed`);
      } catch (error) { entry.status = 'failed'; entry.error = error.stack ?? String(error); throw error; }
      finally { await browser.close(); }
    }
  });
  for (const [file, digest] of Object.entries(report.hashes)) {
    assert.equal(hash(await readFile(file.startsWith('scripts/') ? resolve(file) : resolve(source, file))), digest, file);
  }
  if (generate) await writeFile(fixture, '[\n' + expected.map(item => JSON.stringify(item)).join(',\n') + '\n]\n');
  report.status = 'passed';
} catch (error) { report.status = 'failed'; report.error = error.stack ?? String(error); throw error; }
finally { await writeFile(resolve(output, 'source.json'), JSON.stringify(report, null, 2) + '\n'); }
