// Directed source probes for public reentrant mutation, without timer sleeps.
import { createRequire } from 'node:module';
import { mkdir,writeFile,readFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import path from 'node:path';
const require=createRequire(path.resolve('source_repo/package.json'));
const {build}=require('esbuild');
const outfile=path.resolve('tmp/headless-reentry/source.cjs');
await mkdir(path.dirname(outfile),{recursive:true});
const bundled=await build({entryPoints:['source_repo/src/headless/public/Terminal.ts'],outfile,bundle:true,
    platform:'node',format:'cjs',tsconfig:'source_repo/src/headless/tsconfig.json',metafile:true,logLevel:'silent'});
const {Terminal}=require(outfile);
const outcomes=[];
for(const action of ['observe','reset','dispose','resize','input','write']) {
    const term=new Terminal({cols:8,rows:3,allowProposedApi:true});
    const events=[];
    term.onData(data=>events.push(['data',data]));
    term.onResize(size=>events.push(['resize',size.cols,size.rows]));
    term.onBell(()=>{
        events.push(['bell',term.buffer.active.getLine(0).translateToString(true)]);
        if(action==='reset')term.reset();
        if(action==='dispose')term.dispose();
        if(action==='resize')term.resize(6,3);
        if(action==='input')term.input('in');
        if(action==='write')term.write('C',()=>events.push(['inner-callback']));
    });
    let error=null;
    term.write('A\x07B',()=>events.push(['outer-callback']));
    try{term._core._writeBuffer.flushSync();}catch(thrown){error=thrown.message;}
    outcomes.push({action,events,error,cols:term.cols,
        disposed:term._core._store.isDisposed,line:term.buffer.active.getLine(0)?.translateToString(true)});
    term.dispose();
}
const hashes={};
for(const file of Object.keys(bundled.metafile.inputs).sort())hashes[file]=createHash('sha256').update(await readFile(file)).digest('hex');
const report={command:'node scripts/headless-reentry-oracle.mjs',node:process.version,platform:process.platform,
    sourceHashes:hashes,mechanism:'Original queued write followed by original WriteBuffer.flushSync; no source assertions modified',outcomes};
await mkdir('results/headless',{recursive:true});
await writeFile('results/headless/reentry-source.json',JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(outcomes,null,2));
