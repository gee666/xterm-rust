// Source is bundled unchanged. Inputs and assertions are shared with WASM.
import fs from 'node:fs/promises';
import path from 'node:path';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {joinerRowScenarios} from '../crates/xterm-web/tests/support/joiner_row.mjs';
const root=process.cwd();
const require=createRequire(process.env.SOURCE_PACKAGE_JSON ?? path.join(root,'source_repo/package.json'));
const esbuild=require('esbuild');
const directory=path.join(root,'tmp/joiner-row');
await fs.mkdir(directory,{recursive:true});
const out=path.join(directory,'source.cjs');
const build=await esbuild.build({stdin:{contents:`
export {DomRendererRowFactory} from './source_repo/src/browser/renderer/dom/DomRendererRowFactory';
export {BufferLine} from './source_repo/src/common/buffer/BufferLine';`,resolveDir:root},
  outfile:out,bundle:true,platform:'node',format:'cjs',metafile:true,
  tsconfig:path.join(root,'source_repo/src/browser/tsconfig.json'),logLevel:'silent'});
const original='source_repo/src/browser/renderer/dom/DomRendererRowFactory.test.ts';
const originalOutput=path.join(directory,'original-tests.cjs');
await esbuild.build({entryPoints:[original],outfile:originalOutput,bundle:true,
  platform:'node',format:'cjs',packages:'external',
  tsconfig:path.join(root,'source_repo/src/browser/tsconfig.json'),logLevel:'silent'});
esbuild.stop();
const baseline=spawnSync(process.execPath,[require.resolve('mocha/bin/mocha.js'),originalOutput,
  '--grep','DomRendererRowFactory','--reporter','json'],{
  env:{...process.env,NODE_PATH:path.dirname(path.dirname(require.resolve('mocha/package.json')))},
  encoding:'utf8',timeout:60000,maxBuffer:16*1024*1024});
assert.equal(baseline.status,0,baseline.stderr||baseline.stdout);
const originalResult=JSON.parse(baseline.stdout);
assert.equal(originalResult.stats.failures,0);
const {DomRendererRowFactory,BufferLine}=require(out);
const {JSDOM}=require('jsdom');
const dom=new JSDOM('');
const actual=joinerRowScenarios(() => {
  const line=new BufferLine(6);
  for(const [i,cp,w] of [[0,97,1],[1,0x4e2d,2],[2,0,0],[3,98,1],[4,99,1],[5,100,1]]) {
    line.setCellFromCodepoint(i,cp,w,{fg:0,bg:0,extended:{}});
  }
  let raw=[];
  const color={css:'#ffffff',rgba:0xffffffff};
  const factory=new DomRendererRowFactory(dom.window.document,{getJoinedCharacters:()=>raw},
    {rawOptions:{drawBoldTextInBrightColors:true,minimumContrastRatio:1}},
    {isFocused:true},{isCursorHidden:false,isCursorInitialized:true},
    {forEachDecorationAtCell(){}},{colors:{foreground:color,background:{css:'#000000',rgba:255},
      ansi:Array(256).fill(color),selectionBackgroundOpaque:color,selectionInactiveBackgroundOpaque:color}});
  return {
    set:(i,cp,w)=>line.setCellFromCodepoint(i,cp,w,{fg:0,bg:0,extended:{}}),
    selection:(a,b)=>factory.handleSelectionChanged([a,0],[b,0],false),
    render(ranges,options) {
      raw=ranges;
      if(options.selection)factory.handleSelectionChanged([options.selection[0],0],[options.selection[1],0],!!options.column);
      const spans=factory.createRow(line,0,!!options.cursor,'block','outline',options.x??0,false,true,5,{get:()=>5},-1,-1);
      const container=dom.window.document.createElement('div');
      container.replaceChildren(...spans);
      return container.innerHTML;
    }
  };
});
dom.window.close();
const fixture='crates/xterm-web/tests/fixtures/joiner-row.json';
if(process.argv.includes('--generate'))await fs.writeFile(fixture,actual+'\n');
else assert.equal((await fs.readFile(fixture,'utf8')).trim(),actual,'Raw row fixture drift');
const hashes={};
for(const file of Object.keys(build.metafile.inputs).filter(file=>!file.startsWith('<'))) {
  hashes[file]=createHash('sha256').update(await fs.readFile(file)).digest('hex');
}
const report={node:process.version,platform:process.platform,
  originalSuite:{source:original,sha256:createHash('sha256').update(await fs.readFile(original)).digest('hex'),...originalResult.stats},scenarios:Object.keys(JSON.parse(actual)).length,
  seed:'0x348C0DE',randomized:256,sourceHashes:hashes,
  fixtureSha256:createHash('sha256').update(actual+'\n').digest('hex'),
  scope:'Raw DOM row consumption only. Infinite selection loops are not executed. Not full terminal equivalence.'};
await fs.mkdir('results/joiner-row',{recursive:true});
await fs.writeFile('results/joiner-row/original-suite.json',baseline.stdout);
await fs.writeFile('results/joiner-row/source.json',JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(report));
