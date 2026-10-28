// Build a WOFF1 test container from the MIT-selected Monoid fixture and run the
// actual reference font engine. No source tables or glyph data are modified.
import fs from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {deflateSync} from 'node:zlib';
const root=process.cwd(),generate=process.argv.slice(2).includes('--generate');
if(process.argv.slice(2).some(arg=>arg!=='--generate'))throw new Error('Only --generate is supported');
const source='source_repo/addons/addon-ligatures/fonts/Monoid-Regular.ttf';
const bytes=await fs.readFile(source),count=bytes.readUInt16BE(4),align=value=>(value+3)&~3;
const directory=Buffer.alloc(44+count*20),parts=[];let offset=align(directory.length),sfntSize=12+count*16;
for(let index=0;index<count;index++){
 const record=12+index*16,position=44+index*20,original=bytes.subarray(bytes.readUInt32BE(record+8),bytes.readUInt32BE(record+8)+bytes.readUInt32BE(record+12));
 const compressed=deflateSync(original),data=compressed.length<original.length?compressed:original;
 bytes.copy(directory,position,record,record+4);directory.writeUInt32BE(offset,position+4);directory.writeUInt32BE(data.length,position+8);directory.writeUInt32BE(original.length,position+12);directory.writeUInt32BE(bytes.readUInt32BE(record+4),position+16);
 parts.push(data,Buffer.alloc(align(data.length)-data.length));offset+=align(data.length);sfntSize+=align(original.length);
}
directory.write('wOFF',0,'ascii');bytes.copy(directory,4,0,4);directory.writeUInt32BE(offset,8);directory.writeUInt16BE(count,12);directory.writeUInt32BE(sfntSize,16);directory.writeUInt16BE(1,20);
const woff=Buffer.concat([directory,Buffer.alloc(align(directory.length)-directory.length),...parts]);
const report=JSON.parse(await fs.readFile('results/font-addons/ligature-oracle.json','utf8'));
for(const[file,hash]of Object.entries(report.hashes)){const actual=createHash('sha256').update(await fs.readFile(file.replaceAll('\\','/'))).digest('hex');if(actual!==hash)throw new Error('Reference changed; rerun ligature-font-oracle first: '+file)}
const {loadBuffer}=await import(pathToFileURL(path.join(root,'tmp/ligature-font-oracle/reference-index.ts')));
const font=loadBuffer(woff.buffer.slice(woff.byteOffset,woff.byteOffset+woff.byteLength));
const cases=['abc','==>','===','😀==','¯\\_(ツ)_/¯','\uD800=='].map(input=>{const result=font.findLigatures(input);return {input:Array.from({length:input.length},(_,index)=>input.charCodeAt(index)),inputGlyphs:result.inputGlyphs,outputGlyphs:result.outputGlyphs,ranges:result.contextRanges,rangeOnly:font.findLigatureRanges(input)}});
const fixture='crates/xterm-addons/tests/fixtures/ligature-fonts/Monoid-Regular.woff',expected='crates/xterm-addons/tests/fixtures/ligature-woff.json',text=JSON.stringify(cases)+'\n';
if(generate){await fs.writeFile(fixture,woff);await fs.writeFile(expected,text)}else{if(!(await fs.readFile(fixture)).equals(woff))throw new Error('WOFF fixture drift');if(await fs.readFile(expected,'utf8')!==text)throw new Error('WOFF expected trace drift')}
const summary={node:process.version,source,sourceSha256:createHash('sha256').update(bytes).digest('hex'),woffSha256:createHash('sha256').update(woff).digest('hex'),bytes:woff.length,cases:cases.length,sourceEngine:'actual supplied fontLigatures plus opentype.js',mode:generate?'generate':'verify'};
await fs.writeFile('results/font-addons/woff-oracle.json',JSON.stringify(summary,null,2)+'\n');console.log(JSON.stringify(summary));
