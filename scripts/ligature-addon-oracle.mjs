// Executes copied upstream addon code. Only module resolution is adapted.
import fs from 'node:fs/promises';
import path from 'node:path';
import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';
import { createHash } from 'node:crypto';
import assert from 'node:assert/strict';
import {deflateSync} from 'node:zlib';
import {mutationCases, mutationInput, mutate, bufferMutationScenarios} from '../crates/xterm-font-addons/tests/support/font_buffer_mutations.mjs';
import {kernWarningScenarios} from '../crates/xterm-font-addons/tests/support/font_kern_warning_scenarios.mjs';
import {svgLoadingScenario} from '../crates/xterm-font-addons/tests/support/font_svg_scenarios.mjs';
import {metadataRejectionScenarios} from '../crates/xterm-font-addons/tests/support/font_metadata_scenarios.mjs';
import { lifecycleScenarios, loadingScenarios } from '../crates/xterm-font-addons/tests/support/ligature_scenarios.mjs';
const root = process.cwd();
const require = createRequire(process.env.SOURCE_PACKAGE_JSON ?? path.join(root, 'source_repo/package.json'));
const ts = require('typescript');
const source = path.join(root, 'source_repo/addons/addon-ligatures/src');
const temporary = path.join(root, 'tmp/ligature-addon-oracle');
const hashes = {};
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
await fs.mkdir(temporary, { recursive: true });
await fs.writeFile(path.join(temporary, 'package.json'), '{"type":"module"}\n');
async function copy(directory, relative = '') {
  for (const entry of await fs.readdir(directory, { withFileTypes: true })) {
    const name = path.join(relative, entry.name), file = path.join(directory, entry.name);
    if (entry.isDirectory()) { await copy(file, name); continue; }
    if (!name.endsWith('.ts') || name.endsWith('.test.ts')) continue;
    const bytes = await fs.readFile(file); hashes[path.relative(root, file)] = hash(bytes);
    let text = bytes.toString('utf8').replace(/from\s+(['"])(\.[^'"]*)\1/g, (_, q, spec) => `from ${q}${spec === '.' ? './index' : spec}.js${q}`);
    text = text.replace('export { IFont as Font, ILigatureData as LigatureData, IOptions as Options };', 'export type { IFont as Font, ILigatureData as LigatureData, IOptions as Options };');
    text = text.replace(/from\s+(['"])(opentype\.js|lru-cache)\1/g, (_, q, spec) => `from ${q}${pathToFileURL(path.join(temporary, spec + '.js')).href}${q}`);
    const output = ts.transpileModule(text, { compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ES2022 } }).outputText;
    const destination = path.join(temporary, name.replace(/\.ts$/, '.js'));
    await fs.mkdir(path.dirname(destination), { recursive: true }); await fs.writeFile(destination, output);
  }
}
await copy(source);
for (const [name, binding] of [['opentype.js', 'parse'], ['lru-cache', 'LRUCache']]) {
  await fs.writeFile(path.join(temporary, name + '.js'), `import {createRequire} from 'node:module'; const require=createRequire(${JSON.stringify(require.resolve(name))}); export const ${binding}=require(${JSON.stringify(name)}).${binding};\n`);
}
const { LigaturesAddon } = await import(pathToFileURL(path.join(temporary, 'LigaturesAddon.js')));
const bufferMutations = process.argv.includes('--buffer-mutations');
const svg = process.argv.includes('--svg');
const metadata = process.argv.includes('--metadata');
const kernWarnings = process.argv.includes('--kern-warnings');
const bytes = new Uint8Array(await fs.readFile(path.join(root, 'crates/xterm-addons/tests/fixtures/ligature-fonts', metadata || kernWarnings || svg ? 'UbuntuMono-Regular.ttf' : 'FiraCode-Regular.otf')));
const create = options => new LigaturesAddon(options);
async function mutationResults() {
  const {loadBuffer} = await import(pathToFileURL(path.join(temporary, 'fontLigatures/index.js')));
  const constructors = [], inputs = mutationCases().map(item => mutationInput(bytes, item, deflateSync));
  const descriptor = Object.getOwnPropertyDescriptor(console, 'warn');
  try {
    for (const [index, item] of mutationCases().entries()) {
      const data = inputs[index].slice(), sentinel = {}; let warnings = 0, outcome;
      Object.defineProperty(console, 'warn', {configurable: true, value() {warnings++; mutate(data, item); if (item.throws) throw sentinel;}});
      try {const font = loadBuffer(data.buffer); outcome = {status: 'ok', ranges: font.findLigatureRanges('a===b')};}
      catch (error) {outcome = error === sentinel ? {status: 'warning'} : {status: 'error', name: error.name, message: error.message};}
      constructors.push({...item, warnings, outcome});
    }
  } finally {if (descriptor) Object.defineProperty(console, 'warn', descriptor); else delete console.warn;}
  return {constructors, addon: await bufferMutationScenarios(create, inputs)};
}
const result = bufferMutations ? await mutationResults() : svg ? await svgLoadingScenario(create, bytes) : kernWarnings ? await kernWarningScenarios(create, bytes) : metadata ? await metadataRejectionScenarios(create, bytes) : { lifecycle: lifecycleScenarios(create), loading: await loadingScenarios(create, bytes) };
// Guard the dependency adapter: a broken OpenType import must not silently
// turn every expected native result into an error/fallback trace.
if (bufferMutations) {
  assert.equal(result.constructors.length, 39);
  assert.deepEqual(result.constructors[0].outcome.ranges, [[1, 4]]);
  assert.equal(result.addon['SFNT IgnoreBadPairs'].refreshes, 1);
  assert.equal(result.addon['SFNT ReadBadPairs'].refreshes, 0);
  assert.deepEqual(result.addon['SFNT DiscardCalt'].ranges, []);
  assert.equal(result.addon['WOFF-kern AddPair'].refreshes, 1);
} else if (svg) {
  assert.deepEqual(result.rejected, {ranges:[[0,3]],reads:1,refreshes:0});
  assert.deepEqual(result.accepted, {ranges:[],reads:2,refreshes:1});
  assert.deepEqual(result.cached, {reads:1,refreshes:0});
} else if (kernWarnings) {
  assert.equal(result.Valid.first.refreshes, 1);
  assert.equal(result.Truncated.first.refreshes, 0);
  assert.equal(result.Throw.previousIdentity, true);
  assert.equal(result.GetterThrow.previousIdentity, true);
  for (const mode of ['PrimitiveThrow', 'SymbolThrow', 'NullThrow', 'UndefinedThrow']) assert.equal(result[mode].previousIdentity, true);
  assert.equal(result.Reenter.first.refreshes, 1);
  assert.equal(result.Change.first.refreshes, 1);
} else if (metadata) {
  assert.deepEqual(result.metadata.rejected, {ranges:[[0,3]],reads:1,refreshes:0});
  assert.deepEqual(result.metadata.accepted, {ranges:[],reads:2,refreshes:1});
  assert.deepEqual(result.kern, result.metadata);
  assert.deepEqual(result.meta, result.metadata);
} else {
  assert.deepEqual(result.loading.nativeRanges, [[1, 3]]);
  assert.deepEqual(result.loading.foundTrace, [['register', 0], ['refresh', 0, 4]]);
}
const content = JSON.stringify(result, null, 2) + '\n';
const fixture = path.join(root, `crates/xterm-font-addons/tests/fixtures/${bufferMutations ? 'font-buffer-mutations' : svg ? 'font-svg-addon' : kernWarnings ? 'font-kern-warning-addon' : metadata ? 'font-metadata-addon' : 'ligature-addon'}.json`);
if (process.argv.includes('--generate')) await fs.writeFile(fixture, content);
else assert.equal(await fs.readFile(fixture, 'utf8'), content, 'Ligature addon source fixture drift');
const directory = path.join(root, 'results/font-addons'); await fs.mkdir(directory, { recursive: true });
const report = { node: process.version, scope: 'Source addon lifecycle/load boundary, not real Terminal renderer evidence', sourceHashes: hashes, ...(bufferMutations ? {dependencyHashes: Object.fromEntries(await Promise.all(['opentype.js', 'lru-cache'].map(async name => [name, hash(await fs.readFile(require.resolve(name)))])))} : {}), fixtureSha256: hash(content), fontSha256: hash(bytes), comparedOperationFields: bufferMutations ? result.constructors.length + Object.keys(result.addon).length : metadata || kernWarnings || svg ? Object.values(result).reduce((sum, item) => sum + Object.keys(item).length, 0) : Object.keys(result.lifecycle).length + Object.keys(result.loading).length };
await fs.writeFile(path.join(directory, bufferMutations ? 'font-buffer-mutation-oracle.json' : svg ? 'font-svg-addon-oracle.json' : kernWarnings ? 'font-kern-warning-addon-oracle.json' : metadata ? 'font-metadata-addon-oracle.json' : 'ligature-addon-oracle.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report));
