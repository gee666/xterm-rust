// One bounded same-artifact Firefox diagnostic, then an unchanged full replay.
import assert from 'node:assert/strict';
import {cp, mkdir, readFile, rm, writeFile} from 'node:fs/promises';
import {join} from 'node:path';
import {freemem} from 'node:os';
import {statfs} from 'node:fs/promises';
import {runCommand} from '../../scripts/quality-process.mjs';
import {buildBootstrap} from '../../scripts/e2e/build-bootstrap.mjs';
import {startFixtureServer} from '../../scripts/e2e/fixture-server.mjs';
import {configForRust, testCounts} from '../../scripts/e2e/upstream-config.mjs';
import {sourceIdentity, verifyBuiltPackage} from '../../scripts/e2e/build-identity.mjs';
const root = process.cwd(), out = join(root,'results/integration520'), mode = process.argv[2];
assert(['firefox','final','review'].includes(mode));
const json = async file => JSON.parse(await readFile(file,'utf8'));
const pointer = join(root,'results/demos/build/latest.json'), original = await readFile(pointer);
const expected = await json(join(out,'browser-build-identity.json'));
const temporary = join(root,'tmp/520-diagnostic');
const reference = process.env.SOURCE_ROOT;
await mkdir(temporary,{recursive:true});
process.env.TEMP = process.env.TMP = temporary;
const report = {mode,status:'failed',input:expected.input,build:expected.build};
let server;
async function headroom() {
  const disk = await statfs(root);
  assert(freemem()>=8*1024**3 && Number(disk.bavail)*Number(disk.bsize)>=10*1024**3);
}
try {
  await headroom();
  await writeFile(pointer, await readFile(join(out,'browser-build-pointer.json')));
  assert.equal((await sourceIdentity(root)).sha256, expected.input.sha256);
  assert.deepEqual(await verifyBuiltPackage(root,expected.input),expected.build);
  if (mode==='firefox') {
    const directory = join(out,'firefox-diagnostic'); await mkdir(directory,{recursive:true});
    const bootstrapFile = join(temporary,'bootstrap.js');
    await buildBootstrap({root,referenceRoot:reference,outputFile:bootstrapFile});
    server = await startFixtureServer({root,referenceRoot:reference,packageRoot:join(root,'browser/pkg'),bootstrapFile,timeoutMs:180000});
    const configuration = join(temporary,'playwright.cjs');
    await writeFile(configuration,configForRust(join(reference,'addons/addon-serialize/test/playwright.config.ts'),server.origin));
    const args = [join(reference,'node_modules/@playwright/test/cli.js'),'test',`--config=${configuration}`,
      '--project=FirefoxStable','--reporter=json','--workers=1','--retries=0','--global-timeout=120000',`--output=${join(temporary,'artifacts')}`];
    report.result = await runCommand(process.execPath,args,{cwd:reference,timeoutMs:130000,log:join(directory,'playwright.log'),resourceCheck:headroom,
      env:{...process.env,CI:'1',FORCE_COLOR:'0',PLAYWRIGHT_JSON_OUTPUT_FILE:join(directory,'playwright.json')}});
    await writeFile(join(directory,'configuration.cjs.txt'),await readFile(configuration));
    report.counts = testCounts(await json(join(directory,'playwright.json')));
    assert.equal(report.result.status,'passed'); assert.equal(report.counts.passed,40);
    for (const field of ['failed','skipped','unrun']) assert.equal(report.counts[field],0);
  } else {
    report.result = await runCommand(process.execPath,['scripts/serialize-browser-check.mjs','--suite'],{
      cwd:root,env:process.env,timeoutMs:600000,resourceCheck:headroom,log:join(out,`serialize-browser-${mode}.log`)});
    await cp(join(root,'results/serialize-browser'),join(out,`browser-${mode}`),{recursive:true});
    const final = await json(join(out,`browser-${mode}/verification.json`));
    assert.equal(final.error,undefined); assert.deepEqual(final.cleanupErrors,[]);
    assert.equal(final.browsers.length,3);
    for (const entry of final.browsers) {
      assert.equal(entry.error,undefined); assert.equal(entry.discovered,85); assert.equal(entry.executed,85);
      assert.equal(entry.passed,85); assert.equal(entry.failed,0); assert.deepEqual(entry.mismatches,[]);
    }
    assert.equal(final.suites.length,2);
    for (const suite of final.suites) {
      assert.equal(suite.result.status,'passed'); assert.equal(suite.counts.passed,120);
      for (const field of ['failed','skipped','unrun']) assert.equal(suite.counts[field],0);
    }
    assert.equal(report.result.status,'passed'); report.browsers=final.browsers; report.suites=final.suites;
  }
  assert.equal((await sourceIdentity(root)).sha256,expected.input.sha256);
  assert.deepEqual(await verifyBuiltPackage(root,expected.input),expected.build);
  report.status='passed';
} catch(error) { report.error=String(error.stack); process.exitCode=1; }
finally {
  report.cleanupErrors=[];
  if(server) {
    report.requests=server.requests; report.failures=server.failures;
    try { await server.close(); } catch(error) { report.cleanupErrors.push(String(error)); process.exitCode=1; }
  }
  await writeFile(pointer,original); await rm(temporary,{recursive:true,force:true});
  await writeFile(join(out,`${mode}-diagnostic.json`),JSON.stringify(report,null,2)+'\n');
}
console.log(JSON.stringify(report));
