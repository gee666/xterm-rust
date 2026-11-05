// Execute the shared DOM trace through the unchanged original public Terminal.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { withArtifactServer } from './browser-artifact-server.mjs';

const source = resolve(process.env.SOURCE_ROOT ?? 'source_repo');
const output = resolve('results/link-screen');
const fixture = resolve('crates/xterm-web/tests/fixtures/link-screen.json');
const program = await readFile('scripts/link-screen-trace.js', 'utf8');
const generate = process.argv.includes('--generate');
const playwright = await import(pathToFileURL(process.env.PLAYWRIGHT_MODULE ??
  resolve(source, 'node_modules/playwright/index.mjs')));
const hash = value => createHash('sha256').update(value).digest('hex');
const map = JSON.parse(await readFile(resolve(source, 'lib/xterm.mjs.map'), 'utf8'));
const report = { platform: process.platform, node: process.version,
  scope: 'Custom real Terminal DOM trace, not unchanged upstream test definitions.',
  hashes: { 'scripts/link-screen-trace.js': hash(program) }, browsers: [] };
for (const file of ['lib/xterm.mjs', 'lib/xterm.mjs.map', 'src/browser/CoreBrowserTerminal.ts',
  'src/browser/Linkifier.ts', 'src/browser/services/MouseCoordsService.ts', 'src/browser/input/Mouse.ts']) {
  const bytes = await readFile(resolve(source, file));
  report.hashes[file] = hash(bytes);
  if (file.endsWith('.ts')) {
    const index = map.sources.findIndex(name => name.endsWith(file.slice(4)));
    assert(index >= 0, `Bundle must include ${file}`);
    assert.equal(map.sourcesContent[index], bytes.toString('utf8'), `Stale bundle: ${file}`);
  }
}
let expected = generate ? undefined : JSON.parse(await readFile(fixture, 'utf8'));
await mkdir(output, { recursive: true });
try {
  await withArtifactServer({ directory: resolve(source, 'lib'),
    css: await readFile(resolve(source, 'css/xterm.css')),
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
            const terminal = new Terminal({ cols: 20, rows: 4 });
            terminal.open(document.querySelector('#terminal'));
            try {
              const root = terminal.element;
              await new Promise(resolve => terminal.write('\x1b]8;;\x1b\\.....\x1b]8;;https://example.com/screen\x1b\\L\x1b]8;;\x1b\\!', resolve));
              await new Promise((resolve, reject) => {
                const timeout = setTimeout(() => { subscription.dispose(); reject(new Error('Render deadline')); }, 10_000);
                const subscription = terminal.onRender(() => { clearTimeout(timeout); subscription.dispose(); resolve(); });
                terminal.refresh(0, 3);
              });
              const cell = terminal._core._renderService.dimensions.css.cell;
              if (!(cell.width > 0 && cell.height > 0)) throw new Error('Invalid rendered cell dimensions');
              const probe = Function('root,width,height', program)(root, cell.width, cell.height);
              terminal.options.linkHandler = probe.handler;
              const snapshots = probe.run();
              terminal.dispose();
              return { snapshots, afterDispose: probe.afterDispose() };
            } finally { terminal.dispose(); }
          }, program), new Promise((_, reject) => {
            deadline = setTimeout(() => reject(new Error('DOM trace exceeded 30 seconds')), 30_000);
          })]);
        } finally { clearTimeout(deadline); }
        if (expected === undefined) expected = entry.trace;
        assert.deepEqual(entry.trace, expected);
        assert.equal(entry.trace.snapshots.length, 6);
        const hover = ['hover', 'mousemove', 'https://example.com/screen', 6, 6];
        const activate = ['activate', 'mouseup', 'https://example.com/screen', 6, 6];
        const leave = ['leave', 'mousemove', 'https://example.com/screen', 6, 6];
        assert.deepEqual(entry.trace.snapshots.map(({ rootPointer, screenPointer, log }) =>
          [rootPointer, screenPointer, log]), [
          [false, false, []], [false, true, [hover]], [false, true, [hover]],
          [false, true, [hover, activate]], [false, true, [hover, activate]],
          [false, false, [hover, activate, leave]]
        ]);
        assert.deepEqual(entry.trace.afterDispose, [hover, activate, leave]);
        assert.deepEqual(entry.errors, []);
        entry.status = 'passed';
        console.log(`${name}: six screen DOM snapshots passed`);
      } catch (error) { entry.status = 'failed'; entry.error = error.stack ?? String(error); throw error; }
      finally { await browser.close(); }
    }
  });
  if (generate) await writeFile(fixture, JSON.stringify(expected, null, 2) + '\n');
} finally { await writeFile(resolve(output, 'source.json'), JSON.stringify(report, null, 2) + '\n'); }
