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
const terminalProgram = await readFile('scripts/osc-handler-terminal-trace.js', 'utf8');
const terminalFixture = resolve('crates/xterm-web/tests/fixtures/osc-handler-terminal.json');
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
let expectedTerminal = generate ? undefined : JSON.parse(await readFile(terminalFixture, 'utf8'));
const report = { platform: process.platform, node: process.version, hashes,
  traceSha256: hash(program), terminalTraceSha256: hash(terminalProgram), scope: 'Directed primitive OSC handler callbacks, not full links parity.', browsers: [] };
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
        // The older Rust policy test used a full circular list but expected a
        // missing row. Confirm the capacity alias before repairing its setup.
        entry.missingRowCapacity = await page.evaluate(async () => {
          const { Terminal } = await import('/xterm.mjs');
          return [0, 10].map(scrollback => {
            const terminal = new Terminal({ cols: 5, rows: 3, scrollback });
            try {
              let result;
              terminal._core._linkProviderService.linkProviders[0].provideLinks(99, links => {
                result = links === undefined ? 'undefined' : links.length;
              });
              return { scrollback, result };
            } finally { terminal.dispose(); }
          });
        });
        assert.deepEqual(entry.missingRowCapacity, [{ scrollback: 0, result: 0 },
          { scrollback: 10, result: 'undefined' }]);
        try {
          entry.liveTerminal = await Promise.race([page.evaluate(async program => {
          const { Terminal } = await import('/xterm.mjs');
          const terminal = new Terminal({ cols: 20, rows: 4, linkHandler: false });
          const parent = document.querySelector('#terminal');
          terminal.open(parent);
          try {
            await new Promise(resolve => terminal.write('\x1b]8;;custom:resource\x1b\\linked\x1b]8;;\x1b\\!', resolve));
            const rendered = new Promise((resolve, reject) => {
              const timeout = setTimeout(() => {
                subscription.dispose(); reject(new Error('Terminal render exceeded 10 seconds'));
              }, 10_000);
              const subscription = terminal.onRender(() => {
                clearTimeout(timeout); subscription.dispose(); resolve();
              });
            });
            terminal.refresh(0, 3);
            await rendered;
            const root = terminal.element;
            const rect = root.getBoundingClientRect();
            const cell = terminal._core._renderService.dimensions.css.cell;
            return Function('root,x,y', program)(root, rect.left + 2.5 * cell.width,
              rect.top + 0.5 * cell.height);
          } finally { terminal.dispose(); }
          }, terminalProgram), new Promise((_, reject) => {
            deadline = setTimeout(() => reject(new Error('Live OSC handler trace exceeded 30 seconds')), 30_000);
          })]);
        } finally { clearTimeout(deadline); }
        if (expectedTerminal === undefined) expectedTerminal = entry.liveTerminal;
        assert.deepEqual(entry.liveTerminal, expectedTerminal);
        assert.equal(entry.liveTerminal.pointerOn, true);
        assert.equal(entry.liveTerminal.pointerOff, true);
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
    await writeFile(terminalFixture, JSON.stringify(expectedTerminal, null, 2) + '\n');
  }
} finally {
  await writeFile(resolve(output, 'source.json'), JSON.stringify(report, null, 2) + '\n');
}
