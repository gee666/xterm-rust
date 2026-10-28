// Run the untouched upstream SerializeAddon browser assertions against Rust.
import { createRequire } from 'node:module';
import { readFile,writeFile,mkdir } from 'node:fs/promises';
import { createServer } from 'node:http';
import { runCommand } from './quality-process.mjs';
import path from 'node:path';
const root=path.resolve(import.meta.dirname,'..');
const require=createRequire(path.join(root,'source_repo/package.json'));
const {build}=require('esbuild');
const temp=path.join(root,'tmp/serialize-browser');await mkdir(temp,{recursive:true});
const testFile=path.join(temp,'SerializeAddon.test.cjs');
await build({absWorkingDir:root,entryPoints:['source_repo/addons/addon-serialize/test/SerializeAddon.test.ts'],outfile:testFile,
  bundle:true,platform:'node',format:'cjs',target:'node24',logLevel:'warning',packages:'external',
  tsconfig:'source_repo/src/browser/tsconfig.json',plugins:[{name:'rust-wasm-ready',setup(build){
    build.onLoad({filter:/test[\\/]playwright[\\/]TestUtils\.ts$/},async({path:file})=>({loader:'ts',contents:(await readFile(file,'utf8')).replace("await page.goto('/test');","await page.goto('/test'); await page.waitForFunction(() => window.rustSerializeReady === true);")}));
  }}]});
const html=`<!doctype html><html><head><link rel="stylesheet" href="/browser/xterm.css"></head><body><div id="terminal-container"></div><script type="module">
import initialize,* as terminal from '/browser/xterm.mjs';
import {SerializeAddon} from '/browser/serialize.mjs';
await initialize();Object.assign(window,terminal,{SerializeAddon});window.rustSerializeReady=true;
</script></body></html>`;
const mime={'.mjs':'text/javascript','.js':'text/javascript','.wasm':'application/wasm','.css':'text/css'};
const server=createServer(async(req,res)=>{
  try {
    const url=new URL(req.url,'http://localhost');
    if(url.pathname==='/test'){res.writeHead(200,{'content-type':'text/html'}).end(html);return;}
    if(!url.pathname.startsWith('/browser/')){res.writeHead(404).end();return;}
    const file=path.resolve(root,`.${decodeURIComponent(url.pathname)}`);
    if(!file.startsWith(path.join(root,'browser')+path.sep)){res.writeHead(403).end();return;}
    res.writeHead(200,{'content-type':mime[path.extname(file)]||'application/octet-stream'}).end(await readFile(file));
  }catch(error){res.writeHead(error.code==='ENOENT'?404:500).end(error.message);}
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const port=server.address().port;
const config=path.join(temp,'playwright.config.cjs');
await writeFile(config,`module.exports={testDir:${JSON.stringify(temp)},testMatch:'SerializeAddon.test.cjs',timeout:30000,workers:1,retries:0,use:{baseURL:'http://127.0.0.1:${port}',headless:true},projects:[{name:'chromium',use:{browserName:'chromium'}},{name:'firefox',use:{browserName:'firefox'}},{name:'webkit',use:{browserName:'webkit'}}]};\n`);
const cli=require.resolve('@playwright/test/cli');
try {
  const result=await runCommand(process.execPath,[cli,'test','--config',config,'--reporter=line',...process.argv.slice(2)],{
    cwd:root,timeoutMs:600000,log:path.join(temp,'playwright.log'),env:{...process.env,NODE_PATH:path.join(root,'source_repo/node_modules')}
  });
  console.log(await readFile(path.join(temp,'playwright.log'),'utf8'));
  process.exitCode=result.status==='passed'?0:1;
}finally{server.closeAllConnections();await new Promise(resolve=>server.close(resolve));}
