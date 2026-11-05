// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
// Directed regression through the original production Terminal and public providers.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile, mkdir, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { withArtifactServer } from './browser-artifact-server.mjs';

const reference = resolve(process.env.SOURCE_ROOT ?? 'source_repo');
const output = resolve('results/links/disposal-reentry');
await mkdir(output, { recursive: true });
const playwright = await import(pathToFileURL(process.env.PLAYWRIGHT_MODULE ?? resolve(reference, 'node_modules/playwright/index.mjs')));
const hashes = {};
for (const file of ['lib/xterm.mjs', 'lib/xterm.mjs.map', 'src/browser/Linkifier.ts']) {
  hashes[file] = createHash('sha256').update(await readFile(resolve(reference, file))).digest('hex');
}
const map = JSON.parse(await readFile(resolve(reference, 'lib/xterm.mjs.map'), 'utf8'));
const index = map.sources.findIndex(name => name.endsWith('/browser/Linkifier.ts'));
assert(index >= 0, 'Original bundle must identify its Linkifier source');
assert.equal(map.sourcesContent[index], await readFile(resolve(reference, 'src/browser/Linkifier.ts'), 'utf8'));
const report = { node: process.version, platform: process.platform, hashes, browsers: [],
  scope: 'Three new public-provider disposal reentry traces, not complete unchanged upstream suites.' };
try {
  await withArtifactServer({ directory: resolve(reference, 'lib'),
    css: await readFile(resolve(reference, 'css/xterm.css')),
    html: '<!doctype html><link rel="stylesheet" href="/xterm.css"><div id="terminal"></div>' }, async url => {
    for (const name of (process.env.BROWSERS ?? 'chromium,firefox,webkit').split(',')) {
      assert(['chromium', 'firefox', 'webkit'].includes(name));
      const browser = await playwright[name].launch({ headless: true, timeout: 30_000 });
      const entry = { name, version: browser.version(), status: 'running', errors: [], traces: [] };
      report.browsers.push(entry);
      try {
        const page = await browser.newPage();
        page.on('pageerror', error => entry.errors.push(error.message));
        await page.goto(url);
        const execution = page.evaluate(async () => {
          const { Terminal } = await import('/xterm.mjs');
          const traces = [];
          for (const replace of [false, true]) {
            const terminal = new Terminal({ cols: 80, rows: 4 });
            terminal.open(document.querySelector('#terminal'));
            try {
              // Wait for initial rendering before provider registration, so it cannot
              // introduce an unrelated cache invalidation during the synchronous trace.
              await new Promise(resolve => terminal.write('ready', resolve));
              await new Promise(resolve => requestAnimationFrame(resolve));
              const replies = [[], []];
              terminal.registerLinkProvider({ provideLinks(y, callback) { replies[0].push(callback); } });
              terminal.registerLinkProvider({ provideLinks(y, callback) { replies[1].push(callback); } });
              const dimensions = terminal._core._renderService.dimensions;
              const root = terminal.element.querySelector('.xterm-screen');
              const bounds = root.getBoundingClientRect();
              const move = row => root.dispatchEvent(new MouseEvent('mousemove', {
                clientX: bounds.left + 2.5 * dimensions.css.cell.width,
                clientY: bounds.top + (row - 0.5) * dimensions.css.cell.height,
                bubbles: true
              }));
              const log = [];
              const link = (x, dispose) => ({ text: `link${x}`, range: {
                start: { x, y: 1 }, end: { x: x + 2, y: 1 }
              }, activate() {}, dispose });
              move(1);
              const firstReply = replies[0].shift();
              const laterReply = replies[1].shift();
              if (replace) laterReply([link(10, () => log.push('old'))]);
              firstReply([link(3, () => {
                log.push('first');
                laterReply([link(20, () => log.push('new'))]);
              })]);
              move(2);
              traces.push({ replace, log });
            } finally { terminal.dispose(); }
          }
          const terminal = new Terminal({ cols: 80, rows: 4 });
          terminal.open(document.querySelector('#terminal'));
          try {
            await new Promise(resolve => terminal.write('ready', resolve));
            await new Promise(resolve => requestAnimationFrame(resolve));
            const replies = [];
            terminal.registerLinkProvider({ provideLinks(y, callback) { replies.push(callback); } });
            const dimensions = terminal._core._renderService.dimensions;
            const root = terminal.element.querySelector('.xterm-screen');
            const bounds = root.getBoundingClientRect();
            const move = row => root.dispatchEvent(new MouseEvent('mousemove', {
              clientX: bounds.left + 2.5 * dimensions.css.cell.width,
              clientY: bounds.top + (row - 0.5) * dimensions.css.cell.height, bubbles: true
            }));
            const log = [];
            const link = (x, dispose) => ({ text: `link${x}`, range: {
              start: { x, y: 1 }, end: { x: x + 2, y: 1 }
            }, activate() {}, dispose });
            move(1);
            let nested = false;
            replies.shift()([
              link(3, () => {
                log.push('first');
                if (!nested) {
                  nested = true;
                  move(3);
                  replies.shift()([link(20, () => log.push('new-cache'))]);
                }
              }), link(10, () => log.push('tail'))
            ]);
            move(2);
            traces.push({ nested: true, log });
          } finally { terminal.dispose(); }
          return traces;
        });
        let deadline;
        try {
          entry.traces = await Promise.race([execution, new Promise((_, reject) => {
            deadline = setTimeout(() => reject(new Error('Link disposal trace exceeded 30 seconds')), 30_000);
          })]);
        } finally { clearTimeout(deadline); }
        assert.deepEqual(entry.traces, [
          { replace: false, log: ['first', 'new'] }, { replace: true, log: ['first', 'new'] },
          { nested: true, log: ['first', 'first', 'tail', 'tail'] }
        ]);
        assert.deepEqual(entry.errors, []);
        entry.status = 'passed';
        console.log(`${name}: ${JSON.stringify(entry.traces)}`);
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
