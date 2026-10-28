// Real installed QOI WASM codec versus the native Rust memory/command decoder.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {spawnSync} from 'node:child_process';
const require=createRequire(import.meta.url),Decoder=require('../source_repo/node_modules/xterm-wasm-parts/lib/qoi/QoiDecoder.wasm.js').default;
const Encoder=require('../source_repo/node_modules/xterm-wasm-parts/lib/qoi/QoiEncoder.wasm.js').default;
const root=path.resolve(import.meta.dirname,'..');const output=path.join(root,'results/qoi');fs.mkdirSync(output,{recursive:true});
const paletteBytes=fs.readFileSync(path.join(root,'source_repo/addons/addon-image/fixture/palette.blob'));
const paletteEncoded=Uint8Array.from(new Encoder(1048576).encode(paletteBytes,640,80));
const paletteDirectory=path.join(root,'crates/xterm-addons/tests/fixtures/qoi');
if(process.argv.includes('--generate')){fs.mkdirSync(paletteDirectory,{recursive:true});fs.writeFileSync(path.join(paletteDirectory,'palette.qoi'),paletteEncoded);fs.writeFileSync(path.join(paletteDirectory,'palette.rgba'),paletteBytes);}
assert.deepEqual(fs.readFileSync(path.join(paletteDirectory,'palette.qoi')),Buffer.from(paletteEncoded));
assert.deepEqual(fs.readFileSync(path.join(paletteDirectory,'palette.rgba')),paletteBytes);
let decoder=new Decoder(4194304);const cases=[];
function runs(data){const result=[];for(let offset=0;offset+3<data.length;offset+=4){const word=(data[offset]|data[offset+1]<<8|data[offset+2]<<16|data[offset+3]<<24)>>>0;const last=result.at(-1);if(last&&last[0]===word)last[1]++;else result.push([word,1]);}return result;}
function record(operation){let pixels=[],error=null;try{if(operation.op==='new')decoder=new Decoder(operation.keep);else if(operation.op==='release')decoder.release();else{const data=operation.file==='palette.qoi'?paletteEncoded:operation.file?fs.readFileSync(path.join(root,'source_repo/addons/addon-image/fixture/testimages',operation.file)):Uint8Array.from(operation.data);pixels=runs(decoder.decode(data));}}catch(cause){error=cause.name;}cases.push({operation,expected:{width:decoder.width,height:decoder.height,memory:decoder._mem?.buffer.byteLength??0,initialized:!!decoder._inst,pixels,error}});}
function image(width,height,body=[],magic='qoif'){const bytes=Buffer.alloc(14+body.length+8);bytes.write(magic,0);bytes.writeInt32BE(width,4);bytes.writeInt32BE(height,8);bytes[12]=4;bytes.set(body,14);bytes[bytes.length-1]=1;return Array.from(bytes);}
record({op:'new',keep:4194304});record({op:'decode',file:'dice.qoi'});record({op:'decode',file:'palette.qoi'});
for(const keep of[0,65535,65536,4194304]){
 record({op:'new',keep});
 for(const data of[[],image(0,0),image(1,1,[0xff,1,2,3,4]),image(2,1,[0xfe,255,0,0,0xc0]),image(2,1),image(-1,1),image(-1,512),image(-2100,1),image(-100000,1),image(1,-3),image(-3,-3),image(500,0,new Array(512).fill(0))]){record({op:'decode',data});record({op:'release'});}
}
for(let opcode=0;opcode<256;opcode++){
 record({op:'new',keep:4194304});record({op:'decode',data:image(64,2,[0xff,20,30,40,50,opcode,0x88,0x99,0xaa,0xbb])});
}
const full=image(2,2,[0xff,1,2,3,4,0xfe,5,6,7,0x40,0x80,0x12]);
record({op:'new',keep:4194304});for(let length=0;length<=full.length;length++)record({op:'decode',data:full.slice(0,length)});
record({op:'decode',data:image(1,1,new Array(70000).fill(0))});record({op:'release'});
const seed=0x90a12026;let state=seed;const random=n=>{state^=state<<13;state^=state>>>17;state^=state<<5;return(state>>>0)%n;};
for(let index=0;index<500;index++){
 if(index%7===0)record({op:'new',keep:[0,65535,65536,4194304][random(4)]});
 const width=random(21)-4,height=random(21)-4,body=Array.from({length:random(512)},()=>random(256));
 record({op:'decode',data:image(width,height,body,index%3?'qoif':'junk')});if(random(5)===0)record({op:'release'});
}
const fixture=path.join(root,'crates/xterm-addons/tests/fixtures/qoi-source.json');
if(process.argv.includes('--generate'))fs.writeFileSync(fixture,JSON.stringify({seed,cases})+'\n');
else assert.deepEqual(JSON.parse(fs.readFileSync(fixture,'utf8')),{seed,cases},'reference fixture drift');
const build=spawnSync('cargo',['build','--locked','-p','xterm-addons','--example','qoi_trace'],{cwd:root,encoding:'utf8',timeout:120000});assert.equal(build.status,0,build.stderr||build.error?.message);
const exe=path.join(root,'target/debug/examples/qoi_trace'+(process.platform==='win32'?'.exe':''));
const run=spawnSync(exe,[],{cwd:root,input:cases.map(c=>JSON.stringify(c.operation)).join('\n')+'\n',encoding:'utf8',timeout:60000,maxBuffer:64*1024*1024});assert.equal(run.status,0,run.stderr||run.error?.message);
const actual=run.stdout.trimEnd().split(/\r?\n/).map(line=>JSON.parse(line));assert.equal(actual.length,cases.length);
for(let index=0;index<cases.length;index++)try{assert.deepEqual(actual[index],cases[index].expected);}catch{fs.writeFileSync(path.join(output,'counterexample.json'),JSON.stringify({seed,index,operation:cases[index].operation,expected:cases[index].expected,actual:actual[index]},null,2));throw Error(`QOI mismatch ${index}`);}
const summary={cases:cases.length,seed,mismatches:0,scope:'Native QOI pixels/memory only; browser ImageData and IIP host remain integration work'};fs.writeFileSync(path.join(output,'summary.json'),JSON.stringify(summary,null,2)+'\n');console.log(summary);
