import { createHash, randomUUID } from 'node:crypto';
import { createRequire } from 'node:module';
import { readFile, mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { withArtifactServer } from './browser-artifact-server.mjs';

// Runs the shared FastDomNode and Widget WASM assertions.
// No reference source or assertion is edited; esbuild only removes TS syntax.
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const require = createRequire(import.meta.url);
const source = 'source_repo/src/browser/scrollable/fastDomNode.ts';
const fixture = 'crates/xterm-web/tests/fixtures/fast-dom-cases.mjs';
const widgetSource = 'source_repo/src/browser/scrollable/widget.ts';
const widgetFixture = 'crates/xterm-web/tests/fixtures/widget-cases.mjs';
const visibilitySource = 'source_repo/src/browser/scrollable/scrollbarVisibilityController.ts';
const visibilityFixture = 'crates/xterm-web/tests/fixtures/visibility-cases.mjs';
const runId = `${new Date().toISOString().replace(/[:.]/g, '-')}-${randomUUID()}`;
const results = resolve(root, 'results/scrollbar-helpers/source', runId);
const work = resolve(root, 'tmp/fast-dom-source', runId);
await mkdir(results, { recursive: true });
await mkdir(work, { recursive: true });
const hash = async file => createHash('sha256').update(await readFile(resolve(root, file))).digest('hex');
const report = { status: 'preparing', platform: process.platform, node: process.version,
  source, fixture, widgetSource, widgetFixture, visibilitySource, visibilityFixture,
  command: process.argv, browsers: [] };
async function persist() {
  const json = JSON.stringify(report, null, 2) + '\n';
  await writeFile(resolve(results, 'report.json'), json);
  await writeFile(resolve(root, 'results/scrollbar-helpers/source-latest.json'), json);
}
try {
  await persist();
  const esbuild = require(process.env.ESBUILD_MODULE ?? '../source_repo/node_modules/esbuild');
  const playwright = await import(pathToFileURL(process.env.PLAYWRIGHT_MODULE ??
    resolve(root, 'source_repo/node_modules/playwright/index.mjs')));
  report.esbuild = esbuild.version;
  for (const key of ['source', 'fixture', 'widgetSource', 'widgetFixture', 'visibilitySource', 'visibilityFixture']) {
    report[`${key}Sha256`] = await hash(report[key]);
  }
  const bundle = await esbuild.build({ absWorkingDir: root, metafile: true, stdin: { contents: `
    export { FastDomNode } from './${source}';
    export { runFastCase, fastCases } from './${fixture}';
    export { Widget } from './${widgetSource}';
    export { runWidgetCase, widgetCases } from './${widgetFixture}';
    export { ScrollbarVisibilityController } from './${visibilitySource}';
    export { runVisibilityCase, visibilityCases } from './${visibilityFixture}';`, resolveDir: root },
    bundle: true, format: 'esm', platform: 'browser', outfile: resolve(work, 'suite.mjs') });
  report.inputs = Object.fromEntries(await Promise.all(Object.keys(bundle.metafile.inputs)
    .filter(file => file !== '<stdin>').sort().map(async file => [file, await hash(file)])));
  report.bundleSha256 = createHash('sha256').update(await readFile(resolve(work, 'suite.mjs'))).digest('hex');
  await withArtifactServer({ directory: work, css: '', html: '<!doctype html><meta charset="utf-8">' }, async url => {
    for (const name of ['chromium', 'firefox', 'webkit']) {
      const entry = { name, status: 'unrun', cases: [], errors: [] };
      report.browsers.push(entry);
      let browser;
      try {
        browser = await playwright[name].launch({ headless: true, timeout: 30000 });
        entry.version = browser.version();
        const page = await browser.newPage();
        page.setDefaultTimeout(60000);
        page.on('pageerror', error => entry.errors.push(error.stack ?? error.message));
        await page.goto(url);
        entry.cases = await page.evaluate(async () => {
          const { FastDomNode, runFastCase, fastCases, Widget, runWidgetCase, widgetCases,
            ScrollbarVisibilityController, runVisibilityCase, visibilityCases } = await import('/suite.mjs');
          const check = (group, name, run) => {
            try { return { group, name, status: 'passed', assertions: run() }; }
            catch (error) { return { group, name, status: 'failed', error: error.stack ?? String(error) }; }
          };
          return [
            ...fastCases.map(name => check('fastDomNode', name, () => runFastCase(name, node => new FastDomNode(node)))),
            ...widgetCases.map(name => check('widget', name, () => runWidgetCase(name, (node, kind, callback) => {
              const widget = new Widget();
              widget[`_on${kind}`](node, callback);
              return () => widget.dispose();
            }))),
            ...visibilityCases.map(name => check('visibility', name, () => runVisibilityCase(name, (mode, node) => {
              const controller = new ScrollbarVisibilityController(mode, 'shown', 'hidden');
              controller.setDomNode(new FastDomNode(node));
              return {
                setVisibility: value => controller.setVisibility(value),
                setIsNeeded: value => controller.setIsNeeded(value),
                setShouldBeVisible: value => controller.setShouldBeVisible(value),
                setDomNode: value => controller.setDomNode(new FastDomNode(value)),
                ensureVisibility: () => controller.ensureVisibility(),
                dispose: () => controller.dispose(),
              };
            }))),
          ];
        });
        entry.status = entry.cases.every(c => c.status === 'passed') && !entry.errors.length ? 'passed' : 'failed';
        console.log(`${name} ${entry.version}: ${entry.status}, ${entry.cases.length} shared source cases`);
      } catch (error) { entry.status = 'failed'; entry.errors.push(error.stack ?? error.message); }
      finally { if (browser) await browser.close(); await persist(); }
    }
  });
  report.status = report.browsers.length === 3 && report.browsers.every(b => b.status === 'passed') ? 'passed' : 'failed';
} catch (error) {
  report.status = 'failed'; report.error = error.stack ?? error.message;
  console.error(report.error);
}
finally { report.finishedAt = new Date().toISOString(); await persist(); }
if (report.status !== 'passed') process.exitCode = 1;
