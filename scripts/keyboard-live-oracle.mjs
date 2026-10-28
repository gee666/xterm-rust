// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Reuses verify-keyboard's unchanged-source TS erasure, without building Rust.
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync,mkdirSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {stripTypeScriptTypes} from 'node:module';
import {resolve} from 'node:path';
import vm from 'node:vm';
const root=resolve(import.meta.dirname,'..');
assert.equal(process.versions.unicode,'17.0','live case data requires the pinned Unicode17 oracle');
const hashes={};
function read(file) {const text=readFileSync(resolve(root,file),'utf8');hashes[file]=createHash('sha256').update(text).digest('hex');return text;}
function erase(source) {return stripTypeScriptTypes(source.replace(/^import .*;\r?$/gm,'').replace(/^export /gm,''),{mode:'transform'});}
const typeEnum=read('source_repo/src/common/Types.ts').match(/export const enum KeyboardResultType\s*\{[^}]*\}/)?.[0];
assert.ok(typeEnum);
const context=vm.createContext({});
vm.runInContext(erase(read('source_repo/src/common/data/EscapeSequences.ts'))+'\n'+erase(typeEnum)+'\n'+erase(read('source_repo/src/common/input/Keyboard.ts')),context);
const evaluate=context.evaluateKeyboardEvent;
const fields=['keyCode','key','code','shiftKey','altKey','ctrlKey','metaKey'];
const letters=['n','k','d','s','a','c','m'];
const units=text=>Array.from({length:text.length},(_,i)=>text.charCodeAt(i).toString(16)).join(',')||'-';
const number=value=>Number.isNaN(value)?'NaN':value===Infinity?'Infinity':value===-Infinity?'-Infinity':Object.is(value,-0)?'-0':String(value);
function make(name,mask=0,options=0,changes={}) {
  return {name,options,fields:{keyCode:[255],key:['x'],code:[''],shiftKey:[!!(mask&1)],altKey:[!!(mask&2)],ctrlKey:[!!(mask&4)],metaKey:[!!(mask&8)],...changes}};
}
const inputs=[];
const keys=['','x','A','c','/','@','^','_','é','中','😀','\ud800','\udfff','Dead','Enter'];
const codes=['','KeyN','Keyİ','Key😀','Key','Minus','Digit2','Digit6'];
const special=[0,8,9,13,27,33,34,35,36,37,38,39,40,45,46,48,50,51,55,56,65,90,112,115,116,123,124,186,219,220,221,222,255];
for(const code of special) for(let mask=0;mask<16;mask++) for(const options of [0,2,6,7]) {
  inputs.push(make(`static-${code}-${mask}-${options}`,mask,options,{keyCode:[code],key:[keys[(code+mask)%keys.length]],code:[codes[(code+mask)%codes.length]]}));
}
for(const key of ['UIKeyInputUpArrow','UIKeyInputLeftArrow','UIKeyInputRightArrow','UIKeyInputDownArrow']) {
  for(let mask=0;mask<16;mask++) for(let options=0;options<8;options++) inputs.push(make(`mobile-${key}-${mask}-${options}`,mask,options,{keyCode:[0],key:[key]}));
}
const numbers=[NaN,Infinity,-Infinity,-0,-1,0,47.9,48.1,50.5,51.5,55.9,56,64.9,65.5,89.9,90.1,255,65535,65536,9007199254740992,1e100];
for(let index=0;index<numbers.length;index++) for(let mask=0;mask<16;mask++) {
  inputs.push(make(`numeric-${index}-${mask}`,mask,0,{keyCode:[numbers[index]],key:['/'],code:['Minus']}));
}
for(let index=0;index<numbers.length;index++) {
  inputs.push(make(`changing-control-${index}`,4,0,{keyCode:[255,65,90,numbers[index]+64]}));
}
inputs.push(make('changing-mobile',0,0,{keyCode:[0],key:['?','?','UIKeyInputRightArrow']}));
inputs.push(make('changing-printable',0,0,{key:['x','y','changed😀']}));
inputs.push(make('changing-printable-empty',0,0,{key:['x','y','']}));
inputs.push(make('changing-dead-code',2,0,{key:['Dead'],code:['KeyN','abcİ']}));
inputs.push(make('changing-dead-code-empty',2,0,{key:['Dead'],code:['KeyN','']}));
inputs.push(make('changing-modifiers',0,2,{ctrlKey:[false,true],shiftKey:[false,true],altKey:[true,false],metaKey:[true,false]}));
// Every changed BMP uppercase mapping is exercised through a changing final
// Alt-letter keyCode, not only through a table-text comparison.
for(let unit=0;unit<65536;unit++) {
  const key=String.fromCharCode(unit);
  if(key.toUpperCase()!==key || [0,0xd800,0xdfff,0xffff].includes(unit)) {
    inputs.push(make(`changing-uppercase-${unit.toString(16)}`,3,0,{keyCode:[255,255,65,90,unit-32]}));
  }
}
const seed=0x4c495645;let state=seed;
function random(){state^=state<<13;state^=state>>>17;state^=state<<5;return state>>>0;}
for(let index=0;index<1024;index++) {
  const bools=()=>Array.from({length:4},()=>!!(random()&1));
  inputs.push(make(`seeded-${index}`,0,random()&7,{keyCode:Array.from({length:7},()=>numbers[random()%numbers.length]),
    key:Array.from({length:3},()=>keys[random()%keys.length]),code:Array.from({length:3},()=>codes[random()%codes.length]),
    shiftKey:bools(),altKey:bools(),ctrlKey:bools(),metaKey:bools()}));
}
function execute(input,throwAt) {
  const reads=[];const counts={};const event={};const marker=new Error('exact getter marker');
  for(let index=0;index<fields.length;index++) {
    const field=fields[index];counts[field]=0;
    Object.defineProperty(event,field,{get(){reads.push(letters[index]);if(reads.length===throwAt) throw marker;
      const values=input.fields[field];const position=counts[field]++;return values[Math.min(position,values.length-1)];}});
  }
  Object.defineProperty(event,'type',{get(){throw new Error('unused type getter was read');}});
  let result;
  try {result=evaluate(event,!!(input.options&1),!!(input.options&2),!!(input.options&4));}
  catch(error) {assert.equal(error,marker,`unexpected error in ${input.name}`);return {reads:reads.join(''),type:'throw',cancel:0,payload:'!'};}
  assert.equal(throwAt,0,`throw boundary ${throwAt} was not reached in ${input.name}`);
  return {reads:reads.join(''),type:result.type,cancel:Number(result.cancel),payload:result.key===undefined?'!':units(result.key)};
}
function row(input,throwAt,result) {
  const f=input.fields;
  return [input.name,input.options,throwAt,f.keyCode.map(number).join(';'),f.key.map(units).join(';'),f.code.map(units).join(';'),
    f.shiftKey.map(Number).join(''),f.altKey.map(Number).join(''),f.ctrlKey.map(Number).join(''),f.metaKey.map(Number).join(''),
    result.reads,result.type,result.cancel,result.payload].join('\t');
}
const rows=[];let failures=0;let getterReads=0;
for(const input of inputs) {
  const result=execute(input,0);rows.push(row(input,0,result));getterReads+=result.reads.length;
  // Exhaust failure boundaries for all branch/mutation families. Uppercase data
  // rows repeat the same getter route, so inject on one representative only.
  if(!input.name.startsWith('changing-uppercase-') || input.name==='changing-uppercase-df') {
    for(let index=1;index<=result.reads.length;index++) {
      const failed=execute(input,index);assert.equal(failed.type,'throw');assert.equal(failed.reads,result.reads.slice(0,index));
      rows.push(row(input,index,failed));failures++;getterReads+=failed.reads.length;
    }
  }
}
const fixture=rows.join('\n')+'\n';const path=resolve(root,'crates/xterm-core/tests/fixtures/keyboard-live.tsv');
if(process.argv.includes('--generate')) writeFileSync(path,fixture);
else assert.equal(readFileSync(path,'utf8'),fixture,'live getter fixture drift; review before --generate');
mkdirSync(resolve(root,'results/keyboard-live'),{recursive:true});
const report={command:'node scripts/keyboard-live-oracle.mjs',node:process.version,unicode:process.versions.unicode,platform:process.platform,
  sourceHashes:hashes,seed,successfulCases:inputs.length,throwBoundaryCases:failures,totalCases:rows.length,getterReads,
  fixtureSha256:createHash('sha256').update(fixture).digest('hex'),scope:'Original JS live evaluator only; Rust replay and actual browser DOM integration are separate gates.'};
writeFileSync(resolve(root,'results/keyboard-live/oracle.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report,null,2));
