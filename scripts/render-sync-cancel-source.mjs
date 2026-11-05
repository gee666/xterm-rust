// Test-only intact TypeScript source oracle. No reference files are modified.
import { createRequire } from 'node:module';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { resolve } from 'node:path';
import { createHash } from 'node:crypto';
import { pathToFileURL } from 'node:url';
const root = resolve(import.meta.dirname, '..');
const args = process.argv.slice(2);
if (args.length > 1 || args.some(arg => arg !== '--write')) throw new Error('Usage: node scripts/render-sync-cancel-source.mjs [--write]');
const require = createRequire(import.meta.url);
const esbuild = require(process.env.ESBUILD_MODULE ?? resolve(root, 'source_repo/node_modules/esbuild'));
const source = 'source_repo/src/browser/services/RenderService.ts';
const hash = createHash('sha256').update(await readFile(resolve(root, source))).digest('hex');
const entry = `import { RenderService } from './source_repo/src/browser/services/RenderService.ts';
export function makeSource(host) {
  const logs = [];
  const event = () => ({ dispose() {} });
  const core = { decPrivateModes: { synchronizedOutput: false } };
  const service = new RenderService(3, {},
    { onOptionChange: event, onMultipleOptionChange: event }, { warn() {} },
    { hasValidSize: true, onCharSizeChange: event }, core,
    { onDecorationRegistered: event, onDecorationRemoved: event },
    { onResize: event, buffers: { onBufferActivate: event } },
    { window: host.window, onDprChange: event, onWindowChange: event },
    { onChangeColors: event });
  service.onRenderedViewportChange(e => logs.push('viewport:' + e.start + ':' + e.end));
  service.onRender(e => logs.push('render:' + e.start + ':' + e.end));
  service.setRenderer({ onRequestRedraw: event, dispose() { logs.push('A.dispose'); },
    renderRows(start, end) { logs.push('A.rows:' + start + ':' + end); },
    handleSelectionChanged() { logs.push('A.selection:-:-:false'); } });
  host.frame(); logs.length = 0;
  return command => {
    switch (command.op) {
      case 'mode': core.decPrivateModes.synchronizedOutput = command.value; return;
      case 'refresh': return service.refreshRows(command.start, command.end, true, false);
      case 'dispose': return service.dispose();
      case 'state': return core.decPrivateModes.synchronizedOutput;
      case 'drain': return logs.splice(0);
      default: throw new Error('Unknown command ' + command.op);
    }
  };
}`;
const compiled = esbuild.buildSync({ stdin: { contents: entry, resolveDir: root, sourcefile: 'render-sync-cancel-entry.ts' },
  bundle: true, write: false, format: 'esm', target: 'es2022', metafile: true,
  tsconfigRaw: { compilerOptions: { experimentalDecorators: true } } });
const pins = [];
for (const path of Object.keys(compiled.metafile.inputs).filter(p => p.startsWith('source_repo/')).sort()) {
  pins.push({ path, sha256: createHash('sha256').update(await readFile(resolve(root, path))).digest('hex') });
}
const cases = await readFile(resolve(root, 'crates/xterm-web/tests/support/render_sync_cases.mjs'), 'utf8');
const handleCases = await readFile(resolve(root, 'crates/xterm-web/tests/support/render_sync_handles.mjs'), 'utf8');
const output = resolve(root, 'crates/xterm-web/tests/support/render_sync_oracle.mjs');
const fixture = '// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.\n'
  + '// Generated test-only intact source oracle. SHA-256: ' + hash + '\n' + compiled.outputFiles[0].text + '\n' + cases + '\n' + handleCases
  + '\nexport function sourceSyncCancelProgram(program) { return runSyncCancelProgram(makeSource, program); }\n'
  + 'export function rustSyncCancelProgram(program, callback) { return runSyncCancelProgram(host => command => callback(host, command), program); }\n'
  + 'export function sourceSyncHandleProgram(program) { return runSyncHandleProgram(makeSource, program); }\n'
  + 'export function rustSyncHandleProgram(program, callback) { return runSyncHandleProgram(host => command => callback(host, command), program); }\n';
if (args[0] === '--write') await writeFile(output, fixture);
else if (await readFile(output, 'utf8') !== fixture) throw new Error('Synchronized output oracle differs from unchanged source');
const oracle = await import(pathToFileURL(output).href + '?source-check');
const results = oracle.runSyncCancelCases(oracle.makeSource);
const fixturePath = resolve(root, 'crates/xterm-web/tests/fixtures/render-sync-cancel.json');
const json = JSON.stringify({ source, sha256: hash, pins, results }, null, 2) + '\n';
if (args[0] === '--write') await writeFile(fixturePath, json);
else if (await readFile(fixturePath, 'utf8') !== json) throw new Error('Synchronized output source fixture drift');
await mkdir(resolve(root, 'results/render-sync-cancel'), { recursive: true });
await writeFile(resolve(root, 'results/render-sync-cancel/source.json'), json);
const handles = JSON.stringify({ source, sha256: hash, pins, results: oracle.runSyncHandleCases(oracle.makeSource) }, null, 2) + '\n';
const handlesPath = resolve(root, 'crates/xterm-web/tests/fixtures/render-sync-handles.json');
if (args[0] === '--write') await writeFile(handlesPath, handles);
else if (await readFile(handlesPath, 'utf8') !== handles) throw new Error('Synchronized output handle fixture drift');
await mkdir(resolve(root, 'results/render-sync-handles'), { recursive: true });
await writeFile(resolve(root, 'results/render-sync-handles/source.json'), handles);
console.log(JSON.stringify({ source, sha256: hash, programs: results.length,
  handlePrograms: oracle.syncHandlePrograms.length, pins: pins.length,
  scope: 'source-only component corpus; not Rust or full terminal equivalence' }));
