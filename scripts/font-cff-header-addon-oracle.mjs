// Executes unchanged copied source modules; only TypeScript/module resolution is adapted.
import fs from 'node:fs';
import path from 'node:path';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
import {cffHeaderInput} from '../crates/xterm-font-addons/tests/support/font_cff_header_input.mjs';
import {cffHeaderAddonScenarios} from '../crates/xterm-font-addons/tests/support/font_cff_header_addon.mjs';
const root=process.cwd(),require=createRequire(process.env.SOURCE_PACKAGE_JSON??path.join(root,'source_repo/package.json')),ts=require('typescript'),hashes={},cache=new Map();
const hash=x=>createHash('sha256').update(x).digest('hex');
function load(file) {
  if(cache.has(file))return cache.get(file).exports;
  const text=fs.readFileSync(file,'utf8');hashes[path.relative(root,file).replaceAll('\\','/')]=hash(text);
  const module={exports:{}};cache.set(file,module);
  function local(id) {
    if(!id.startsWith('.'))return require(id);
    const target=path.resolve(path.dirname(file),id),name=[target+'.ts',path.join(target,'index.ts')].find(p=>fs.existsSync(p));
    if(!name)throw new Error(`Unresolved source import ${id} in ${file}`);
    return load(name);
  }
  const code=ts.transpileModule(text,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.CommonJS,experimentalDecorators:true}}).outputText;
  new Function('require','module','exports',code)(local,module,module.exports);return module.exports;
}
const {LigaturesAddon}=load(path.join(root,'source_repo/addons/addon-ligatures/src/LigaturesAddon.ts'));
const fixture=JSON.parse(fs.readFileSync('crates/xterm-addons/tests/fixtures/font-cff-header.json'));
const selected={BadCff:c=>c.tag==='CFF '&&JSON.stringify(c.data)==='[3,7]'&&c.remove?.[0]==='hmtx'&&!c.woff,BadCff2:c=>c.tag==='CFF2'&&JSON.stringify(c.data)==='[3,7]'&&c.remove?.[0]==='hmtx'&&!c.woff,GoodCff:c=>c.tag==='CFF '&&JSON.stringify(c.header)==='[1,0,4]'&&!c.woff,GoodCff2:c=>c.tag==='CFF2'&&JSON.stringify(c.header)==='[1,0,4]'&&c.woff};
const base=new Uint8Array(fs.readFileSync('crates/xterm-addons/tests/fixtures/ligature-fonts/FiraCode-Regular.otf'));
const inputs=Object.fromEntries(Object.entries(selected).map(([name,predicate])=>{const c=fixture.cases.find(predicate);assert(c);return [name,cffHeaderInput(base,c,{...fixture.woffTemplates.otf,...c.encoded})];}));
const result=await cffHeaderAddonScenarios(options=>new LigaturesAddon(options),inputs);
assert.equal(result.first.refreshes,0);assert.equal(result.cached.reads,1);assert.equal(result.recovery.refreshes,1);assert.equal(result.second.refreshes,1);assert.equal(result.final.refreshes,2);
assert.deepEqual(result.debug,[null,{name:'Error',message:'Unsupported CFF table version 3.7'}]);
const text=JSON.stringify(result,null,2)+'\n',file='crates/xterm-font-addons/tests/fixtures/font-cff-header-addon.json';
if(process.argv.includes('--generate'))fs.writeFileSync(file,text);else assert.equal(fs.readFileSync(file,'utf8'),text);
fs.mkdirSync('results/font-cff-header',{recursive:true});fs.writeFileSync('results/font-cff-header/addon-oracle.json',JSON.stringify({node:process.version,sourceHashes:hashes,dependencySha256:hash(fs.readFileSync(require.resolve('opentype.js'))),fixtureSha256:hash(text),scenarios:4,result},null,2)+'\n');console.log(JSON.stringify(result));
