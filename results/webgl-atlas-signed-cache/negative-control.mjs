// Run only under an explicit compiler/browser allocation. Both mutations target
// real production paths. finally restores exact bytes even after failed probes.
import {readFile, writeFile, copyFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {resolve} from 'node:path';
import {runCommand} from '../../scripts/quality-process.mjs';
const root = fileURLToPath(new URL('../../', import.meta.url));
const out = resolve(root, 'results/webgl-atlas-signed-cache');
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const mutations = [
  {name: 'packed-cache-negative', path: 'crates/xterm-web/src/webgl/texture_atlas.rs',
    from: 'let cache_key = (key, colors);',
    to: 'let cache_key = (key, colors.map(|word| i64::from(word as u32)));', passed: 15, failed: 3},
  {name: 'packed-gpu-negative', path: 'crates/xterm-web/src/webgl/glyph_gpu.rs',
    from: 'atlas.rasterize_with_colors(\n                        cell,\n                        numbers,\n                        cell.chars.len() > 1,\n                        self.element.as_ref(),\n                    )',
    to: 'atlas.rasterize(cell, cell.chars.len() > 1, self.element.as_ref())', passed: 17, failed: 1},
];
const report = {date: new Date().toISOString(), mutations: [], commands: []};
async function browser(name, browsers, test = 'wasm_webgl_texture_atlas') {
  const result = await runCommand(process.execPath, ['scripts/browser-host-check.mjs'], {
    cwd: root, env: {...process.env, CARGO_BUILD_JOBS: '2', BROWSERS: browsers, WASM_TEST: test},
    log: resolve(out, name + '.log'), timeoutMs: 240000,
  });
  report.commands.push({name, ...result});
  if (result.error || result.timedOut || result.terminationError || result.signal) throw Error('Abnormal probe: ' + name);
  const saved = resolve(out, name + '.json');
  await copyFile(resolve(root, 'results/browser-host', test + '-results.json'), saved);
  return {result, receipt: JSON.parse(await readFile(saved, 'utf8'))};
}
async function persist() {await writeFile(resolve(out, 'negative-control.json'), JSON.stringify(report, null, 2) + '\n');}
for (const spec of mutations) {
  const file = resolve(root, spec.path), original = await readFile(file), before = hash(original);
  const text = original.toString('utf8');
  if (text.split(spec.from).length !== 2) throw Error('Expected one fixed production block: ' + spec.name);
  const mutant = text.replace(spec.from, spec.to);
  const entry = {...spec, before, mutant: hash(mutant)};
  report.mutations.push(entry);
  await writeFile(resolve(out, spec.name + '.rs.txt'), mutant);
  try {
    await writeFile(file, mutant);
    const {result, receipt} = await browser(spec.name, 'chromium');
    const counts = receipt.browsers[0]?.counts;
    if (result.exitCode !== 1 || counts?.passed !== spec.passed || counts?.failed !== spec.failed || counts?.ignored !== 0 || counts?.filtered !== 0) {
      throw Error('Unexpected mutant test counts: ' + spec.name + ' ' + JSON.stringify(counts));
    }
    entry.counts = counts;
    entry.failedTests = receipt.browsers[0].console.filter(line => line.text.includes('... FAIL')).map(line => line.text);
  } finally {
    await writeFile(file, original);
    entry.restored = hash(await readFile(file));
    await persist();
    if (entry.restored !== before) throw Error('Source not restored: ' + file);
  }
}
for (const [test, expected] of [['wasm_webgl_texture_atlas', 18], ['wasm_webgl_glyphs', 8]]) {
  const name = test === 'wasm_webgl_texture_atlas' ? 'final-atlas' : 'final-gpu';
  const {result, receipt} = await browser(name, 'chromium,firefox,webkit', test);
  if (result.exitCode !== 0 || receipt.browsers.length !== 3 || receipt.browsers.some(b =>
    b.counts?.passed !== expected || b.counts?.failed !== 0 || b.counts?.ignored !== 0 || b.counts?.filtered !== 0)) {
    throw Error('Restored test failure: ' + test);
  }
}
for (const entry of report.mutations) {
  if (hash(await readFile(resolve(root, entry.path))) !== entry.before) throw Error('Final source changed: ' + entry.path);
}
report.finishedAt = new Date().toISOString();
await persist();
console.log('Actual cache mutant15PASS3FAIL/GPU forwarding17PASS1FAIL; exactrestore; atlas18+GPU8 PASS each3engines');
