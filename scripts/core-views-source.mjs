// Read-only original Terminal trace. No source assertion or implementation edits.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { withArtifactServer } from './browser-artifact-server.mjs';

const reference = resolve(process.env.SOURCE_ROOT ?? 'source_repo');
const caseName = process.env.CORE_VIEW_CASES ?? 'core-views';
const cases = {
  'core-views': { trace: 'core-view-trace', sources: ['src/common/services/CoreService.ts',
    'src/common/services/BufferService.ts', 'src/common/buffer/BufferSet.ts'] },
  'image-core': { trace: 'image-core-trace', sources: ['src/common/InputHandler.ts',
    'src/common/parser/EscapeSequenceParser.ts', 'src/common/buffer/AttributeData.ts',
    'src/common/buffer/BufferLine.ts', 'src/browser/services/ThemeService.ts',
    'src/browser/services/CoreBrowserService.ts'] }
};
const selected = cases[caseName];
assert(selected, 'Unknown core view case');
const output = resolve('results', caseName);
const fixture = resolve('crates/xterm-web/tests/fixtures', `${caseName}.json`);
const generate = process.argv.includes('--generate');
const program = await readFile(`scripts/${selected.trace}.js`, 'utf8');
await mkdir(output, { recursive: true });
const playwright = await import(pathToFileURL(process.env.PLAYWRIGHT_MODULE ?? resolve(reference, 'node_modules/playwright/index.mjs')));
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const map = JSON.parse(await readFile(resolve(reference, 'lib/xterm.mjs.map'), 'utf8'));
const hashes = {};
for (const file of ['lib/xterm.mjs', 'lib/xterm.mjs.map', ...selected.sources]) {
  const bytes = await readFile(resolve(reference, file));
  hashes[file] = hash(bytes);
  if (file.endsWith('.ts')) {
    const index = map.sources.findIndex(name => name.endsWith(file.slice(4)));
    assert(index >= 0, `Bundle source map must include ${file}`);
    assert.equal(map.sourcesContent[index], bytes.toString('utf8'), `Rebuild stale source bundle for ${file}`);
  }
}
let expected = generate ? undefined : JSON.parse(await readFile(fixture, 'utf8'));
const report = { platform: process.platform, node: process.version, hashes,
  traceSha256: hash(program), scope: 'Directed read-only service views, not full internal ABI or parity.', browsers: [] };
try {
  await withArtifactServer({ directory: resolve(reference, 'lib'),
    css: await readFile(resolve(reference, 'css/xterm.css')),
    html: '<!doctype html><link rel="stylesheet" href="/xterm.css"><div id="terminal"></div>' }, async url => {
    for (const name of (process.env.BROWSERS ?? 'chromium,firefox,webkit').split(',')) {
      assert(['chromium', 'firefox', 'webkit'].includes(name));
      const browser = await playwright[name].launch({ headless: true, timeout: 30_000 });
      const entry = { name, version: browser.version(), status: 'running', errors: [] };
      report.browsers.push(entry);
      try {
        const page = await browser.newPage();
        page.on('pageerror', error => entry.errors.push(error.message));
        await page.goto(url);
        let deadline;
        try {
          entry.trace = await Promise.race([page.evaluate(async program => {
            const { Terminal } = await import('/xterm.mjs');
            const terminal = new Terminal({ cols: 10, rows: 4, allowProposedApi: true });
            terminal.open(document.querySelector('#terminal'));
            try { return await Function('terminal', program)(terminal); }
            finally { terminal.dispose(); }
          }, program), new Promise((_, reject) => {
            deadline = setTimeout(() => reject(new Error('Core view trace exceeded 30 seconds')), 30_000);
          })]);
        } finally { clearTimeout(deadline); }
        if (expected === undefined) expected = entry.trace;
        assert.deepEqual(entry.trace, expected);
        assert.deepEqual(entry.errors, []);
        entry.status = 'passed';
        console.log(`${name}: ${entry.trace.length} core/buffer view snapshots passed`);
      } catch (error) {
        entry.status = 'failed';
        entry.error = error.stack ?? String(error);
        throw error;
      } finally { await browser.close(); }
    }
  });
  if (generate) {
    assert(expected, 'No source trace generated');
    await writeFile(fixture, JSON.stringify(expected, null, 2) + '\n');
  }
} finally {
  await writeFile(resolve(output, 'source.json'), JSON.stringify(report, null, 2) + '\n');
}
