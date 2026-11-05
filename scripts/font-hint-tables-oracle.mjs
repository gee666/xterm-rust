// The unchanged reference is an oracle only, never a runtime font dependency.
import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {constants,deflateSync} from 'node:zlib';
import {sourceLoader} from './source-ts-loader.mjs';
import {buildHintFont,hintAddonScenarios} from '../crates/xterm-font-addons/tests/support/font_hint_tables.mjs';
if (process.argv.slice(2).some(arg => arg !== '--generate')) throw new Error('Only --generate is supported');
const packageFile = process.env.SOURCE_PACKAGE_JSON ?? path.resolve('source_repo/package.json');
const require = createRequire(packageFile), dependency = require.resolve('opentype.js');
const {parse} = require('opentype.js');
const loader = sourceLoader(path.dirname(path.dirname(packageFile)));
const {LigaturesAddon} = loader.load(path.join(loader.source,'addons/addon-ligatures/src/LigaturesAddon.ts'));
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const base = new Uint8Array(await fs.readFile('crates/xterm-addons/tests/fixtures/ligature-fonts/UbuntuMono-Regular.ttf'));
const cases = [];
function add(tag, woff, name, entries, other = {}) { cases.push({name:tag + ' ' + (woff?'WOFF ':'SFNT ') + name, tag, woff, entries, ...other}); }
for (const tag of ['cvt ','fpgm','prep']) for (const woff of [false,true]) {
  for (const size of [0,1,2,3,4,15,16,63,64]) for (const length of [0,1,2,3,4,15,16,64]) {
    add(tag,woff,'bytes'+size+' length'+length,[{data:Array(size).fill(0),length}]);
  }
  for (const length of [0,1,2,3,4]) add(tag,woff,'invalid offset length'+length,[{data:[],length,offset:0xfffffff0}]);
  // An uncompressed list may read beyond its nominal descriptor into a later
  // table, but a zero-element list never reads even a wildly invalid offset.
  add(tag,woff,'later bytes satisfy read',[{data:[],length:4}],{tail:[0,0,0,0]});
  for (const first of [false,true]) {
    add(tag,woff,'duplicate bad '+(first?'first':'last'),[
      {data:[0,0,0,0],length:4,first:!first},
      {data:[],length:2,offset:0xfffffff0,first}]);
    add(tag,woff,'head precedence '+(first?'first':'last'),[
      {data:[],length:tag==='cvt '?1:2,offset:0xfffffff0,first}],{badHead:true});
  }
  if (woff) {
    const payload = [...deflateSync(Buffer.alloc(64))];
    let seed = 0x420d3f1a;
    const random = () => {seed^=seed<<13;seed^=seed>>>17;seed^=seed<<5;return seed>>>0;};
    for (const [name,data] of [['ramp',Buffer.from(Array.from({length:8192},(_,i)=>i%63))],['biased',Buffer.from(Array.from({length:8192},()=>random()%17))]]) {
      const packed=[...deflateSync(data,{strategy:constants.Z_HUFFMAN_ONLY})];
      assert.equal((packed[2]>>>1)&3,2,'Retain actual dynamic Huffman input');
      for(const length of [data.length,data.length+1]) add(tag,true,'dynamic '+name+' length'+length,[{data:packed,length}]);
    }
    const storedBlocks=[...deflateSync(Buffer.alloc(131072),{level:0})];
    for(const length of [200000,200001]) add(tag,true,'multiple stored blocks length'+length,[{data:storedBlocks,length}]);
    add(tag,true,'compressed valid',[{data:payload,length:64}]);
    add(tag,true,'compressed longer recorded length',[{data:payload,length:65}]);
    add(tag,true,'compressed shorter recorded length',[{data:payload,length:62}]);
    const badHeader = payload.slice(); badHeader[0] ^= 255;
    const badChecksum = payload.slice(); badChecksum[badChecksum.length-1] ^= 255;
    add(tag,true,'ignored zlib header',[{data:badHeader,length:64}]);
    add(tag,true,'ignored zlib checksum',[{data:badChecksum,length:64}]);
    add(tag,true,'empty deflate zero fills',[{data:[...deflateSync(Buffer.alloc(0))],length:64}]);
    add(tag,true,'streamed overflow discarded',[{data:[...deflateSync(Buffer.alloc(65536))],length:4096}]);
    for (const compressedLength of [0,1]) add(tag,true,'compressed length'+compressedLength,[{data:payload,length:64,compressedLength}]);
    add(tag,true,'compressed invalid stream',[{data:Array(12).fill(0),length:64}]);
    add(tag,true,'compressed odd length precedes array failure',[{data:Array(12).fill(0),length:65}]);
    add(tag,true,'compressed invalid offset',[{data:payload,length:64,offset:0xfffffff0}]);
    // The source stored-block loop reads missing payload as undefined and its
    // Uint8Array destination stores zero. Header failures still precede cvt's
    // fractional Array length. These inputs all terminate in the source.
    const stored = [1,4,0,251,255,0,0,0,0];
    for (let size=0;size<=stored.length;size++) for (const length of [64,65]) {
      add(tag,true,'stored truncated'+size+' length'+length,[{data:[120,156,...stored.slice(0,size)],length}]);
    }
    add(tag,true,'stored maximum missing payload',[{data:[120,156,1,255,255,0,0],length:64}]);
    add(tag,true,'stored maximum missing inverse header',[{data:[120,156,1,255,255],length:64}]);
    for (const storedLength of [0,4,65535]) for (const inverseLow of [0,1,255]) for (const length of [64,65]) {
      add(tag,true,'partial inverse '+storedLength+'/'+inverseLow+' length'+length,[{data:[120,156,1,storedLength&255,storedLength>>>8,inverseLow],length}]);
    }
    for (const length of [64,65]) {
      add(tag,true,'stored prefix partial inverse length'+length,[{data:[120,156,0,0,0,255,255,1,255,255,1],length}]);
      add(tag,true,'fixed prefix partial inverse length'+length,[{data:[120,156,2,4,255,255,1],length}]);
      const pack = bits => Array.from({length:Math.ceil(bits.length/8)},(_,i)=>bits.slice(i*8,i*8+8).reduce((n,b,j)=>n+(b<<j),0));
      for (const [name,code,extra] of [['absent history',[0,0,0,0,0,0,1],0],['reserved286',[1,1,0,0,0,1,1,0],6],['reserved287',[1,1,0,0,0,1,1,1],0]]) {
        const raw=pack([1,1,0,...code,...Array(extra+5+7).fill(0)]);
        add(tag,true,name+' length'+length,[{data:[120,156,...raw],length}]);
      }
    }
  }
}
for (const c of cases) {
  const bytes = buildHintFont(base,c);
  c.inputSha256 = hash(bytes);
  try {parse(bytes.buffer); c.outcome = {status:'ok'};}
  catch(error) {c.outcome = {status:'error',name:error.name,message:error.message};}
}
const addon = await hintAddonScenarios(options => new LigaturesAddon(options),base,cases);
for (let i=0;i<cases.length;i++) {
  const accepted = cases[i].outcome.status === 'ok';
  assert.deepEqual(addon[i],{name:cases[i].name,loading:[[0,3]],ranges:accepted?[]:[[0,3]],reads:1,refreshes:accepted?1:0});
}
assert(cases.some(c => c.outcome.message === 'Invalid array length'));
assert(cases.some(c => c.outcome.status === 'ok'));
assert(cases.some(c => c.outcome.message === 'Data error'));
const nontermination = {name:'empty dynamic Huffman trees',tag:'prep',woff:true,entries:[{data:[120,156,5,0,0,0],length:64}]};
nontermination.inputSha256 = hash(buildHintFont(base,nontermination));
const fixture = {sourceSha256:hash(await fs.readFile(dependency)),fontSha256:hash(base),cases,addon,nontermination};
const text = JSON.stringify(fixture)+'\n', destination = 'crates/xterm-addons/tests/fixtures/font-hint-tables.json';
if (process.argv.includes('--generate')) await fs.writeFile(destination,text);
else assert.equal(await fs.readFile(destination,'utf8'),text,'Font hint corpus drift');
await fs.mkdir('results/font-hint-tables',{recursive:true});
const report = {node:process.version,sourceSha256:fixture.sourceSha256,fontSha256:fixture.fontSha256,
  sourceHashes:Object.fromEntries(loader.hashes),fixtureSha256:hash(text),cases:cases.length,
  accepted:cases.filter(c=>c.outcome.status==='ok').length,rejected:cases.filter(c=>c.outcome.status==='error').length,
  addonCases:addon.length,scope:'Unchanged source constructor and recorded addon boundary, not full terminal acceptance'};
await fs.writeFile('results/font-hint-tables/oracle.json',JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(report));
