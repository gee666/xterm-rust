// Public headless disposal phase probes against the unchanged source engine.
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {createHash,randomUUID} from 'node:crypto';
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import path from 'node:path';
const root=process.cwd();const id=randomUUID();const directory=`results/headless/disposal/${id}`;
await mkdir(directory,{recursive:true});
const report={id,status:'incomplete',node:process.version,platform:process.platform,sourceHashes:{},observations:[],
    counts:{discovered:10,executed:0,passed:0,failed:0,skipped:0,unrun:10}};
const persist=()=>writeFile(`${directory}/report.json`,JSON.stringify(report,null,2)+'\n');await persist();
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
try {
    const require=createRequire(path.resolve('source_repo/package.json'));
    let bundle;
    if(process.argv.includes('--existing-bundle')) {
        bundle=path.resolve('tmp/headless-reentry/source.cjs');
        const previous=JSON.parse(await readFile('results/headless/reentry-source.json','utf8'));
        for(const[file,expected]of Object.entries(previous.sourceHashes)) {
            const observed=hash(await readFile(file));assert.equal(observed,expected,file);report.sourceHashes[file]=observed;
        }
        report.provenance='Existing reference bundle from headless-reentry-oracle.mjs; source hashes revalidated, no fresh compilation. Diagnostic until fresh-bundle rerun.';
    } else {
        const {build}=require('esbuild');bundle=path.resolve(`tmp/headless-disposal/${id}/source.cjs`);
        await mkdir(path.dirname(bundle),{recursive:true});
        const built=await build({entryPoints:['source_repo/src/headless/public/Terminal.ts'],outfile:bundle,
            bundle:true,platform:'node',format:'cjs',metafile:true,logLevel:'silent',tsconfig:'source_repo/src/headless/tsconfig.json'});
        for(const file of Object.keys(built.metafile.inputs).sort())report.sourceHashes[file]=hash(await readFile(file));
        report.provenance='Fresh esbuild compilation of unchanged source implementation; no assertions or engine behavior rewritten.';
    }
    report.bundleSha256=hash(await readFile(bundle));const {Terminal}=require(bundle);
    const term=new Terminal({cols:8,rows:3,allowProposedApi:true,linkHandler:{activate(){}},documentOverride:{probe:true}});
    const phase={case:'marker-during-reentrant-dispose',publicData:[],internalData:[],unicode:[]};
    term.onData(data=>phase.publicData.push(data));term._core.coreService.onData(data=>phase.internalData.push(data));
    term._core.unicodeService.onChange(version=>phase.unicode.push(version));
    term.registerMarker(0).onDispose(()=>{
        phase.linkNull=term.options.linkHandler===null;phase.documentNull=term.options.documentOverride===null;
        term.input('M');term.unicode.activeVersion='6';
    });
    term.onBell(()=>term.dispose());term.write('A\x07B');term._core._writeBuffer.flushSync();
    phase.final=term.buffer.active.getLine(0).translateToString(true);phase.disposed=term._core._store.isDisposed;
    report.observations.push(phase);report.counts.executed++;
    assert.deepEqual(phase,{case:phase.case,publicData:[],internalData:['M'],unicode:['6'],linkNull:true,documentNull:true,final:'AB',disposed:true});
    report.counts.passed++;
    for(const action of ['resize','clear','reset','scroll','input','write','marker','parser','unicode']) {
        const term=new Terminal({cols:8,rows:3,allowProposedApi:true});term.write('A');term._core._writeBuffer.flushSync();term.dispose();
        let callback=false,error=null;
        try {
            switch(action) {
                case'resize':term.resize(6,4);break;case'clear':term.clear();break;case'reset':term.reset();break;
                case'scroll':term.scrollLines(-1);break;case'input':term.input('M');break;
                case'write':term.write('B',()=>{callback=true;});break;case'marker':term.registerMarker(1);break;
                case'parser':term.parser.registerOscHandler(999,()=>true);break;case'unicode':term.unicode.activeVersion='6';break;
            }
        } catch(thrown) {error=String(thrown.message);}
        const observed={action,error,callback,cols:term.cols,rows:term.rows,line:term.buffer.active.getLine(0)?.translateToString(true),markers:term.markers.length};
        report.observations.push(observed);report.counts.executed++;
        try {assert.deepEqual(observed,{action,error:null,callback:false,cols:action==='resize'?6:8,rows:action==='resize'?4:3,line:action==='reset'?'':'A',markers:action==='marker'?1:0});report.counts.passed++;}
        finally {for(const buffer of[term._core.buffers.normal,term._core.buffers.alt])buffer._memoryCleanupQueue.clear();for(const marker of[...term.markers])marker.dispose();}
    }
    report.status='passed';
} catch(error) {report.status='failed';report.error=error.stack??String(error);process.exitCode=1;}
finally {
    report.counts.failed=report.counts.executed-report.counts.passed;
    report.counts.unrun=report.counts.discovered-report.counts.executed;
    await persist();console.log(JSON.stringify({directory,...report},null,2));
}
