// Executes copied upstream addon code. Only module resolution is adapted.
import fs from 'node:fs/promises';
import path from 'node:path';
import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';
import { createHash } from 'node:crypto';
import assert from 'node:assert/strict';
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
const bytes = new Uint8Array(await fs.readFile(path.join(root, 'crates/xterm-addons/tests/fixtures/ligature-fonts/FiraCode-Regular.otf')));
const create = options => new LigaturesAddon(options);
const result = { lifecycle: lifecycleScenarios(create), loading: await loadingScenarios(create, bytes) };
// Guard the dependency adapter: a broken OpenType import must not silently
// turn every expected native result into an error/fallback trace.
assert.deepEqual(result.loading.nativeRanges, [[1, 3]]);
assert.deepEqual(result.loading.foundTrace, [['register', 0], ['refresh', 0, 4]]);
const content = JSON.stringify(result, null, 2) + '\n';
const fixture = path.join(root, 'crates/xterm-font-addons/tests/fixtures/ligature-addon.json');
if (process.argv.includes('--generate')) await fs.writeFile(fixture, content);
else assert.equal(await fs.readFile(fixture, 'utf8'), content, 'Ligature addon source fixture drift');
const directory = path.join(root, 'results/font-addons'); await fs.mkdir(directory, { recursive: true });
const report = { node: process.version, scope: 'Source addon lifecycle/load boundary, not real Terminal renderer evidence', sourceHashes: hashes, fixtureSha256: hash(content), fontSha256: hash(bytes), comparedOperationFields: Object.keys(result.lifecycle).length + Object.keys(result.loading).length };
await fs.writeFile(path.join(directory, 'ligature-addon-oracle.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report));
