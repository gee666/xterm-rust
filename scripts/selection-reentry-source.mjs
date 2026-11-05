// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
// Exercise the real, unmodified original Terminal, not a model of its handlers.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile, mkdir, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { withArtifactServer } from './browser-artifact-server.mjs';

const reference = resolve(process.env.SOURCE_ROOT ?? 'source_repo');
const output = resolve('results/selection-service/reentry');
await mkdir(output, { recursive: true });
const playwright = await import(pathToFileURL(process.env.PLAYWRIGHT_MODULE ?? resolve(reference, 'node_modules/playwright/index.mjs')));
const files = ['lib/xterm.mjs', 'lib/xterm.mjs.map', 'src/browser/services/SelectionService.ts'];
const hashes = Object.fromEntries(await Promise.all(files.map(async name => [name,
  createHash('sha256').update(await readFile(resolve(reference, name))).digest('hex')])));
const sourceMap = JSON.parse(await readFile(resolve(reference, 'lib/xterm.mjs.map'), 'utf8'));
const selectionIndex = sourceMap.sources.findIndex(name => name.endsWith('/services/SelectionService.ts'));
assert(selectionIndex >= 0, 'Original bundle must identify its SelectionService source');
assert.equal(sourceMap.sourcesContent[selectionIndex],
  await readFile(resolve(reference, 'src/browser/services/SelectionService.ts'), 'utf8'),
  'Rebuild original bundle: embedded selection source differs from the read-only reference');
const css = await readFile(resolve(reference, 'css/xterm.css'));
const report = { node: process.version, platform: process.platform, hashes, browsers: [],
  scope: 'One new real Terminal reentry regression, not the complete unchanged selection suite.' };
try {
  await withArtifactServer({ directory: resolve(reference, 'lib'), css,
    html: '<!doctype html><link rel="stylesheet" href="/xterm.css"><div id="terminal" style="width:800px;height:160px"></div>' }, async url => {
    for (const name of (process.env.BROWSERS ?? 'chromium,firefox,webkit').split(',')) {
      assert(['chromium', 'firefox', 'webkit'].includes(name));
      const browser = await playwright[name].launch({ headless: true, timeout: 30_000 });
      const entry = { name, version: browser.version(), errors: [], status: 'running' };
      report.browsers.push(entry);
      try {
        const page = await browser.newPage();
        page.setDefaultTimeout(30_000);
        page.on('pageerror', error => entry.errors.push(error.message));
        await page.goto(url);
        const execution = page.evaluate(async () => {
          const { Terminal } = await import('/xterm.mjs');
          const terminal = new Terminal({ cols: 10, rows: 4 });
          const parent = document.querySelector('#terminal');
          terminal.open(parent);
          try {
            await new Promise(resolve => terminal.write('abcdefghij', resolve));
            const root = terminal.element;
            const dimensions = terminal._core._renderService.dimensions;
            const rect = root.getBoundingClientRect();
            const x = column => rect.left + (column + 0.1) * dimensions.css.cell.width;
            const y = rect.top + 0.5 * dimensions.css.cell.height;
            const mouse = (kind, column) => {
              const event = new MouseEvent(kind, { clientX: x(column), clientY: y, detail: 1, bubbles: true, cancelable: true });
              root.dispatchEvent(event);
              return event.defaultPrevented;
            };
            terminal.selectAll();
            let changes = 0;
            let nestedPrevented = false;
            const subscription = terminal.onSelectionChange(() => {
              if (++changes === 1) {
                terminal.selectAll();
                nestedPrevented = mouse('mousedown', 4);
              }
            });
            mouse('mousedown', 1);
            mouse('mousemove', 7);
            mouse('mouseup', 7);
            const result = { nestedPrevented, text: terminal.getSelection(), changes };
            subscription.dispose();
            return result;
          } finally { terminal.dispose(); }
        });
        let deadline;
        try {
          entry.result = await Promise.race([execution, new Promise((_, reject) => {
            deadline = setTimeout(() => reject(new Error('Source selection trace exceeded 30 seconds')), 30_000);
          })]);
        } finally { clearTimeout(deadline); }
        assert.deepEqual(entry.result, { nestedPrevented: true, text: 'efg', changes: 4 });
        assert.deepEqual(entry.errors, []);
        entry.status = 'passed';
        console.log(`${name}: ${JSON.stringify(entry.result)}`);
      } catch (error) {
        entry.status = 'failed';
        entry.error = error.stack ?? String(error);
        throw error;
      } finally { await browser.close(); }
    }
  });
} finally {
  await writeFile(resolve(output, 'source.json'), JSON.stringify(report, null, 2) + '\n');
}
