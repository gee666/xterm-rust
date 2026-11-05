// Full original Terminal construction, not a Viewport or overview mock.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { createHash } from 'node:crypto';
import { mkdir, mkdtemp, readFile, writeFile, rm } from 'node:fs/promises';
import { resolve, dirname, relative } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { withArtifactServer } from './browser-artifact-server.mjs';
import { scrollbarExpected } from '../crates/xterm-web/tests/scrollbar_construction_cases.js';
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const reference = resolve(process.env.REFERENCE_ROOT ?? resolve(root, 'source_repo'));
await mkdir(resolve(root, 'tmp'), { recursive: true });
const temporary = await mkdtemp(resolve(root, 'tmp/scrollbar-construction-'));
const output = resolve(root, 'results/scrollbar-construction/source.json');
const report = { status: 'failed', scope: 'Eight synchronous full-Terminal scrollbar access cases',
  platform: process.platform, node: process.version, sources: [], browsers: [] };
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
await mkdir(dirname(output), { recursive: true });
try {
  const require = createRequire(resolve(reference, 'package.json'));
  const esbuild = require('esbuild');
  const fixture = resolve(root, 'crates/xterm-web/tests/scrollbar_construction_cases.js');
  const source = resolve(reference, 'src/browser/public/Terminal.ts');
  const build = await esbuild.build({
    stdin: { contents: `export {Terminal} from ${JSON.stringify(source)}; export * from ${JSON.stringify(fixture)};`,
      resolveDir: root, sourcefile: 'scrollbar-oracle-entry.ts' },
    outfile: resolve(temporary, 'source.js'), bundle: true, format: 'iife', globalName: 'ScrollbarOracle',
    platform: 'browser', target: 'es2022', tsconfig: resolve(reference, 'src/browser/tsconfig.json'),
    metafile: true, logLevel: 'silent'
  });
  for (const file of Object.keys(build.metafile.inputs).filter(file => !file.startsWith('scrollbar-oracle-entry'))) {
    const full = resolve(root, file);
    report.sources.push({ path: relative(root, full).replaceAll('\\', '/'), sha256: hash(await readFile(full)) });
  }
  report.esbuild = esbuild.version;
  const playwright = await import(pathToFileURL(resolve(reference, 'node_modules/playwright/index.mjs')));
  await withArtifactServer({ directory: temporary, css: await readFile(resolve(reference, 'css/xterm.css')),
    html: '<!doctype html><script src="/source.js"></script>' }, async origin => {
    for (const name of (process.env.BROWSERS ?? 'chromium,firefox,webkit').split(',')) {
      if (!['chromium', 'firefox', 'webkit'].includes(name)) throw new Error(`Unknown browser ${name}`);
      const browser = await playwright[name].launch({ headless: true, timeout: 30000 });
      const entry = { name, version: browser.version(), status: 'failed' };
      report.browsers.push(entry);
      try {
        const page = await browser.newPage();
        page.setDefaultTimeout(30000);
        await page.goto(origin);
        entry.captures = await page.evaluate(() => ScrollbarOracle.scrollbarExpected.map((_, index) =>
          ScrollbarOracle.scrollbarSourceCase(ScrollbarOracle.Terminal, index)));
        assert.deepEqual(entry.captures, scrollbarExpected);
        entry.status = 'passed';
      } catch (error) { entry.error = String(error.stack ?? error); }
      finally { await browser.close(); }
    }
  });
  if (report.browsers.every(entry => entry.status === 'passed')) report.status = 'passed';
} catch (error) { report.error = String(error.stack ?? error); }
finally {
  await writeFile(output, JSON.stringify(report, null, 2) + '\n');
  await rm(temporary, { recursive: true, force: true });
}
console.log(JSON.stringify(report.browsers, null, 2));
if (report.status !== 'passed') process.exitCode = 1;
