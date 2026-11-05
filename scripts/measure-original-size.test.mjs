import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, existsSync, lstatSync, rmSync, rmdirSync } from 'node:fs';
import { resolve, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
const project = resolve(dirname(fileURLToPath(import.meta.url)), '..');
function fixture(run) {
  mkdirSync(join(project, 'tmp'), { recursive: true });
  const root = mkdtempSync(join(project, 'tmp/measure-original-'));
  const source = join(root, 'source'), deps = join(root, 'deps'), staging = join(root, 'stage'), output = join(root, 'results');
  for (const path of [source, join(source, 'bin'), join(source, 'css'), join(deps, 'esbuild')]) mkdirSync(path, { recursive: true });
  writeFileSync(join(source, 'LICENSE'), 'Fixture license');
  writeFileSync(join(source, 'package-lock.json'), '{}');
  writeFileSync(join(source, 'css/xterm.css'), '.fixture {}');
  writeFileSync(join(deps, 'esbuild/package.json'), '{"version":"fixture"}');
  const script = 'import fs from "node:fs"; const headless=process.argv.includes("--headless"); const folder=headless?"headless/lib-headless":"lib"; fs.mkdirSync(folder,{recursive:true}); fs.writeFileSync(folder+"/"+(headless?"xterm-headless.mjs":"xterm.mjs"),"export const fixture=true;");';
  writeFileSync(join(source, 'bin/esbuild.mjs'), script);
  const args = ['--source', source, '--dependencies', deps, '--staging', staging, '--output', output];
  const invoke = (values = args) => spawnSync(process.execPath, [join(project, 'scripts/measure-original-size.mjs'), ...values], { encoding: 'utf8', timeout: 15000 });
  try { run({ source, deps, staging, output, invoke, args, script }); }
  finally {
    // The command owns any dependency junction it creates. Never recurse through it.
    assert.equal(existsSync(join(source, 'node_modules')), false);
    rmSync(root, { recursive: true });
  }
}
test('unchanged-source fixture builds produce sizes, preserve sources and clean owned links/staging', () => fixture(({ source, staging, output, invoke, script }) => {
  const result = invoke();
  assert.equal(result.status, 0, result.stderr + result.stdout);
  const report = JSON.parse(readFileSync(join(output, 'original-build.json')));
  assert.equal(report.status, 'passed');
  assert.equal(report.commands.length, 2);
  assert.equal(readFileSync(join(source, 'bin/esbuild.mjs'), 'utf8'), script);
  assert.equal(existsSync(join(source, 'node_modules')), false);
  assert.equal(existsSync(staging), false);
  const first = readFileSync(join(output, 'original-sizes.json'), 'utf8');
  assert.equal(invoke().status, 0);
  assert.equal(readFileSync(join(output, 'original-sizes.json'), 'utf8'), first);
}));
test('failed build propagates, cleans its link and invalidates old sizes', () => fixture(({ source, output, staging, invoke }) => {
  assert.equal(invoke().status, 0);
  writeFileSync(join(source, 'bin/esbuild.mjs'), 'process.exit(7);');
  const result = invoke();
  assert.equal(result.status, 1);
  const report = JSON.parse(readFileSync(join(output, 'original-build.json')));
  assert.equal(report.status, 'failed');
  assert.equal(report.commands[0].status, 7);
  assert.equal(existsSync(join(output, 'original-sizes.json')), false);
  assert.equal(existsSync(staging), false);
}));
test('reference mutation fails instead of yielding a size comparison', () => fixture(({ source, output, invoke, script }) => {
  writeFileSync(join(source, 'bin/esbuild.mjs'), script + 'fs.writeFileSync("css/xterm.css","mutated");');
  assert.equal(invoke().status, 1);
  assert.match(JSON.parse(readFileSync(join(output, 'original-build.json'))).error, /inventory changed/);
  assert.equal(existsSync(join(output, 'original-sizes.json')), false);
}));
test('existing staging content and dependencies are not replaced or deleted', () => fixture(({ source, deps, staging, args, invoke }) => {
  mkdirSync(staging);
  writeFileSync(join(staging, 'keep'), 'owned by another run');
  assert.notEqual(invoke().status, 0);
  assert.equal(readFileSync(join(staging, 'keep'), 'utf8'), 'owned by another run');
  rmSync(staging, { recursive: true });
  mkdirSync(join(source, 'node_modules'));
  try {
    assert.notEqual(invoke().status, 0);
    assert.equal(lstatSync(join(source, 'node_modules')).isDirectory(), true);
    const invalid = [...args];
    invalid[invalid.indexOf('--staging') + 1] = deps;
    assert.notEqual(invoke(invalid).status, 0);
  } finally { rmdirSync(join(source, 'node_modules')); }
}));
