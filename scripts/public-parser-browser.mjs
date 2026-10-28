// Execute every unchanged Parser.test.ts assertion against the Rust browser ABI.
// Only the fixture transport changes. No original terminal engine is served.
import { createServer } from 'node:http';
import { mkdir, readFile, writeFile, statfs } from 'node:fs/promises';
import { resolve, sep } from 'node:path';
import { pathToFileURL } from 'node:url';
import { createHash } from 'node:crypto';
import { runCommand } from './quality-process.mjs';

const root = resolve(import.meta.dirname,'..');
const id = new Date().toISOString().replace(/[:.]/g,'-');
const temporary = resolve(root,'tmp/public-parser-browser',id);
const results = resolve(root,'results/public-adapters/browser-parser',id);
await mkdir(temporary,{recursive:true});
await mkdir(results,{recursive:true});
const source = resolve(root,'source_repo/test/playwright/Parser.test.ts');
const original = await readFile(source,'utf8');
const terminalSource=resolve(root,'source_repo/test/playwright/Terminal.test.ts');
const terminalOriginal=await readFile(terminalSource,'utf8');
const hash = text=>createHash('sha256').update(text).digest('hex');
const report = {
  kind:'unchanged-parser-assertions-rust-browser',source:'source_repo/test/playwright/Parser.test.ts',sourceSha256:hash(original),
  fixture:'Original createTestContext/openTerminal/proxy transport replaced with Rust ABI page and direct public write/reset/evaluate calls. Actual .open is required. Unicode-graphemes addon setup omitted for these ASCII parser cases only.',
  selection:'Every Parser.test.ts case and only the loadAddon group from Terminal.test.ts; other Terminal groups are separate unmet evidence, not skipped passes.',
  terminalSourceSha256:hash(terminalOriginal),platform:process.platform,node:process.version,commands:[],status:'failed',
};
async function headroom() {
  const disk = await statfs(root);
  if(Number(disk.bavail)*Number(disk.bsize)<512*1024*1024) throw new Error('Build stopped below512MiB disk headroom');
}
async function command(executable,args,file,timeoutMs=180000,env=process.env) {
  await headroom();
  const entry = await runCommand(executable,args,{cwd:root,env,log:resolve(results,file),timeoutMs,resourceCheck:headroom});
  report.commands.push(entry);
  if(entry.status!=='passed') throw new Error(`Command failed: ${file}`);
}
function tests(suites,parents=[]) {
  return suites.flatMap(suite=>[
    ...(suite.specs??[]).flatMap(spec=>(spec.tests??[]).map(test=>({title:[...parents,suite.title,spec.title].join(' / '),project:test.projectName,results:test.results??[]}))),
    ...tests(suite.suites??[],[...parents,suite.title]),
  ]);
}
let server;
try {
  await command(process.env.CARGO??'cargo',['build','-p','xterm-web','--target','wasm32-unknown-unknown'],'build.log');
  await command(process.env.WASM_BINDGEN??resolve(root,'tmp/wasm-tools/wasm-bindgen-0.2.100-x86_64-pc-windows-msvc/wasm-bindgen.exe'),[
    resolve(root,'target/wasm32-unknown-unknown/debug/xterm_web.wasm'),'--target','web','--out-dir',resolve(temporary,'pkg'),'--out-name','xterm_web'
  ],'bindgen.log');
  const helper = resolve(temporary,'fixture.mjs');
  await writeFile(helper,`
export {pollFor,timeout} from ${JSON.stringify(resolve(root,'source_repo/test/playwright/TestUtils.ts').replaceAll('\\','/'))};
export async function createTestContext(browser) {
  const page = await browser.newPage();
  await page.goto('/test');
  await page.waitForFunction(()=>window.rustReady===true);
  const proxy = {
    evaluate:async fn=>page.evaluate(fn,[await page.evaluateHandle('window.term')]),
    reset:()=>page.evaluate(()=>window.term.reset()),
    write:data=>page.evaluate(data=>new Promise(resolve=>window.term.write(data,resolve)),data)
  };
  return {browser,page,proxy};
}
export async function openTerminal(ctx,options={}) {
  await ctx.page.evaluate(options=>{
    if(window.term) window.term.dispose();
    let element=document.querySelector('#terminal-container');
    if(element.children.length!==0) throw new Error('there must be no terminals on the page');
    const replacement=element.cloneNode(false);element.replaceWith(replacement);element=replacement;
    window.term = new window.Terminal({allowProposedApi:true,...options});
    window.term.open(element);
  },options);
}
`);
  const { build } = await import(pathToFileURL(resolve(root,'source_repo/node_modules/esbuild/lib/main.js')));
  await build({entryPoints:{'Parser.test':source,'Terminal.test':terminalSource},bundle:true,platform:'node',format:'cjs',outdir:temporary,outExtension:{'.js':'.cjs'},
    external:['@playwright/test'],nodePaths:[resolve(root,'source_repo/node_modules')],plugins:[{name:'rust-fixture-transport',setup(builder){
      builder.onResolve({filter:/[/\\]TestUtils$/},()=>({path:helper}));
    }}]});
  report.fixtureSha256=hash(await readFile(helper,'utf8'));
  const html = `<!doctype html><meta charset="utf-8"><link rel="stylesheet" href="/css/xterm.css"><div id="terminal-container"></div>
<script type="module">import {initialize,Terminal} from '/browser/xterm.mjs';await initialize();window.Terminal=Terminal;window.rustReady=true;</script>`;
  server=createServer(async(request,response)=>{
    try {
      const pathname=new URL(request.url,'http://localhost').pathname;
      if(pathname==='/test') {response.setHeader('Content-Type','text/html');response.end(html);return;}
      let file;
      if(pathname==='/browser/xterm.mjs') file=resolve(root,'browser/xterm.mjs');
      else if(pathname==='/css/xterm.css') file=resolve(root,'source_repo/css/xterm.css');
      else if(pathname.startsWith('/browser/pkg/')) {
        file=resolve(temporary,'pkg','.'+decodeURIComponent(pathname.slice('/browser/pkg'.length)));
        if(!file.startsWith(resolve(temporary,'pkg')+sep)) {response.writeHead(403).end();return;}
      } else {response.writeHead(404).end();return;}
      response.setHeader('Content-Type',file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':'text/javascript');
      response.end(await readFile(file));
    } catch(error) {response.writeHead(error.code==='ENOENT'?404:500).end(error.message);}
  });
  await new Promise((ready,reject)=>{server.once('error',reject);server.listen(0,'127.0.0.1',ready);});
  const port=server.address().port;
  const config=resolve(temporary,'playwright.config.cjs');
  await writeFile(config,`const loaded=require(${JSON.stringify(resolve(root,'source_repo/out-esbuild-test/playwright/playwright.config.js'))});
const original=loaded.default??loaded;
module.exports={...original,testDir:__dirname,testMatch:['Parser.test.cjs','Terminal.test.cjs'],grep:/Parser\\.test\\.cjs|Terminal\\.test\\.cjs.*loadAddon/,webServer:undefined,retries:0,workers:1,use:{...original.use,baseURL:'http://127.0.0.1:${port}'},reporter:'json'};
`);
  const raw=resolve(results,'playwright.json');
  try {
    await command(process.execPath,[resolve(root,'source_repo/node_modules/@playwright/test/cli.js'),'test',`--config=${config}`,`--output=${resolve(results,'artifacts')}`],'playwright.log',180000,
      {...process.env,NODE_PATH:resolve(root,'source_repo/node_modules'),PLAYWRIGHT_JSON_OUTPUT_FILE:raw,CI:'1',FORCE_COLOR:'0'});
  } finally {
    try {
      const data=JSON.parse(await readFile(raw,'utf8'));
      report.tests=tests(data.suites??[]);
      report.errors=data.errors??[];
      report.projects=Object.fromEntries([...new Set(report.tests.map(test=>test.project))].map(project=>{
        const selected=report.tests.filter(test=>test.project===project);
        return [project,{discovered:selected.length,executed:selected.filter(test=>test.results.some(result=>result.status!=='skipped')).length,
          passed:selected.filter(test=>test.results.at(-1)?.status==='passed').length,failed:selected.filter(test=>test.results.length&& !['passed','skipped'].includes(test.results.at(-1).status)).length,
          skipped:selected.filter(test=>test.results.at(-1)?.status==='skipped').length,unrun:selected.filter(test=>!test.results.length).length}];
      }));
    } catch(error) {if(error.code!=='ENOENT') throw error;}
  }
  if(hash(await readFile(source,'utf8'))!==report.sourceSha256 || hash(await readFile(terminalSource,'utf8'))!==report.terminalSourceSha256) throw new Error('Source assertions changed during execution');
  report.status='passed';
} catch(error) {report.error=error.stack??String(error);process.exitCode=1;}
finally {
  if(server) await new Promise((done,reject)=>server.close(error=>error?reject(error):done()));
  await writeFile(resolve(results,'report.json'),JSON.stringify(report,null,2)+'\n');
  await writeFile(resolve(root,'results/public-adapters/browser-parser-latest.json'),JSON.stringify({...report,artifacts:results},null,2)+'\n');
  console.log(JSON.stringify({status:report.status,projects:report.projects,error:report.error,results},null,2));
}
