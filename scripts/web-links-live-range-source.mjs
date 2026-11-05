// Original opened Terminal and addon are the oracle, never the Rust engine.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { withArtifactServer } from './browser-artifact-server.mjs';

const reference = resolve(process.env.SOURCE_ROOT ?? 'source_repo');
const output = resolve('results/web-links-live-range');
const fixture = resolve('crates/xterm-web/tests/fixtures/web-links-live-range.json');
const program = await readFile('scripts/web-links-live-range-trace.js', 'utf8');
const generate = process.argv.includes('--generate');
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const hashes = {};
for (const [bundle, files] of [
  ['lib/xterm.mjs', ['src/browser/Linkifier.ts', 'src/browser/services/SelectionService.ts', 'src/browser/public/Terminal.ts']],
  ['addons/addon-web-links/lib/addon-web-links.mjs', ['addons/addon-web-links/src/WebLinksAddon.ts', 'addons/addon-web-links/src/WebLinkProvider.ts']]
]) {
  const sourceMap = JSON.parse(await readFile(resolve(reference, bundle + '.map'), 'utf8'));
  for (const file of [bundle, bundle + '.map', ...files]) {
    const bytes = await readFile(resolve(reference, file));
    hashes[file] = hash(bytes);
    if (file.endsWith('.ts')) {
      const index = sourceMap.sources.findIndex(name => name.endsWith(file.slice(file.indexOf('src/') + 4)));
      assert(index >= 0, `Bundle source map must contain ${file}`);
      assert.equal(sourceMap.sourcesContent[index], bytes.toString('utf8'), `Stale source bundle for ${file}`);
    }
  }
}
await mkdir(output, { recursive: true });
const report = { platform: process.platform, node: process.version, hashes, traceSha256: hash(program),
  scope: 'Directed utility live-range/callback trace, not full link equivalence.', browsers: [] };
const playwright = await import(pathToFileURL(process.env.PLAYWRIGHT_MODULE ?? resolve(reference, 'node_modules/playwright/index.mjs')));
let expected = generate ? undefined : JSON.parse(await readFile(fixture, 'utf8'));
try {
  await withArtifactServer({ directory: reference, css: await readFile(resolve(reference, 'css/xterm.css')),
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
          const { Terminal } = await import('/lib/xterm.mjs');
          const { WebLinksAddon } = await import('/addons/addon-web-links/lib/addon-web-links.mjs');
          const terminal = new Terminal({ cols: 20, rows: 4 });
          let timer;
          try {
            return await Promise.race([Function('terminal', 'createAddon', 'createTerminal', program)(terminal, (handler, options) => new WebLinksAddon(handler, options), () => new Terminal({ cols: 20, rows: 4 })),
              new Promise((_, reject) => { timer = setTimeout(() => reject(new Error('Utility trace deadline')), 30000); })]);
          } finally { clearTimeout(timer); terminal.dispose(); }
        }, program);
        if (expected === undefined) expected = entry.trace;
        assert.deepEqual(entry.trace, expected);
        assert.equal(entry.trace.length, 28);
        assert.deepEqual(entry.errors, []);
        entry.status = 'passed';
        console.log(`${name}: ${entry.trace.length} source utility phases passed`);
      } catch (error) { entry.status = 'failed'; entry.error = error.stack ?? String(error); throw error; }
      finally { await browser.close(); }
    }
  });
  if (generate) { assert(expected); await writeFile(fixture, JSON.stringify(expected, null, 2) + '\n'); }
} finally { await writeFile(resolve(output, 'source.json'), JSON.stringify(report, null, 2) + '\n'); }
