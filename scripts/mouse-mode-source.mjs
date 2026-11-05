// Capture shared mouse cases from the read-only original browser Terminal.
import {mkdir, readFile, writeFile, rm} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {withArtifactServer} from './browser-artifact-server.mjs';
const generate = process.argv.slice(2).join(' ') === '--generate';
if (process.argv.length > 2 && !generate) throw new Error('Usage: node scripts/mouse-mode-source.mjs [--generate]');
const root = resolve(import.meta.dirname, '..');
const reference = resolve(process.env.XT_REFERENCE_ROOT ?? resolve(root, 'source_repo'));
const dependencies = createRequire(resolve(reference, 'package.json'));
const {build} = dependencies('esbuild');
const playwright = await import(pathToFileURL(resolve(reference, 'node_modules/playwright/index.mjs')));
const family = process.env.MOUSE_CASE_FAMILY ?? 'modes';
const families = {
  modes: ['browser_mouse_mode_cases.js', 'mouse-modes'],
  'wheel-receiver': ['browser_wheel_receiver_cases.js', 'wheel-receiver'],
  'builtin-methods': ['browser_mouse_builtin_cases.js', 'mouse-builtin-methods'],
};
if (!Object.hasOwn(families, family)) throw new Error('Invalid MOUSE_CASE_FAMILY');
const [caseFile, resultDirectory] = families[family];
const temporary = resolve(root, `tmp/mouse-${family}-source`);
const output = resolve(root, `results/${resultDirectory}`);
await mkdir(temporary, {recursive:true});
await mkdir(output, {recursive:true});
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const source = resolve(reference, 'src/browser/public/Terminal.ts');
const cases = resolve(root, 'crates/xterm-web/tests', caseFile);
const mouseSource = resolve(reference, 'src/common/services/MouseStateService.ts');
const inputSource = resolve(reference, 'src/common/InputHandler.ts');
const report = {status:'failed', node:process.version, platform:process.platform,
  sourceSha256:hash(await readFile(source)), mouseSourceSha256:hash(await readFile(mouseSource)),
  inputSourceSha256:hash(await readFile(inputSource)), casesSha256:hash(await readFile(cases)), browsers:[]};
try {
  await build({entryPoints:[source], outfile:resolve(temporary, 'source.js'), bundle:true,
    format:'esm', platform:'browser', target:'es2022', nodePaths:[resolve(reference,'src')],
    tsconfig:resolve(reference,'src/browser/tsconfig.json')});
  await writeFile(resolve(temporary, 'cases.js'), await readFile(cases));
  await withArtifactServer({directory:temporary, css:await readFile(resolve(reference,'css/xterm.css')),
    html:'<!doctype html><link rel="stylesheet" href="/xterm.css">'}, async origin => {
    for (const name of ['chromium', 'firefox', 'webkit']) {
      const browser = await playwright[name].launch({headless:true, timeout:30000});
      try {
        const page = await browser.newPage();
        page.setDefaultTimeout(60000);
        await page.goto(origin);
        await page.evaluate(() => {
          (async () => {
            const {Terminal} = await import('/source.js');
            const cases = await import('/cases.js');
            const captures = {};
            for (const name of Object.keys(cases).sort()) {
              const term = new Terminal();
              try { captures[name] = await cases[name](term); } finally { term.dispose(); }
            }
            return captures;
          })().then(value => { globalThis.mouseResult = {value}; },
            error => { globalThis.mouseResult = {error:String(error.stack ?? error)}; });
        });
        await page.waitForFunction(() => globalThis.mouseResult !== undefined);
        const result = await page.evaluate(() => globalThis.mouseResult);
        if (result.error) throw new Error(result.error);
        const captures = result.value;
        report.browsers.push({name, version:browser.version(), captures});
        const fixture = resolve(output, `source-${name}.json`);
        if (generate) await writeFile(fixture, JSON.stringify(captures,null,2)+'\n');
        else if (JSON.stringify(captures)!==JSON.stringify(JSON.parse(await readFile(fixture,'utf8')))) {
          throw new Error(`Source capture drift in ${name}; inspect before using --generate`);
        }
      } finally { await browser.close(); }
    }
  });
  if (report.browsers.some(entry => JSON.stringify(entry.captures)!==JSON.stringify(report.browsers[0].captures))) {
    throw new Error('Source captures differ across browsers');
  }
  if (hash(await readFile(source))!==report.sourceSha256 || hash(await readFile(cases))!==report.casesSha256 ||
      hash(await readFile(mouseSource))!==report.mouseSourceSha256 || hash(await readFile(inputSource))!==report.inputSourceSha256) {
    throw new Error('Source or cases changed during capture');
  }
  report.status = 'passed';
} catch(error) { report.error=error.stack; process.exitCode=1; }
finally {
  await writeFile(resolve(output,'source-report.json'),JSON.stringify(report,null,2)+'\n');
  await rm(temporary,{recursive:true,force:true});
}
console.log(JSON.stringify(report,null,2));
