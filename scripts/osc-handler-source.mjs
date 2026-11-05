// Original production Terminal supplies OSC8 parsing, options and link discovery.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { withArtifactServer } from './browser-artifact-server.mjs';

const reference = resolve(process.env.SOURCE_ROOT ?? 'source_repo');
const output = resolve('results/osc-handler');
const fixture = resolve('crates/xterm-web/tests/fixtures/osc-handler.json');
const generate = process.argv.includes('--generate');
const program = await readFile('scripts/osc-handler-trace.js', 'utf8');
await mkdir(output, { recursive: true });
const playwright = await import(pathToFileURL(process.env.PLAYWRIGHT_MODULE ?? resolve(reference, 'node_modules/playwright/index.mjs')));
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const map = JSON.parse(await readFile(resolve(reference, 'lib/xterm.mjs.map'), 'utf8'));
const hashes = {};
for (const file of ['lib/xterm.mjs', 'lib/xterm.mjs.map', 'src/browser/OscLinkProvider.ts',
  'src/common/services/OptionsService.ts']) {
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
  traceSha256: hash(program), scope: 'Directed primitive OSC handler callbacks, not full links parity.', browsers: [] };
try {
  await withArtifactServer({ directory: resolve(reference, 'lib'),
    css: await readFile(resolve(reference, 'css/xterm.css')),
    html: '<!doctype html><div id="terminal"></div>' }, async url => {
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
            const probe = async (handler, uri, actions) => {
              const terminal = new Terminal({ cols: 80, rows: 4 });
              try {
                terminal.options.linkHandler = handler;
                await new Promise(resolve => terminal.write(`\x1b]8;;${uri}\x1b\\linked\x1b]8;;\x1b\\!`, resolve));
                let links;
                terminal._core._linkProviderService.linkProviders[0].provideLinks(1, value => { links = value; });
                const event = new MouseEvent('mousemove');
                for (const link of links) for (const action of actions) link[action]?.(event, link.text);
                return links.length;
              } finally { terminal.dispose(); }
            };
            return await Function('probe', program)(probe);
          }, program), new Promise((_, reject) => {
            deadline = setTimeout(() => reject(new Error('OSC handler trace exceeded 30 seconds')), 30_000);
          })]);
        } finally { clearTimeout(deadline); }
        if (expected === undefined) expected = entry.trace;
        assert.deepEqual(entry.trace, expected);
        assert.equal(entry.trace.length, 31);
        assert.deepEqual(entry.errors, []);
        entry.status = 'passed';
        console.log(`${name}: ${entry.trace.length} OSC handler scenarios passed`);
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
