// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
// Replays the shared callable-argument assertions on unchanged source classes.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
import { mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { withArtifactServer } from './browser-artifact-server.mjs';

const root = resolve(import.meta.dirname, '..');
const reference = resolve(process.env.SOURCE_ROOT ?? resolve(root, 'source_repo'));
const dependencies = resolve(process.env.SOURCE_DEPENDENCIES ?? reference, 'node_modules');
const require = createRequire(resolve(dependencies, '../package.json'));
const esbuild = require('esbuild');
const temporary = resolve(root, 'tmp/search-call-args-source');
const output = resolve(root, 'results/search-call-args');
const fixture = 'crates/xterm-web/tests/fixtures/search-call-args.js';
const report = {status: 'failed', platform: process.platform, node: process.version,
  scope: '81 shared source-derived callable search argument cases per browser. Not the complete upstream suite or equivalence.',
  browsers: []};
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
await mkdir(temporary, {recursive: true});
await mkdir(output, {recursive: true});
try {
  const sourcePaths = ['addons/addon-search/src/SearchAddon.ts', 'addons/addon-search/src/SearchState.ts',
    'addons/addon-search/src/SearchEngine.ts', 'addons/addon-search/src/DecorationManager.ts',
    'addons/addon-search/src/SearchLineCache.ts', 'addons/addon-search/src/SearchResultTracker.ts'];
  const hashes = async () => Object.fromEntries(await Promise.all(sourcePaths.map(async file =>
    [file, hash(await readFile(resolve(reference, file)))])));
  report.referenceBefore = await hashes();
  const body = await readFile(resolve(root, fixture), 'utf8');
  report.fixture = {path: fixture, sha256: hash(body)};
  await esbuild.build({
    stdin: {contents: "export {Terminal} from './src/browser/public/Terminal';export {SearchAddon} from './addons/addon-search/src/SearchAddon';",
      resolveDir: reference, loader: 'ts'},
    nodePaths: [resolve(reference, 'src')],
    tsconfigRaw: {compilerOptions: {experimentalDecorators: true, useDefineForClassFields: false}},
    outfile: resolve(temporary, 'source.mjs'), bundle: true, platform: 'browser', format: 'esm',
    target: 'es2022', logLevel: 'silent'
  });
  report.sourceBundleSha256 = hash(await readFile(resolve(temporary, 'source.mjs')));
  const css = await readFile(resolve(reference, 'css/xterm.css'));
  const playwright = await import(pathToFileURL(process.env.PLAYWRIGHT_MODULE
    ?? resolve(dependencies, 'playwright/index.mjs')));
  await withArtifactServer({directory: temporary, css,
    html: '<!doctype html><link rel="stylesheet" href="/xterm.css"><div id="terminal"></div>'}, async origin => {
    for (const name of ['chromium', 'firefox', 'webkit']) {
      const entry = {name, status: 'failed'};
      report.browsers.push(entry);
      let browser;
      try {
        browser = await playwright[name].launch({headless: true, timeout: 30000});
        entry.version = browser.version();
        const page = await browser.newPage();
        await page.goto(origin, {timeout: 30000});
        entry.cases = await page.evaluate(async body => {
          const {Terminal, SearchAddon} = await import('/source.mjs');
          const term = new Terminal({cols: 20, rows: 3, scrollback: 1000, allowProposedApi: true});
          const addon = new SearchAddon();
          term.loadAddon(addon);
          term.open(document.querySelector('#terminal'));
          const write = data => new Promise(resolve => term.write(data, resolve));
          const equal = (actual, expected) => {
            if (JSON.stringify(actual) !== JSON.stringify(expected)) {
              throw Error('actual ' + JSON.stringify(actual) + ' expected ' + JSON.stringify(expected));
            }
          };
          try {
            return await new Function(body + '\nreturn checkSearchCallArgs;')()(term, addon, write, equal);
          } finally { term.dispose(); }
        }, body);
        assert.equal(entry.cases, 81);
        entry.status = 'passed';
      } catch (error) { entry.error = String(error.stack ?? error); }
      finally { if (browser) await browser.close(); }
    }
  });
  report.referenceAfter = await hashes();
  assert.deepEqual(report.referenceAfter, report.referenceBefore, 'Search reference changed');
  assert.ok(report.browsers.every(entry => entry.status === 'passed'), 'Source search argument assertions failed');
  report.status = 'passed';
} catch (error) { report.error = String(error.stack ?? error); process.exitCode = 1; }
finally {
  esbuild.stop();
  await rm(temporary, {recursive: true, force: true});
  await writeFile(resolve(output, 'source.json'), JSON.stringify(report, null, 2) + '\n');
}
console.log(`Search source arguments ${report.status}: ${report.browsers.map(entry => `${entry.name} ${entry.cases ?? 0}/81`).join(', ')}`);
