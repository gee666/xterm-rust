// Reuse the exact seven physical packages needed by accepted537's original suites.
import assert from 'node:assert/strict';
import {readFile,writeFile,mkdir,lstat,readdir,copyFile,realpath} from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
const source='P:/workshop/Windows_Worshop/oira666/xterm-rust/source_repo/node_modules';
const destination=path.resolve('source_repo/node_modules');
const packages=['playwright','playwright-core','@playwright/test','esbuild','@esbuild/win32-x64','@lunapaint/png-codec','pako'];
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
await writeFile('results/integration554/dependency-copy.json',JSON.stringify({source,destination,packages,files:entries.length,entries},null,2)+'\n');
console.log('Copied and SHA-verified '+entries.length+' regular files from existing seven packages; no install or reference link.');
