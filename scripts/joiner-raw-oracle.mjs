// Bundle unchanged source. Only module resolution and input construction differ.
import fs from 'node:fs/promises';
import path from 'node:path';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {pathToFileURL} from 'node:url';
import assert from 'node:assert/strict';
import {joinerRawScenarios} from '../crates/xterm-web/tests/support/joiner_raw.mjs';
const root = process.cwd();
const require = createRequire(process.env.SOURCE_PACKAGE_JSON ?? path.join(root, 'source_repo/package.json'));
const esbuild = require('esbuild');
const temp = path.join(root, 'tmp/joiner-raw');
await fs.mkdir(temp, {recursive:true});
const output = path.join(temp, 'source.cjs');
const build = await esbuild.build({
  stdin: {contents: `export {CharacterJoinerService} from './source_repo/src/browser/services/CharacterJoinerService'; export {BufferLine} from './source_repo/src/common/buffer/BufferLine';`, resolveDir:root},
  outfile:output, bundle:true, platform:'node', format:'cjs', metafile:true,
  tsconfig:path.join(root, 'source_repo/src/browser/tsconfig.json'), logLevel:'silent'
});
const originalTest = 'source_repo/src/browser/services/CharacterJoinerService.test.ts';
const originalOutput = path.join(temp, 'original-tests.cjs');
await esbuild.build({entryPoints:[originalTest], outfile:originalOutput, bundle:true,
  platform:'node', format:'cjs', tsconfig:path.join(root,'source_repo/src/browser/tsconfig.json'),
  nodePaths:[path.resolve(path.dirname(require.resolve('typescript/package.json')), '..')], logLevel:'silent'});
esbuild.stop();
const Mocha = require('mocha');
const mocha = new Mocha({reporter:'dot'});
mocha.addFile(originalOutput);
const suite = await new Promise(resolve => {
  const runner = mocha.run(failed => resolve({failed, ...runner.stats}));
});
assert.equal(suite.failed, 0, 'Unchanged original CharacterJoinerService suite');
const {CharacterJoinerService, BufferLine} = await import(pathToFileURL(output)).then(module => module.default);
const actual = joinerRawScenarios(() => {
  const line = new BufferLine(6);
  for (const [index, cp, width] of [[0,97,1],[1,0x4e2d,2],[2,0,0],[3,98,1],[4,99,1],[5,100,1]]) {
    line.setCellFromCodepoint(index, cp, width, {fg:0,bg:0,extended:{}});
  }
  const service = new CharacterJoinerService({buffer:{lines:{get:() => line}}});
  return {register:handler => service.register(handler), deregister:id => service.deregister(id), get:() => service.getJoinedCharacters(0)};
});
const fixture = path.join(root, 'crates/xterm-web/tests/fixtures/joiner-raw.json');
if (process.argv.includes('--generate')) await fs.writeFile(fixture, actual + '\n');
else assert.equal((await fs.readFile(fixture,'utf8')).trim(), actual, 'Raw source joiner fixture drift');
const hashes = {};
for (const file of Object.keys(build.metafile.inputs).filter(file => !file.startsWith('<'))) {
  hashes[file] = createHash('sha256').update(await fs.readFile(file)).digest('hex');
}
const report = {node:process.version, platform:process.platform, esbuild:esbuild.version,
  originalSuite:{source:originalTest, sha256:createHash('sha256').update(await fs.readFile(originalTest)).digest('hex'), ...suite},
  scenarios:Object.keys(JSON.parse(actual)).length, directed:36, randomized:256, seed:'0x335C0DE', sourceHashes:hashes,
  fixtureSha256:createHash('sha256').update(actual+'\n').digest('hex'),
  scope:'Shared raw operation inputs against unchanged source. Not full renderer or terminal equivalence.'};
await fs.mkdir('results/joiner-raw',{recursive:true});
await fs.writeFile('results/joiner-raw/source.json', JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(report));
