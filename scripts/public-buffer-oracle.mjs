// Copyright (c) 2021 The xterm.js authors. MIT license.
// Observe public adapter behavior using the real source CircularList and CellData.
import assert from 'node:assert/strict';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
const root = resolve(import.meta.dirname,'..');
const { build } = await import(pathToFileURL(resolve(root,'source_repo/node_modules/esbuild/lib/main.js')));
const temporary = resolve(root,'tmp/public-buffer-oracle');
mkdirSync(temporary,{recursive:true});
const modules = {};
for (const [name,file] of [['BufferApiView','public/BufferApiView.ts'],['CircularList','CircularList.ts']]) {
  const output = resolve(temporary,`${name}.mjs`);
  await build({entryPoints:[resolve(root,'source_repo/src/common',file)],bundle:true,platform:'node',format:'esm',nodePaths:[resolve(root,'source_repo/src')],outfile:output});
  modules[name] = (await import(pathToFileURL(output)))[name];
}
let checks = 0;
const equal = (actual,expected) => {assert.deepEqual(actual,expected); checks++;};
const lines = new modules.CircularList(3);
const first = { label:'first' }, second = { label:'second' };
lines.push(first); lines.push(second);
const backing = {x:5,y:2,ydisp:1,ybase:4,lines};
const view = new modules.BufferApiView(backing,'normal');
equal([view.type,view.cursorX,view.cursorY,view.viewportY,view.baseY,view.length],['normal',5,2,1,4,2]);
equal(view.getLine(0),first);
equal(view.getLine(3),first);
equal(view.getLine(2),undefined);
lines.pop();
equal(view.getLine(1),second);
equal(view.length,1);
const a = view.getNullCell(), b = view.getNullCell();
assert.notEqual(a,b); checks++;
equal([a.getWidth(),a.getCode(),a.getChars(),a.fg,a.bg],[0,0,'',0,0]);
a.content=123;
equal(b.content,0);
const replacement = {x:0,y:0,ydisp:0,ybase:0,lines:new modules.CircularList(3)};
assert.equal(view.init(replacement),view); checks++;
equal([view.cursorX,view.cursorY,view.viewportY,view.baseY,view.length],[0,0,0,0,0]);
const methods=['getWidth','getChars','getCode','getFgColorMode','getBgColorMode','getFgColor','getBgColor',
  'isBold','isItalic','isDim','isUnderline','isBlink','isInverse','isInvisible','isStrikethrough','isOverline',
  'isFgRGB','isBgRGB','isFgPalette','isBgPalette','isFgDefault','isBgDefault','isAttributeDefault',
  'getUnderlineStyle','getUnderlineColor','getUnderlineColorMode','isUnderlineColorRGB','isUnderlineColorPalette','isUnderlineColorDefault'];
const empty=view.getNullCell();
const cellMethods=Object.fromEntries(methods.map(name=>[name,empty[name]()]));
cellMethods.attributesEquals=empty.attributesEquals(empty);
const fixture=resolve(root,'crates/xterm-web/tests/fixtures/public-cell-methods.json');
if(process.argv.includes('--generate')) {
  mkdirSync(resolve(root,'crates/xterm-web/tests/fixtures'),{recursive:true});
  writeFileSync(fixture,`${JSON.stringify(cellMethods,null,2)}\n`);
} else equal(cellMethods,JSON.parse(readFileSync(fixture,'utf8')));
const report = {source:'source_repo/src/common/public/BufferApiView.ts',status:'passed',node:process.version,checks,
  observations:{capacityIndexAliasesZero:true,poppedSlotRemainsVisible:true,nullCell:{width:0,content:0},initReturnsSameView:true},
  hashes:Object.fromEntries(['public/BufferApiView.ts','CircularList.ts','buffer/CellData.ts'].map(file=>[file,createHash('sha256').update(readFileSync(resolve(root,'source_repo/src/common',file))).digest('hex')])),
  scope:'Directed source adapter observations only. Public namespace/line/browser suites and full equivalence remain unproven.'};
mkdirSync(resolve(root,'results/public-adapters'),{recursive:true});
writeFileSync(resolve(root,'results/public-adapters/buffer-oracle.json'),`${JSON.stringify(report,null,2)}\n`);
console.log(JSON.stringify(report));
