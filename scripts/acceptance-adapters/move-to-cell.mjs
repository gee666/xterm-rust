// Copyright (c) 2017-2026 The xterm.js authors. All rights reserved. MIT license.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { readFile, writeFile, mkdir, statfs, copyFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import path from 'node:path';
import { runCommand } from '../quality-process.mjs';
import { pinArtifact } from '../acceptance-artifacts.mjs';
const require = createRequire(import.meta.url);
const seed=0x6d6f7665;
const relative=(root,file)=>path.relative(root,file).replaceAll('\\','/');
const sha256=bytes=>createHash('sha256').update(bytes).digest('hex');

export async function runCase({caseId,root,outputDirectory,signal}) {
  assert(['move-01','move-02'].includes(caseId),'unknown move case');
  await mkdir(outputDirectory,{recursive:true});
  const commands=[];
  async function resourceCheck(){
    signal?.throwIfAborted();const disk=await statfs(root);
    if(Number(disk.bavail)*Number(disk.bsize)<1024*1024*1024)throw Error('Move oracle needs 1024MiB shared disk reserve');
  }
  async function command(executable,args,cwd,logName,extraEnv={}){
    await resourceCheck();const log=path.join(outputDirectory,logName);
    const result=await runCommand(executable,args,{cwd,env:{...process.env,...extraEnv},log,timeoutMs:120000,resourceCheck});
    commands.push({argv:[executable,...args],exitCode:result.exitCode,log:relative(root,log)});
    assert.equal(result.status,'passed',`command failed: ${log}`);return readFile(log,'utf8');
  }
  const sourceRoot=path.join(root,'source_repo');
  await command(process.execPath,['bin/esbuild.mjs'],sourceRoot,'original-build.log');
  const originalReport=path.join(outputDirectory,'original-mocha.json');
  await command(process.execPath,['node_modules/mocha/bin/mocha.js','out-esbuild/browser/input/MoveToCell.test.js',
    '--reporter=json',`--reporter-options=output=${originalReport}`],sourceRoot,'original-mocha.log');
  const baseline=JSON.parse(await readFile(originalReport,'utf8'));
  assert.equal(baseline.stats.tests,4);assert.equal(baseline.stats.passes,4);assert.equal(baseline.stats.failures,0);
  const {moveToCellSequence}=require(path.join(sourceRoot,'out-esbuild/browser/input/MoveToCell.js'));
  const {BufferService}=require(path.join(sourceRoot,'out-esbuild/common/services/BufferService.js'));
  const {OptionsService}=require(path.join(sourceRoot,'out-esbuild/common/services/OptionsService.js'));
  const {DEFAULT_ATTR_DATA}=require(path.join(sourceRoot,'out-esbuild/common/buffer/BufferLine.js'));
  const {MockLogService}=require(path.join(sourceRoot,'out-esbuild/common/TestUtils.test.js'));
  function snapshot(service){
    const b=service.buffer;let text=`${b.x},${b.y}:${b.ybase}:${b.ydisp}:${b.lines.length}`;
    for(let row=0;row<b.lines.length;row++){
      const line=b.lines.get(row);text+=`|${Number(line.isWrapped)}`;
      for(let col=0;col<line.length;col++){
        const string=line.getString(col);const units=Array.from({length:string.length},(_,i)=>string.charCodeAt(i).toString(16).padStart(4,'0')).join('-');
        text+=`;${line.getWidth(col)},${line.getFg(col)},${line.getBg(col)},${line.getCodePoint(col)},${units}`;
      }
    }return text;
  }
  const edgeService=new BufferService(new OptionsService({cols:5,rows:5,scrollback:0}),new MockLogService());
  edgeService.buffer.x=3;edgeService.buffer.y=3;
  const zeroScrollback=moveToCellSequence(4,4,edgeService,false);
  assert.equal(zeroScrollback,'\x1b[B\x1b[C');edgeService.dispose();
  const fixtures=[],reference=[];let state=seed;
  const random=()=>state=(Math.imul(state,1664525)+1013904223)>>>0;
  for(const id of ['move-01','move-02'])for(let i=0;i<512;i++){
    const cols=2+random()%7,rows=2+random()%5;
    const startX=random()%cols,startY=random()%rows,targetX=random()%(cols+1),targetY=random()%(rows+1),app=random()%2;
    const wraps=Array.from({length:rows},()=>random()%3===0?'1':'0').join('');const kind=random()%5;
    const service=new BufferService(new OptionsService({cols,rows,scrollback:100}),new MockLogService());
    if(id==='move-02')service.buffers.activateAltBuffer();
    const b=service.buffer;b.x=startX;b.y=startY;
    for(let row=0;row<rows;row++){
      const line=b.lines.get(row);b.setWrapped(row,wraps[row]==='1');
      for(let col=0;col<cols;col++){
        const pattern=(row+col+kind)%5;
        const [cp,width]=[[0x4e2d,2],[0,0],[0x1f600,1],[0xd800,1],[101,1]][pattern];
        line.setCellFromCodepoint(col,cp,width,DEFAULT_ATTR_DATA);if(pattern===4)line.addCodepointToCell(col,0x301,0);
      }
    }
    const events=[];const subscriptions=[service.onResize(e=>events.push(['resize',e])),service.onScroll(e=>events.push(['scroll',e]))];
    const before=snapshot(service);const result=moveToCellSequence(targetX,targetY,service,!!app);const after=snapshot(service);
    assert.equal(after,before);assert.equal(service.buffer,b);assert.deepEqual(events,[]);
    const hex=Buffer.from(result,'utf8').toString('hex');
    fixtures.push([id,cols,rows,startX,startY,targetX,targetY,app,wraps,kind,hex].join('\t'));
    if(id===caseId)reference.push({returns:hex,errors:null,postState:after,events});
    for(const subscription of subscriptions)subscription.dispose();service.dispose();
  }
  const fixturePath=path.join(root,'crates/xterm-core/tests/fixtures/move-to-cell.tsv');
  const fixture=fixtures.join('\n')+'\n';
  if(process.env.MOVE_GENERATE==='1')await writeFile(fixturePath,fixture);
  else assert.equal((await readFile(fixturePath,'utf8')).replace(/\r\n/g,'\n'),fixture,'fixture drift; explicit MOVE_GENERATE=1 is required to regenerate');
  const cargo=process.env.CARGO||'cargo';
  const compilation=await command(cargo,['test','-p','xterm-core','--test','move_to_cell','--no-run','--message-format=json'],root,'rust-build.log');
  const messages=compilation.split(/\r?\n/).filter(line=>line.startsWith('{')).map(line=>JSON.parse(line));
  const artifact=messages.find(message=>message.reason==='compiler-artifact'&&message.target.name==='move_to_cell'&&message.profile.test);
  assert(artifact?.executable,'missing Rust native test executable');
  // Pin the executed artifact: shared target cleanup or later rebuilds must not
  // invalidate an immutable receipt before independent review.
  const pinned=await pinArtifact(root,artifact.executable);
  const nativeArtifact=path.join(root,pinned.path);
  const inputArtifact=path.join(outputDirectory,'input.tsv');await copyFile(fixturePath,inputArtifact);
  const fullRust=await command(nativeArtifact,[],root,'rust-complete-suite.log');
  assert.match(fullRust,/8 passed; 0 failed/);
  const upstreamRust=await command(nativeArtifact,['source_'],root,'rust-source-assertions.log');
  assert.match(upstreamRust,/4 passed; 0 failed/);
  const capturesFile=path.join(outputDirectory,'rust-captures.tsv');
  const testName=caseId==='move-01'?'move_01_differential_corpus':'move_02_differential_corpus';
  await command(nativeArtifact,[testName,'--exact'],root,'rust-run.log',{MOVE_CAPTURE:capturesFile});
  const actual=(await readFile(capturesFile,'utf8')).trimEnd().split(/\r?\n/).map(line=>{
    const [returns,postState,events]=line.split('\t');return {returns,errors:null,postState,events:JSON.parse(events)};
  });
  assert.equal(reference.length,512);assert.equal(actual.length,512);
  // Full independent captures are returned, not a shared success object.
  await writeFile(path.join(outputDirectory,'reference-captures.json'),JSON.stringify(reference,null,2)+'\n');
  await writeFile(path.join(outputDirectory,'rust-captures.json'),JSON.stringify(actual,null,2)+'\n');
  assert.deepEqual(actual,reference);
  const sourcePaths=['source_repo/src/browser/input/MoveToCell.ts','source_repo/src/browser/input/MoveToCell.test.ts',
    'source_repo/src/common/buffer/BufferLine.ts','source_repo/src/common/buffer/Buffer.ts'];
  const sourceHashes=await Promise.all(sourcePaths.map(async file=>({path:file,sha256:sha256(await readFile(path.join(root,file)))})));
  const executable=pinned.path;
  const capture=entries=>Object.fromEntries(['returns','errors','postState','events'].map(field=>[field,entries.map(entry=>entry[field])]));
  const result={caseId,sourceHashes,artifactHashes:[{path:executable,sha256:pinned.sha256},
    {path:relative(root,inputArtifact),sha256:sha256(await readFile(inputArtifact))}],commands,
    captures:[{engine:'native',runtimeVersion:`${process.platform}/${process.arch} Node ${process.version}`,implementation:'rust-native',
      assertions:reference.length*4,reference:capture(reference),rust:capture(actual)}],seed,sourceEdges:{normalZeroScrollback:Buffer.from(zeroScrollback).toString('hex')},original:{tests:4,passed:4,failed:0,skipped:0,unrun:0}};
  await writeFile(path.join(outputDirectory,'result.json'),JSON.stringify(result,null,2)+'\n');return result;
}
