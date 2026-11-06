// Retain raw/runtime identities without generating new compiler or browser work.
import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
const out='results/integration554',sha=b=>createHash('sha256').update(b).digest('hex');
const json=f=>JSON.parse(fs.readFileSync(f));
const pins=[],seen=new Set();
function pin(file,kind,expected) {
 const full=path.resolve(file);if(seen.has(full))return;seen.add(full);
 const bytes=fs.readFileSync(full),hash=sha(bytes);
 if(expected&&hash!==expected)throw Error('Changed pinned file '+file);
 pins.push({path:file.replaceAll('\\','/'),kind,bytes:bytes.length,sha256:hash});
}
function tree(dir,kind='durable'){for(const e of fs.readdirSync(dir,{withFileTypes:true})){const f=path.join(dir,e.name);if(e.isDirectory())tree(f,kind);else if(e.isFile()&&e.name!=='review-pins.json')pin(f,kind);else if(!e.isFile())throw Error('Unexpected linked entry '+f);}}
tree(out);pin('docs/integration554.md','documentation');
for(const name of ['image-initial',...json(out+'/components-summary-wasm_public_decoration.json').completed.map(e=>e.test)]) {
 const report=json(out+'/'+name+'-results.json');
 for(const key of ['compiler','wasm','bindings'])pin(report.artifacts[key].path,'browser-'+key,report.artifacts[key].sha256);
 tree(path.dirname(report.artifacts.wasm.path),'browser-support');
}
for(const e of json(out+'/native-review-pins.json'))pin(e.path,'cached-native',e.sha256);
for(const e of json(out+'/production-artifacts.json').artifacts)pin(e.path,'production',e.sha256);
tree('browser/pkg','production');
pin('browser/xterm.mjs','production');
pin('browser/xterm.css','production');
fs.writeFileSync(out+'/review-pins.json',JSON.stringify({pins},null,2)+'\n');
console.log(JSON.stringify({pins:pins.length,durable:pins.filter(p=>p.kind==='durable').length,native:pins.filter(p=>p.kind==='cached-native').length,browser:pins.filter(p=>p.kind.startsWith('browser')).length,production:pins.filter(p=>p.kind==='production').length}));
