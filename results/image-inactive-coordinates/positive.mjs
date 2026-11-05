// Use when retaining one complete fixed three-engine image-addon matrix.
import {readFile, writeFile, copyFile, mkdir} from 'node:fs/promises';
import {resolve, dirname} from 'node:path';
import {runCommand} from '../../scripts/quality-process.mjs';
const phase = process.argv[2];
if (!['initial', 'final'].includes(phase)) throw Error('Pass initial or final');
const out = resolve('results/image-inactive-coordinates');
await mkdir(out, {recursive: true});
const command = await runCommand(process.execPath, ['scripts/image-canvas-check.mjs'], {
  cwd: process.cwd(), env: {...process.env, WASM_TEST: 'image_addon_browser', BROWSERS: 'chromium,firefox,webkit'},
  timeoutMs: 300_000, log: resolve(out, `${phase}.log`)
});
await writeFile(resolve(out, `${phase}-command.json`), JSON.stringify(command, null, 2) + '\n');
for (const [from, to] of [
  ['results/browser-host/image_addon_browser-results.json', `${phase}-browser.json`],
  ['results/image-addon/verification.json', `${phase}-verification.json`]
]) await copyFile(from, resolve(out, to));
const report = JSON.parse(await readFile(resolve(out, `${phase}-browser.json`), 'utf8'));
if (report.artifacts) {
  const run = dirname(dirname(report.artifacts.wasm.path));
  await copyFile(resolve(run, 'image_addon_browser-build.log'), resolve(out, `${phase}-build.log`));
}
if (command.status !== 'passed' || report.status !== 'passed') throw Error(`${phase} matrix failed`);
if (report.browsers.map(b => b.name).join(',') !== 'chromium,firefox,webkit') throw Error('Unexpected browser matrix');
for (const browser of report.browsers) {
  if (browser.tests.length !== 56 || browser.counts.passed !== 56 || browser.counts.failed !== 0 ||
      browser.counts.ignored !== 0 || browser.counts.filtered !== 0 || browser.errors.length !== 0) {
    throw Error(`Incomplete or failing ${browser.name} run`);
  }
}
console.log(JSON.stringify(report.browsers.map(({name, version, counts}) => ({name, version, counts}))));
