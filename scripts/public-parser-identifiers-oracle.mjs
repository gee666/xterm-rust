// Observe source API accessor order and dispatch without altering source methods.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { createHash, randomUUID } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync, readdirSync, statSync, realpathSync } from 'node:fs';
import { resolve, dirname, relative, sep } from 'node:path';
const root=resolve(import.meta.dirname,'..');
process.chdir(root);
const id=`${new Date().toISOString().replace(/[:.]/g,'-')}-${randomUUID().slice(0,8)}`;
const directory=`results/parser-identifiers/runs/${id}`;
mkdirSync(directory,{recursive:true});
const pointer='results/parser-identifiers/oracle.json';
const report={id,status:'incomplete',scope:'Unchanged source headless API identifier getter/proxy and actual dispatch observations, not Rust/browser coverage.',commands:['node scripts/public-parser-identifiers-oracle.mjs'],node:process.version,platform:process.platform,observations:[]};
function persist(){const text=JSON.stringify(report,null,2)+'\n';writeFileSync(`${directory}/report.json`,text);writeFileSync(pointer,text);}
persist();
function hashes(){
  const paths=['source_repo/src/common','source_repo/src/headless'].flatMap(base=>readdirSync(base,{recursive:true}).map(path=>`${base}/${path}`)).filter(path=>statSync(path).isFile());
  paths.push('scripts/public-parser-identifiers-oracle.mjs','crates/xterm-wasm-support/tests/fixtures/parser_identifiers.mjs');
  return Object.fromEntries(paths.sort().map(path=>[path.replaceAll('\\','/'),createHash('sha256').update(readFileSync(path)).digest('hex')]));
}
try {
  report.inputsBefore=hashes();
  const require=createRequire(resolve(root,'source_repo/package.json'));
  const {identifierSpecs,makeIdentifier,registerIdentifier,identifierHook,observeError}=await import('../crates/xterm-wasm-support/tests/fixtures/parser_identifiers.mjs');
  const reuse=process.argv.includes('--reuse-built');
  let output=resolve(root,'source_repo/out-esbuild/headless/public/Terminal.js');
  function compiledHashes() {
    const entries=[];
    for(const file of Object.keys(require.cache).filter(file=>file.startsWith(realpathSync(resolve(root,'source_repo/out-esbuild'))+sep))) {
      const mapFile=file+'.map';
      const map=JSON.parse(readFileSync(mapFile,'utf8'));
      assert.equal(map.sources.length,map.sourcesContent.length,'missing embedded source provenance');
      for(let index=0;index<map.sources.length;index++) {
        const sourceFile=resolve(dirname(mapFile),map.sources[index]);
        assert.equal(readFileSync(sourceFile,'utf8'),map.sourcesContent[index],`stale generated reference: ${file}`);
      }
      for(const input of [file,mapFile])entries.push([relative(root,input).replaceAll('\\','/'),createHash('sha256').update(readFileSync(input)).digest('hex')]);
    }
    return Object.fromEntries(entries.sort(([a],[b])=>a.localeCompare(b)));
  }
  if(!reuse) {
    const {build}=require('esbuild');
    output=resolve(root,'tmp/parser-identifiers-oracle',id,'source.cjs');
    const bundled=await build({absWorkingDir:root,entryPoints:['source_repo/src/headless/public/Terminal.ts'],outfile:output,bundle:true,
      platform:'node',format:'cjs',target:'node24',tsconfig:'source_repo/src/headless/tsconfig.json',metafile:true,logLevel:'silent'});
    report.bundledSources=Object.keys(bundled.metafile.inputs).sort();
  }
  report.referenceMode=reuse?'Existing original ESBuild modules; every loaded source map content matched unchanged TS; no compiler/bundler invoked.':'Fresh unchanged-source ESBuild bundle.';
  const {Terminal}=require(output);
  if(reuse) {
    report.compiledBefore=compiledHashes();
    assert.ok(Object.keys(report.compiledBefore).length>0,'no compiled source modules were verified');
  }
  for(const spec of identifierSpecs()) {
    const terminal=new Terminal({cols:20,rows:3,scrollback:10,allowProposedApi:true});
    const log=[],calls=[];
    const {id:identifier,sentinel}=makeIdentifier(spec,log,identifierHook(terminal.parser,calls));
    let error=null;
    try { registerIdentifier(terminal.parser,spec,identifier,calls); }
    catch(thrown) { error=observeError(thrown,sentinel); }
    try { await new Promise(done=>terminal.write(spec.sequence,done)); }
    finally {terminal.dispose();}
    report.observations.push({name:spec.name,log,calls,error});
  }
  if(reuse) {
    report.compiledAfter=compiledHashes();
    for(const [file,hash] of Object.entries(report.compiledBefore))assert.equal(report.compiledAfter[file],hash,`compiled reference changed: ${file}`);
  }
  report.inputsAfter=hashes();
  report.snapshotStable=JSON.stringify(report.inputsBefore)===JSON.stringify(report.inputsAfter);
  assert.equal(report.snapshotStable,true,'source/helper changed during observations');
  const fixture='crates/xterm-wasm-support/tests/fixtures/parser-identifiers.json';
  if(process.argv.includes('--generate'))writeFileSync(fixture,JSON.stringify(report.observations,null,2)+'\n');
  else assert.deepEqual(report.observations,JSON.parse(readFileSync(fixture,'utf8')));
  report.fixture=fixture;
  report.fixtureSha256=createHash('sha256').update(readFileSync(fixture)).digest('hex');
  report.status='passed';
} catch(error){report.error=error.stack??String(error);process.exitCode=1;}
finally{persist();console.log(JSON.stringify({id,status:report.status,cases:report.observations.length,error:report.error,directory},null,2));}
