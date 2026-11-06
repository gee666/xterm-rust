import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, existsSync, lstatSync, rmSync, rmdirSync } from 'node:fs';
import { resolve, dirname, join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
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
test('all addon production bundles include dependency licenses and unchanged core-only totals', () => fixture(({ source, deps, output, staging, invoke, args, script }) => {
  for (const name of ['alpha', 'beta']) mkdirSync(join(source, 'addons', `addon-${name}`), { recursive: true });
  mkdirSync(join(deps, 'bundled/dist'), { recursive: true });
  writeFileSync(join(deps, 'bundled/package.json'), '{"name":"bundled","version":"1.2.3","license":"MIT"}');
  writeFileSync(join(deps, 'bundled/dist/package.json'), '{"type":"module"}');
  writeFileSync(join(deps, 'bundled/dist/index.js'), 'export const value=1;');
  writeFileSync(join(deps, 'bundled/LICENSE'), 'Bundled dependency license');
  writeFileSync(join(source, 'bin/esbuild.mjs'), script + `
    const addon=process.argv.find(arg=>arg.startsWith('--addon='))?.split('=')[1];
    if(addon){const base='addons/addon-'+addon+'/lib/addon-'+addon+'.mjs';
    fs.mkdirSync('addons/addon-'+addon+'/lib',{recursive:true});
    fs.writeFileSync(base,'export const addon=true;');
    fs.writeFileSync(base+'.map',JSON.stringify({sources:['../../../node_modules/bundled/dist/index.js',${JSON.stringify(pathToFileURL(join(deps, 'bundled/src/not-shipped.ts')).href)}]}));}`);
  const result = invoke([...args, '--addons', 'all']);
  assert.equal(result.status, 0, result.stderr + result.stdout);
  const report = JSON.parse(readFileSync(join(output, 'original-build.json')));
  assert.deepEqual(report.addons, ['alpha', 'beta']);
  assert.equal(report.commands.length, 4);
  assert.deepEqual(report.bundledDependencies, [{ name: 'bundled', version: '1.2.3', license: 'MIT' }]);
  const sizes = JSON.parse(readFileSync(join(output, 'original-sizes.json')));
  const combined = sizes.distributions.find(row => row.id === 'original-web-all-addons-esm');
  assert.deepEqual(combined.addons, ['alpha', 'beta']);
  assert.equal(combined.files.filter(row => row.role === 'production addon ESM bundle').length, 2);
  assert.ok(combined.files.some(row => row.path === 'licenses/dependencies/bundled/LICENSE'));
  assert.equal(existsSync(staging), false);
  assert.equal(existsSync(join(source, 'node_modules')), false);
}));
test('missing bundled dependency license fails and removes prior sizes and owned scratch', () => fixture(({ source, deps, output, staging, invoke, args, script }) => {
  mkdirSync(join(source, 'addons/addon-alpha'), { recursive: true });
  mkdirSync(join(deps, 'bundled'));
  writeFileSync(join(deps, 'bundled/package.json'), '{"name":"bundled","version":"1.0.0"}');
  writeFileSync(join(deps, 'bundled/index.js'), 'export const value=1;');
  writeFileSync(join(source, 'bin/esbuild.mjs'), script + `
    if(process.argv.some(arg=>arg.startsWith('--addon='))){const base='addons/addon-alpha/lib/addon-alpha.mjs';
    fs.mkdirSync('addons/addon-alpha/lib',{recursive:true});fs.writeFileSync(base,'export const addon=true;');
    fs.writeFileSync(base+'.map',JSON.stringify({sources:['../../../node_modules/bundled/index.js']}));}`);
  assert.equal(invoke().status, 0);
  assert.equal(invoke([...args, '--addons', 'all']).status, 1);
  assert.match(JSON.parse(readFileSync(join(output, 'original-build.json'))).error, /No license found/);
  assert.equal(existsSync(join(output, 'original-sizes.json')), false);
  assert.equal(existsSync(staging), false);
}));
test('supplemental MIT notice is limited to the exact runtime version and source attribution', () => fixture(({ source, deps, output, invoke, args, script }) => {
  mkdirSync(join(source, 'addons/addon-alpha'), { recursive: true });
  mkdirSync(join(deps, 'inwasm-runtime/lib'), { recursive: true });
  writeFileSync(join(deps, 'inwasm-runtime/package.json'), '{"name":"inwasm-runtime","version":"0.1.2","license":"MIT"}');
  const runtime = join(deps, 'inwasm-runtime/lib/index.cjs');
  writeFileSync(runtime, '/**\n * Copyright (c) 2022, 2026 Joerg Breitbart\n * @license MIT\n */');
  writeFileSync(join(source, 'bin/esbuild.mjs'), script + `
    if(process.argv.some(arg=>arg.startsWith('--addon='))){const base='addons/addon-alpha/lib/addon-alpha.mjs';
    fs.mkdirSync('addons/addon-alpha/lib',{recursive:true});fs.writeFileSync(base,'export const addon=true;');
    fs.writeFileSync(base+'.map',JSON.stringify({sources:['../../../node_modules/inwasm-runtime/lib/index.cjs']}));}`);
  const values = [...args, '--addons', 'all'];
  const first = invoke(values);
  assert.equal(first.status, 0, first.stderr + first.stdout);
  const report = JSON.parse(readFileSync(join(output, 'original-build.json')));
  assert.match(report.bundledDependencies[0].supplementalNotice.noticeSha256, /^[a-f0-9]{64}$/);
  const sizes = JSON.parse(readFileSync(join(output, 'original-sizes.json')));
  assert.ok(sizes.distributions.find(row => row.id === 'original-web-all-addons-esm').files.some(row => row.path === 'licenses/dependencies/inwasm-runtime/LICENSE'));
  writeFileSync(runtime, '/** Copyright changed; do not guess a new license. */');
  assert.equal(invoke(values).status, 1);
  assert.match(JSON.parse(readFileSync(join(output, 'original-build.json'))).error, /requires review/);
  assert.equal(existsSync(join(output, 'original-sizes.json')), false);
}));
test('invalid addon policy fails rather than silently selecting core-only', () => fixture(({ invoke, args }) => {
  assert.notEqual(invoke([...args, '--addons', 'partial']).status, 0);
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
