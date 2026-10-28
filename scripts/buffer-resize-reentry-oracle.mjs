// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
// The source bundle method matches buffer-baseline and buffer-service-oracle.
// This separate driver avoids modifying their independently owned fixture sets.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
const root=resolve(import.meta.dirname,'..');
const temporary=resolve(root,'tmp/buffer-resize-reentry');
const output=resolve(root,'results/buffer-reentry');
mkdirSync(temporary,{recursive:true});mkdirSync(output,{recursive:true});
const {build}=await import(pathToFileURL(resolve(root,'source_repo/node_modules/esbuild/lib/main.js')));
const bundle=resolve(temporary,'reference.cjs');
const built=await build({absWorkingDir:root,bundle:true,platform:'node',format:'cjs',metafile:true,
  tsconfig:'source_repo/src/common/tsconfig.json',outfile:bundle,stdin:{resolveDir:root,loader:'ts',contents:`
export { Buffer } from './source_repo/src/common/buffer/Buffer';
export { OptionsService } from './source_repo/src/common/services/OptionsService';
export { AttributeData } from './source_repo/src/common/buffer/AttributeData';
export { MockLogService } from './source_repo/src/common/TestUtils.test';`}});
const {Buffer,OptionsService,AttributeData,MockLogService}=createRequire(import.meta.url)(bundle);
const rows=[];
const units=text=>Array.from({length:text.length},(_,i)=>text.charCodeAt(i).toString(16)).join(',')||'-';
function setup(cols,height,scrollback=3) {
  const options=new OptionsService({cols,rows:height,scrollback});
  const buffer=new Buffer(true,options,{cols,rows:height},new MockLogService());
  buffer.fillViewportRows();
  const identities=Array.from({length:buffer.lines.length},(_,index)=>buffer.lines.get(index));
  const out=[];
  const snapshot=tag=>{
    const lines=[];
    for(let y=0;y<buffer.lines.length;y++) {
      const line=buffer.lines.get(y);
      if(!line) {lines.push('_');continue;}
      let identity=identities.indexOf(line);
      if(identity<0) {identity=identities.length;identities.push(line);}
      const cells=[];
      for(let x=0;x<line.length;x++) {
        const cell=line.getCell(x);
        cells.push([cell.content>>>0,cell.fg>>>0,cell.bg>>>0,cell.extended.ext>>>0,cell.extended.urlId,units(cell.getChars())].join(':'));
      }
      lines.push([identity+1,+line.isWrapped,cells.join('/')].join(';'));
    }
    out.push([tag,[buffer._cols,buffer._rows,buffer.x,buffer.y,buffer.ybase,buffer.ydisp,buffer.savedX,buffer.savedY,buffer.scrollTop,buffer.scrollBottom,buffer.lines.length,buffer.lines.maxLength].join(','),lines.join('|')].join('#'));
  };
  return {buffer,options,snapshot,out};
}
function letters(buffer,row,text) {
  for(let x=0;x<text.length;x++) buffer.lines.get(row).setCellFromCodepoint(x,text.codePointAt(x),1,new AttributeData());
}
function trimmed() {
  const state=setup(4,3);
  for(let i=0;i<3;i++) state.buffer.lines.push(state.buffer.getBlankLine(new AttributeData()));
  state.buffer.y=2;state.buffer.ybase=3;state.buffer.ydisp=3;
  state.options.rawOptions.scrollback=0;
  return state;
}
function run(name,execute) {const trace=execute();assert(trace.length>0);rows.push(`${name}\t${trace.join('^')}`);}
run('trim-read-write',()=>{
  const {buffer,snapshot,out}=trimmed();
  buffer.lines.onTrim(amount=>{snapshot(`trim,${amount}`);buffer.x=2;buffer.ybase=4;buffer.ydisp=3;buffer.savedY=8;snapshot('changed');});
  buffer.resize(4,2);snapshot('final');return out;
});
run('trim-nested-resize',()=>{
  const {buffer,snapshot,out}=trimmed();let nested=false;
  buffer.lines.onTrim(amount=>{snapshot(`trim,${amount}`);if(!nested){nested=true;buffer.resize(6,3);snapshot('nested-return');}});
  buffer.resize(4,2);snapshot('final');return out;
});
function larger() {const state=setup(2,5);letters(state.buffer,0,'ab');letters(state.buffer,1,'cd');state.buffer.setWrapped(1,true);state.buffer.y=4;return state;}
run('larger-read-write',()=>{
  const {buffer,snapshot,out}=larger();
  buffer.lines.onDelete(event=>{snapshot(`delete,${event.index},${event.amount}`);buffer.x=3;buffer.y=2;buffer.savedY=10;snapshot('changed');});
  buffer.resize(4,5);snapshot('final');return out;
});
run('larger-nested-resize',()=>{
  const {buffer,snapshot,out}=larger();let nested=false;
  buffer.lines.onDelete(event=>{snapshot(`delete,${event.index},${event.amount}`);if(!nested){nested=true;buffer.resize(4,5);snapshot('nested-return');}});
  buffer.resize(4,5);snapshot('final');return out;
});
function smaller() {const state=setup(4,5);letters(state.buffer,0,'abcd');state.buffer.y=4;return state;}
run('smaller-read-write',()=>{
  const {buffer,snapshot,out}=smaller();
  buffer.lines.onInsert(event=>{snapshot(`insert,${event.index},${event.amount}`);buffer.x=3;buffer.y=2;buffer.savedY=7;letters(buffer,0,'Z');snapshot('changed');});
  buffer.resize(2,5);snapshot('final');return out;
});
run('smaller-nested-resize',()=>{
  const {buffer,snapshot,out}=smaller();let nested=false;
  buffer.lines.onInsert(event=>{snapshot(`insert,${event.index},${event.amount}`);if(!nested){nested=true;buffer.resize(3,5);snapshot('nested-return');}});
  buffer.resize(2,5);snapshot('final');return out;
});
run('scrollback-coercion-reentry',()=>{
  const {buffer,options,snapshot,out}=setup(4,3);
  options.rawOptions.scrollback={valueOf(){snapshot('coerce');buffer.savedY=7;return 2;}};
  buffer.resize(4,3);snapshot('final');return out;
});
run('windows-getter-reentry',()=>{
  const {buffer,options,snapshot,out}=setup(4,3);
  options.rawOptions.windowsPty={get buildNumber(){snapshot('build');buffer.savedY++;return 21376;},get backend(){snapshot('backend');return 'conpty';}};
  buffer.resize(4,3);snapshot('final');return out;
});
run('trim-callback-throws',()=>{
  const {buffer,snapshot,out}=trimmed();
  buffer.lines.onTrim(()=>{snapshot('trim');throw new Error('resize observer failed');});
  assert.throws(()=>buffer.resize(4,2),/resize observer failed/);snapshot('failed-state');return out;
});
run('larger-callback-replaces-list',()=>{
  const {buffer,snapshot,out}=larger();
  buffer.lines.onDelete(()=>{snapshot('delete');buffer.clear();snapshot('cleared');});
  buffer.resize(4,5);snapshot('final');return out;
});
run('sparse-replacement-then-smaller',()=>{
  const {buffer,snapshot,out}=setup(5,4,6);
  for(let i=0;i<4;i++)buffer.lines.push(buffer.getBlankLine(new AttributeData()));
  letters(buffer,0,'abcde');letters(buffer,1,'fghij');buffer.setWrapped(1,true);
  for(let i=2;i<8;i++)letters(buffer,i,String.fromCharCode(105+i));
  buffer.x=1;buffer.y=3;buffer.ybase=4;buffer.ydisp=4;
  buffer.lines.onDelete(event=>{
    snapshot(`delete,${event.index},${event.amount}`);buffer.clear();buffer.fillViewportRows();
    for(let i=0;i<buffer.lines.length;i++){buffer.lines.get(i).resize(10,buffer.getNullCell(new AttributeData()));letters(buffer,i,'abcdefghij');}
    snapshot('refilled');
  });
  buffer.resize(10,4);snapshot('after-widen');
  assert.throws(()=>buffer.resize(4,4),/Cannot read properties of undefined/);snapshot('failed-state');return out;
});
const fixture=rows.join('\n')+'\n';
const fixturePath=resolve(root,'crates/xterm-core/tests/fixtures/buffer-resize-reentry.tsv');
if(process.argv.includes('--generate')) writeFileSync(fixturePath,fixture);
else assert.equal(readFileSync(fixturePath,'utf8'),fixture,'source reentry fixture drift');
const sourceHashes=Object.fromEntries(Object.keys(built.metafile.inputs).filter(p=>p.startsWith('source_repo/src/')).map(p=>[p,createHash('sha256').update(readFileSync(resolve(root,p))).digest('hex')]));
const report={command:'node scripts/buffer-resize-reentry-oracle.mjs',node:process.version,platform:process.platform,
  cases:rows.map(row=>row.split('\t')[0]),count:rows.length,sourceHashes,fixtureSha256:createHash('sha256').update(fixture).digest('hex'),
  scope:'Directed source Buffer reentrant callbacks only. Rust execution separate. Full equivalence NOT PROVEN.'};
writeFileSync(resolve(output,'oracle.json'),JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify({cases:rows.length,scope:report.scope}));
