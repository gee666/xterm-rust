// Copyright (c) 2019-2026 The xterm.js authors. MIT license.
// Exercise unchanged source with real pointer input before changing our demo probe.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile, mkdir, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { withArtifactServer } from './browser-artifact-server.mjs';

const reference = resolve(process.env.SOURCE_ROOT ?? 'source_repo');
const output = resolve('results/link-demo-reload/source.json');
const { chromium, firefox, webkit } = await import(pathToFileURL(resolve(reference, 'node_modules/playwright/index.mjs')));
const hashes = {};
const inputs = ['lib/xterm.mjs', 'lib/xterm.mjs.map', 'css/xterm.css', 'src/browser/Linkifier.ts',
  'addons/addon-web-links/lib/addon-web-links.mjs', 'addons/addon-web-links/lib/addon-web-links.mjs.map',
  'addons/addon-web-links/src/WebLinksAddon.ts'];
for (const file of inputs) hashes[file] = createHash('sha256').update(await readFile(resolve(reference, file))).digest('hex');
for (const [mapFile, sourceFile] of [['lib/xterm.mjs.map', 'src/browser/Linkifier.ts'],
  ['addons/addon-web-links/lib/addon-web-links.mjs.map', 'addons/addon-web-links/src/WebLinksAddon.ts']]) {
  const map = JSON.parse(await readFile(resolve(reference, mapFile), 'utf8'));
  const index = map.sources.findIndex(name => name.endsWith('/' + sourceFile.split('/').at(-1)));
  assert(index >= 0, `Bundle identifies ${sourceFile}`);
  assert.equal(map.sourcesContent[index], await readFile(resolve(reference, sourceFile), 'utf8'));
}
const report = { node: process.version, platform: process.platform, hashes, browsers: [],
  scope: 'Directed original JS same-cell reload control with real pointer input. Not full link or port parity.' };
await mkdir(resolve('results/link-demo-reload'), { recursive: true });
try {
  await withArtifactServer({ directory: reference, css: await readFile(resolve(reference, 'css/xterm.css')),
    html: '<!doctype html><link rel="stylesheet" href="/xterm.css"><button id="reload">Reload</button><div id="terminal"></div>' }, async url => {
    for (const name of (process.env.BROWSERS ?? 'chromium,firefox,webkit').split(',')) {
      const kind = { chromium, firefox, webkit }[name];
      assert(kind, `Unknown browser ${name}`);
      const browser = await kind.launch({ headless: true, timeout: 30_000 });
      const entry = { name, version: browser.version(), status: 'running', errors: [], stages: [] };
      report.browsers.push(entry);
      try {
        const page = await browser.newPage();
        page.setDefaultTimeout(10_000);
        page.on('pageerror', error => entry.errors.push(error.message));
        await page.context().route('https://example.com/**', route => route.fulfill({ body: 'source reload target' }));
        await page.goto(url);
        await page.evaluate(async () => {
          const { Terminal } = await import('/lib/xterm.mjs');
          const { WebLinksAddon } = await import('/addons/addon-web-links/lib/addon-web-links.mjs');
          const terminal = window.terminal = new Terminal({ cols: 40, rows: 12 });
          terminal.open(document.querySelector('#terminal'));
          let addon;
          window.reloadLink = async () => {
            addon?.dispose();
            addon = new WebLinksAddon();
            terminal.loadAddon(addon);
            terminal.reset();
            await new Promise(done => terminal.write('https://example.com/', done));
            await new Promise(done => requestAnimationFrame(done));
          };
          document.querySelector('#reload').onclick = () => { window.reloaded = window.reloadLink(); };
          await window.reloadLink();
        });
        const rect = await page.locator('.xterm-screen').boundingBox();
        assert(rect && rect.width > 0 && rect.height > 0);
        const point = { x: rect.x + rect.width / 40 * 3, y: rect.y + rect.height / 12 / 2 };
        const inspect = label => page.evaluate(label => {
          const linkifier = terminal._core.linkifier;
          return { label, pointer: !!document.querySelector('.xterm-cursor-pointer'),
            lastCell: linkifier._lastBufferCell, activeLine: linkifier._activeLine,
            current: linkifier.currentLink?.link.text ?? null };
        }, label);
        await page.mouse.move(point.x, point.y);
        await page.waitForFunction(() => !!document.querySelector('.xterm-cursor-pointer'));
        entry.stages.push(await inspect('first hover'));
        const popupReady = page.waitForEvent('popup');
        await page.mouse.click(point.x, point.y);
        const popup = await popupReady;
        await popup.waitForURL('https://example.com/');
        await popup.close();
        entry.firstPopup = true;
        await page.locator('#reload').click();
        await page.evaluate(() => window.reloaded);
        entry.stages.push(await inspect('reload outside terminal'));
        await page.mouse.move(point.x, point.y);
        entry.stages.push(await inspect('same-cell return'));
        assert.equal(entry.stages.at(-1).pointer, false, 'Unchanged source does not rediscover the same cached cell');
        await page.mouse.move(point.x, point.y + rect.height / 12);
        await page.mouse.move(point.x, point.y);
        await page.waitForFunction(() => !!document.querySelector('.xterm-cursor-pointer'));
        entry.stages.push(await inspect('different-row return'));
        const secondPopupReady = page.waitForEvent('popup');
        await page.mouse.click(point.x, point.y);
        const secondPopup = await secondPopupReady;
        await secondPopup.waitForURL('https://example.com/');
        await secondPopup.close();
        entry.secondPopup = true;
        await page.evaluate(() => terminal.dispose());
        assert.deepEqual(entry.errors, []);
        entry.status = 'passed';
        console.log(`${name}: same-cell negative and different-row hover/popup PASS`);
      } catch (error) {
        entry.status = 'failed'; entry.error = String(error.stack ?? error);
        throw error;
      } finally { await browser.close(); }
    }
  });
} finally {
  await writeFile(output, JSON.stringify(report, null, 2) + '\n');
}
