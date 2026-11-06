// Copy three existing packages required to load the original image test module.
import assert from 'node:assert/strict';
import {readFile,writeFile,mkdir,lstat,readdir,copyFile,realpath} from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
const source='P:/workshop/Windows_Worshop/oira666/xterm-rust/source_repo/node_modules';
const destination=path.resolve('source_repo/node_modules');
const packages=['sixel','xterm-wasm-parts','inwasm-runtime'];
for(const pkg of packages) { try { await lstat(path.join(destination,pkg)); throw Error("Refusing existing dependency package "+pkg); } catch(error) { if(error.code!=="ENOENT") throw error; } }
const entries=[],sha=b=>createHash('sha256').update(b).digest('hex');
async function copy(input,output,pkg,relative='') {
  const info=await lstat(input);
  assert(!info.isSymbolicLink(),'No copied dependency links '+input);
  if(info.isDirectory()) {
    await mkdir(output,{recursive:true});
    for(const name of await readdir(input)) await copy(path.join(input,name),path.join(output,name),pkg,path.join(relative,name));
  } else {
    assert(info.isFile(),input);
    const bytes=await readFile(input);
    await copyFile(input,output);
    assert.equal(sha(await readFile(output)),sha(bytes));
    entries.push({package:pkg,path:relative.replaceAll('\\','/'),bytes:bytes.length,sha256:sha(bytes)});
  }
}
for(const pkg of packages) await copy(await realpath(path.join(source,pkg)),path.join(destination,pkg),pkg);
await writeFile('results/integration576/dependency-image-copy.json',JSON.stringify({source,destination,packages,files:entries.length,entries},null,2)+'\n');
console.log('Copied and SHA-verified '+entries.length+' regular files from '+packages.length+' existing packages; no install or reference link.');
