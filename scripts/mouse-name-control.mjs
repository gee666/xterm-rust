// Reuse the same cases against an explicitly supplied Rust/WASM package.
// This direct-ABI control is separate from wasm-bindgen-test execution.
import {readFile, writeFile, mkdir, copyFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {withArtifactServer} from './browser-artifact-server.mjs';

const root = resolve(import.meta.dirname, '..');
const expected = process.env.MOUSE_NAME_EXPECT;
if (!['old', 'fixed'].includes(expected) || !process.env.MOUSE_NAME_CONTROL_DIR) {
  throw new Error('Set MOUSE_NAME_EXPECT=old|fixed and MOUSE_NAME_CONTROL_DIR to a generated Rust package');
}
const directory = resolve(process.env.MOUSE_NAME_CONTROL_DIR);
const output = resolve(root, 'results/mouse-name-errors');
const cases = resolve(root, 'crates/xterm-web/tests/browser_mouse_name_cases.js');
const caseBytes = await readFile(cases);
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const fixture = JSON.parse(await readFile(resolve(output, 'source-chromium.json'), 'utf8'));
const playwright = await import(pathToFileURL(process.env.PLAYWRIGHT_MODULE ??
  resolve(root, 'source_repo/node_modules/playwright/index.mjs')));
const report = {expected, directAbiOnly:true, rustTestFunctions:0, browsers:[],
  casesSha256:hash(caseBytes), fixtureSha256:hash(await readFile(resolve(output, 'source-chromium.json'))),
  wasmSha256:hash(await readFile(resolve(directory, 'suite_bg.wasm'))),
  bindingsSha256:hash(await readFile(resolve(directory, 'suite.js'))), status:'failed'};
await mkdir(output, {recursive:true});
await copyFile(cases, resolve(directory, 'control-cases.js'));
try {
  await withArtifactServer({directory, css:await readFile(resolve(root, 'crates/xterm-web/assets/xterm.css')),
    html:'<!doctype html><link rel="stylesheet" href="/xterm.css">'}, async origin => {
    for (const name of (process.env.BROWSERS ?? 'chromium').split(',')) {
      if (!['chromium', 'firefox', 'webkit'].includes(name)) throw new Error('Invalid browser');
      const browser = await playwright[name].launch({headless:true, timeout:30000});
      try {
        const page = await browser.newPage();
        page.setDefaultTimeout(60000);
        await page.goto(origin);
        const result = await page.evaluate(async () => {
          const bindings = await import('/suite.js');
          await bindings.default();
          if (typeof bindings.Terminal !== 'function') throw new Error('No Rust Terminal export');
          const cases = await import('/control-cases.js');
          const units = value => Array.from({length:value.length}, (_, index) => value.charCodeAt(index));
          const probe = new bindings.Terminal();
          const mouse = probe._core.mouseStateService;
          const counterexamples = [];
          try {
            for (const [key, kind] of [['activeProtocol', 'protocol'], ['activeEncoding', 'encoding']]) {
              let error;
              try { mouse[key] = '\ud800'; } catch (caught) { error = caught; }
              counterexamples.push({key, errorClass:error?.constructor.name,
                actual:units(error?.message ?? ''), expected:units(`unknown ${kind} "\ud800"`),
                state:[mouse.activeProtocol, mouse.activeEncoding]});
            }
          } finally { probe.dispose(); }
          const results = [];
          for (const name of Object.keys(cases).sort()) {
            const term = new bindings.Terminal();
            try { results.push({name, actual:await cases[name](term)}); }
            catch (error) { results.push({name, error:String(error.stack ?? error)}); }
            finally { term.dispose(); }
          }
          return {counterexamples, results};
        });
        const entry = {name, version:browser.version(), ...result};
        report.browsers.push(entry);
        for (const item of entry.results) item.matched = !item.error &&
          JSON.stringify(item.actual) === JSON.stringify(fixture[item.name]);
        const matches = entry.results.filter(item => item.matched).length;
        entry.counts = {attempted:entry.results.length, matched:matches,
          mismatched:entry.results.length - matches, rustTestFunctions:0};
        if (entry.results.length !== 5 || matches !== (expected === 'old' ? 0 : 5)) {
          throw new Error(`Unexpected direct-ABI ${expected} control counts in ${name}`);
        }
        for (const item of entry.counterexamples) {
          const equal = JSON.stringify(item.actual) === JSON.stringify(item.expected);
          if (equal !== (expected === 'fixed') || item.errorClass !== 'Error' ||
            JSON.stringify(item.state) !== '["NONE","DEFAULT"]') {
            throw new Error('Counterexample no longer isolates unknown-name UTF16 conversion');
          }
        }
      } finally { await browser.close(); }
    }
  });
  if (hash(await readFile(cases)) !== report.casesSha256) throw new Error('Cases changed during control');
  report.status = 'passed';
} catch (error) {
  report.error = error.stack;
  process.exitCode = 1;
} finally {
  await writeFile(resolve(output, `control-${expected}.json`), JSON.stringify(report, null, 2) + '\n');
}
console.log(JSON.stringify({expected, status:report.status,
  browsers:report.browsers.map(({name, counts}) => ({name, counts})), error:report.error}, null, 2));
