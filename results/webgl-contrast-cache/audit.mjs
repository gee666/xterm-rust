// Read-only reference, unchanged-assertion, source and retained-evidence audit.
import {readFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
const root=fileURLToPath(new URL('../../',import.meta.url));
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const reference=resolve(process.env.SOURCE_ROOT??resolve(root,'source_repo'));
const manifest=JSON.parse(await readFile(resolve(root,'docs/baseline/reference-manifest.json'),'utf8'));
if(manifest.length!==776)throw Error('Unexpected reference inventory');
for(const entry of manifest) {
  const bytes=await readFile(resolve(reference,entry.path));
  if(bytes.length!==entry.bytes||hash(bytes)!==entry.sha256)throw Error('Reference changed: '+entry.path);
}
const fixture='crates/xterm-web/tests/fixtures/webgl-atlas-colors.json';
const old=JSON.parse(execFileSync(process.env.GIT??'git',['show','3a4f5e8f:'+fixture],{cwd:root,encoding:'utf8',timeout:30000}));
const current=JSON.parse(await readFile(resolve(root,fixture),'utf8'));
if(current.cases.length!==476||JSON.stringify(old.cases)!==JSON.stringify(current.cases.slice(0,452)))
  throw Error('Original452 assertions changed or new24 absent');
for(const entry of current.inputs)if(hash(await readFile(resolve(reference,entry.path)))!==entry.sha256)
  throw Error('Native oracle input changed: '+entry.path);
const probe=JSON.parse(await readFile(resolve(root,'results/webgl-contrast-cache/source-probe.json'),'utf8'));
for(const entry of probe.inputs)if(hash(await readFile(resolve(reference,entry.path)))!==entry.sha256)
  throw Error('Browser oracle input changed: '+entry.path);
if(hash(await readFile(resolve(root,probe.wrapper.path)))!==probe.wrapper.sha256)
  throw Error('Browser oracle wrapper changed');
const checkpoint=JSON.parse(await readFile(resolve(root,'results/webgl-contrast-cache/checkpoint.json'),'utf8'));
for(const entry of checkpoint.pins)if(hash(await readFile(resolve(root,entry.path)))!==entry.sha256)
  throw Error('Pin mismatch: '+entry.path);
for(const name of ['native-negative.json','negative-control.json']) {
  const receipt=JSON.parse(await readFile(resolve(root,'results/webgl-contrast-cache',name),'utf8'));
  for(const mutation of receipt.mutations) {
    if(mutation.before!==mutation.restored||hash(await readFile(resolve(root,mutation.path)))!==mutation.restored)
      throw Error('Mutation not restored: '+mutation.path);
  }
}
if(process.argv.includes('--browser-artifacts'))for(const entry of checkpoint.browserArtifacts)
  if(hash(await readFile(entry.path))!==entry.sha256)throw Error('Browser artifact changed: '+entry.path);
if(process.argv.includes('--native-artifacts'))for(const entry of checkpoint.nativeArtifacts)
  if(hash(await readFile(entry.path))!==entry.sha256)throw Error('Native artifact changed: '+entry.path);
console.log(`PASS776 unchanged reference hashes,452 original assertions+24 additions,${current.inputs.length} native+${probe.inputs.length} browser oracle inputs,${checkpoint.pins.length} current pins,exact mutant restoration`+
  (process.argv.includes('--browser-artifacts')?`,${checkpoint.browserArtifacts.length} actual browser artifact pins`:'')+
  (process.argv.includes('--native-artifacts')?`,${checkpoint.nativeArtifacts.length} native executable pins`:''));
