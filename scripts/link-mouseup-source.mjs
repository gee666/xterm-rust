// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
// Directed geometry-access trace through the original opened Terminal/Linkifier.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile, mkdir, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { withArtifactServer } from './browser-artifact-server.mjs';

const reference = resolve(process.env.SOURCE_ROOT ?? 'source_repo');
const output = resolve('results/link-mouseup/source.json');
const observe = await readFile(new URL('./link-mouseup-observe.js', import.meta.url), 'utf8');
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
  scope: 'Seven directed mouseup coordinate/error phases through actual source Linkifier, not unchanged upstream tests.' };
await mkdir(resolve('results/link-mouseup'), { recursive: true });
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
        page.setDefaultTimeout(30_000);
        page.on('pageerror', error => entry.errors.push(error.message));
        await page.goto(url);
        const execution = page.evaluate(async observeSource => {
          const { Terminal } = await import('/xterm.mjs');
          const terminal = new Terminal({ cols: 80, rows: 4 });
          terminal.open(document.querySelector('#terminal'));
          try {
            await new Promise(resolve => terminal.write('ready', resolve));
            await new Promise(resolve => requestAnimationFrame(resolve));
            const root = terminal.element.querySelector('.xterm-screen');
            const linkifier = terminal._core.linkifier;
            let calls = 0;
            let mode = 0;
            const marker = {};
            // Only the Linkifier uses this controlled coordinate dependency. The
            // real DOM listener and source state machine remain unmodified.
            linkifier._mouseCoordsService = { getCoords() {
              calls++;
              if (mode === 1) return undefined;
              if (mode === 2) throw marker;
              return [5, 1];
            } };
            let reply;
            terminal.registerLinkProvider({ provideLinks(y, callback) { reply = callback; } });
            const observe = new Function(observeSource)();
            const trace = [];
            const log = [];
            const check = label => {
              const before = calls;
              const errors = observe(root, 'mouseup', marker);
              trace.push([label, calls - before, ...errors]);
            };
            mode = 2;
            check('idle');
            mode = 0;
            root.dispatchEvent(new MouseEvent('mousemove'));
            mode = 2;
            check('pending');
            mode = 0;
            reply([{ text: 'https://example.com/', range: {
              start: { x: 3, y: 1 }, end: { x: 7, y: 1 }
            }, activate() { log.push('activate'); }, hover() { log.push('hover'); }, leave() { log.push('leave'); } }]);
            root.dispatchEvent(new MouseEvent('mousedown'));
            check('active');
            mode = 1;
            check('invalid position');
            mode = 2;
            check('active geometry error');
            root.dispatchEvent(new MouseEvent('mouseleave'));
            check('cleared');
            terminal.dispose();
            check('disposed');
            return { trace, log };
          } finally { terminal.dispose(); }
        }, observe);
        let deadline;
        try {
          entry.trace = await Promise.race([execution, new Promise((_, reject) => {
            deadline = setTimeout(() => reject(new Error('Link mouseup trace exceeded 30 seconds')), 30_000);
          })]);
        } finally { clearTimeout(deadline); }
        assert.deepEqual(entry.trace, { trace: [
          ['idle', 0, 0, false], ['pending', 0, 0, false], ['active', 1, 0, false],
          ['invalid position', 1, 0, false], ['active geometry error', 1, 1, true],
          ['cleared', 0, 0, false], ['disposed', 0, 0, false]
        ], log: ['hover', 'activate', 'leave'] });
        assert.deepEqual(entry.errors, []);
        entry.status = 'passed';
        console.log(`${name}: ${JSON.stringify(entry.trace)}`);
      } catch (error) {
        entry.status = 'failed';
        entry.error = error.stack ?? String(error);
        throw error;
      } finally { await browser.close(); }
    }
  });
} finally {
  await writeFile(output, JSON.stringify(report, null, 2) + '\n');
}
