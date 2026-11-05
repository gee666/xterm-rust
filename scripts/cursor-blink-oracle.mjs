// Bundle the unchanged source manager and the shared deterministic host.
import {readFile, writeFile, mkdir} from 'node:fs/promises';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import path from 'node:path';
import vm from 'node:vm';
const root = path.resolve(import.meta.dirname, '..');
const source = path.resolve(process.env.SOURCE_ROOT ?? path.join(root, 'source_repo'));
const require = createRequire(path.join(source, 'package.json'));
const esbuild = require('esbuild');
const hash = b => createHash('sha256').update(b).digest('hex');
const reference = JSON.parse(await readFile(path.join(root, 'docs/baseline/reference-manifest.json')));
for (const entry of reference) {
  if (hash(await readFile(path.join(source, entry.path))) !== entry.sha256) throw Error(`Reference changed: ${entry.path}`);
}
const sourceFile = path.join(source, 'addons/addon-webgl/src/CursorBlinkStateManager.ts');
const sourceBuild = await esbuild.build({entryPoints:[sourceFile], bundle:true, write:false,
  format:'iife', globalName:'BlinkReference', metafile:true,
  alias:{browser:path.join(source, 'src/browser')}});
const casesFile = path.join(root, 'crates/xterm-web/tests/cursor_blink_cases.js');
const casesBuild = await esbuild.build({entryPoints:[casesFile], bundle:true, write:false,
  format:'iife', globalName:'BlinkScenarios'});
const prelude = sourceBuild.outputFiles[0].text + '\n' + casesBuild.outputFiles[0].text;
const context = vm.createContext({console, structuredClone});
vm.runInContext(prelude, context, {timeout:10000});
const cases = vm.runInContext('BlinkScenarios.runCases(BlinkScenarios.sourceFactory)', context, {timeout:10000});
const inputs = [];
for (const file of Object.keys(sourceBuild.metafile.inputs)) {
  inputs.push({path:path.relative(source, file).replaceAll('\\', '/'), sha256:hash(await readFile(file))});
}
const work = path.join(root, 'tmp/cursor-blink');
const results = path.join(root, 'results/cursor-blink');
await mkdir(work, {recursive:true});
await mkdir(results, {recursive:true});
await writeFile(path.join(work, 'prelude.js'), prelude);
await writeFile(path.join(results, 'oracle.json'), JSON.stringify({
  referenceFilesVerified:reference.length, sourceInputs:inputs,
  sourceBundleSha256:hash(sourceBuild.outputFiles[0].text),
  sharedCasesSha256:hash(await readFile(casesFile)), cases
}, null, 2) + '\n');
console.log(`${reference.length} original files verified; ${cases.length} unchanged-source scenarios`);
