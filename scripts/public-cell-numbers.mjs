// Execute untouched BufferLine/CellData methods for JavaScript numeric indices.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
import { captureRustInputs, changedInputs, sha256 } from './public-api-evidence.mjs';
process.chdir(resolve(import.meta.dirname,'..'));
const require=createRequire(resolve('source_repo/package.json'));
const {build}=require('esbuild');
mkdirSync('tmp/public-cell-numbers',{recursive:true});
mkdirSync('results/public-cell-numbers',{recursive:true});
const attempt=String(Date.now());
const summaryPath='results/public-cell-numbers/summary.json';
writeFileSync(summaryPath,JSON.stringify({status:'running',attempt})+'\n');
const bundle=await build({stdin:{contents:`
export {BufferLine} from './source_repo/src/common/buffer/BufferLine';
export {CellData} from './source_repo/src/common/buffer/CellData';
export {ExtendedAttrs} from './source_repo/src/common/buffer/AttributeData';
export * from './source_repo/src/common/buffer/Constants';`,resolveDir:process.cwd(),loader:'ts'},
  outfile:'tmp/public-cell-numbers/source.cjs',bundle:true,platform:'node',format:'cjs',target:'node24',
  tsconfig:'source_repo/src/headless/tsconfig.json',metafile:true,logLevel:'silent'});
const sourceHashes=Object.fromEntries(Object.keys(bundle.metafile.inputs).filter(path=>path!=='<stdin>').sort().map(path=>[path,sha256(path)]));
const {BufferLine,CellData,ExtendedAttrs,Content,FgFlags,BgFlags,Attributes}=require(resolve('tmp/public-cell-numbers/source.cjs'));
const line=new BufferLine(5);
for(const [i,cp,width,fg,bg] of [
  [0,65,1,Content.IS_COMBINED_MASK|65,0],
  [1,Content.IS_COMBINED_MASK,1,FgFlags.UNDERLINE|Attributes.CM_RGB|0x112233,BgFlags.HAS_EXTENDED],
  [2,0x6587,2,0,0],[3,0,0,0,0],[4,0,1,0,0]
]) {
  const cell=new CellData();
  cell.content=cp|(width<<Content.WIDTH_SHIFT);cell.fg=fg;cell.bg=bg;
  cell.combinedData=i===1?'e\u0301':'';
  cell.extended=new ExtendedAttrs((3<<26)|Attributes.CM_RGB|0xabcdef,17);
  line.setCell(i,cell);
}
const methods=['getWidth','getFgColorMode','getBgColorMode','getFgColor','getBgColor','isBold','isItalic','isDim',
  'isUnderline','isBlink','isInverse','isInvisible','isStrikethrough','isOverline','isFgRGB','isFgPalette','isFgDefault',
  'isBgRGB','isBgPalette','isBgDefault','isAttributeDefault','getUnderlineStyle','getUnderlineColor','getUnderlineColorMode',
  'isUnderlineColorRGB','isUnderlineColorPalette','isUnderlineColorDefault'];
const scalar=value=>value===undefined?'u':typeof value==='boolean'?(value?'t':'f'):Number.isNaN(value)?'nan':String(value);
function call(callback){try{return scalar(callback());}catch(error){return `error:${error.name}`;}}
function chars(value){return value===undefined?'u':Array.from({length:value.length},(_,i)=>value.charCodeAt(i).toString(16)).join(',')||'-';}
const bits=new DataView(new ArrayBuffer(8));
const encode=value=>{bits.setFloat64(0,value,false);return bits.getBigUint64(0,false).toString(16).padStart(16,'0');};
const values=[-Infinity,-Number.MAX_VALUE,-1,-Number.MIN_VALUE,-0,0,Number.MIN_VALUE,NaN,1/3,2/3,0.1,0.5,1,1.5,2,3,4,4+1/3,4+2/3,4.999,5,Infinity,Number.MAX_VALUE];
const seed=0x4e554d43;let state=seed;
function random(){state^=state<<13;state^=state>>>17;state^=state<<5;return state>>>0;}
for(let i=0;i<2048;i++){bits.setUint32(0,random(),false);bits.setUint32(4,random(),false);values.push(bits.getFloat64(0,false));}
const rows=values.map(index=>{
  const cell=line.getCell(index);
  if(!cell)return `${encode(index)}\tnone`;
  const raw=[cell.content,cell.fg,cell.bg].map(scalar);
  raw.push(cell.extended===undefined?'u':`${cell.extended.ext},${cell.extended.urlId}`);
  raw.push(chars(cell.getChars()),call(()=>cell.getCode()));
  raw.push(...methods.map(method=>call(()=>cell[method]())));
  raw.push(call(()=>cell.attributesEquals(cell)),call(()=>cell.attributesEquals(new CellData())));
  return `${encode(index)}\t${raw.join('|')}`;
});
const fixture='crates/xterm-core/tests/fixtures/public-cell-numbers.tsv';
const fixtureText=rows.join('\n')+'\n';
if(process.argv.includes('--generate'))writeFileSync(fixture,fixtureText);
else assert.equal(readFileSync(fixture,'utf8'),fixtureText,'numeric cell fixture drift');
const translationRows=[];
for(const start of [undefined,-2,-0,0,1/3,2/3,0.5,1,1.5,2,3,4,5,NaN,Infinity])
for(const end of [undefined,-1,0,1/3,1,2,3,5,7,NaN,-Infinity])
for(const trim of [false,true])for(const withColumns of [false,true]) {
  const columns=[];let outcome;
  try { const text=line.translateToString(trim,start,end,withColumns?columns:undefined);
    outcome=`${chars(text)}|${withColumns?columns.map(encode).join(','):'-'}`;
  } catch(error) {outcome=`error:${error.name}`;}
  translationRows.push(`${start===undefined?'u':encode(start)}\t${end===undefined?'u':encode(end)}\t${trim}\t${withColumns}\t${outcome}`);
}
const translationFixture='crates/xterm-core/tests/fixtures/public-line-number-translations.tsv';
const translationText=translationRows.join('\n')+'\n';
if(process.argv.includes('--generate'))writeFileSync(translationFixture,translationText);
else assert.equal(readFileSync(translationFixture,'utf8'),translationText,'numeric translation fixture drift');
// Directed host callbacks are checked after the fixtures so their row mutation
// cannot silently change expected data used by the native snapshot tests.
const replacement=new CellData();replacement.content=65|(1<<Content.WIDTH_SHIFT);replacement.fg=456;
const mutating={set content(value){this.saved=value;line.setCell(0,replacement);},get content(){return this.saved;}};
assert.equal(line.getCell(0,mutating),mutating);assert.equal(mutating.fg,456);
const overridden={set content(value){},get content(){return Content.IS_COMBINED_MASK;}};
line.getCell(0,overridden);assert.equal(overridden.combinedData,undefined);
let missingMethodError;
try {line.getCell(0).attributesEquals({});}catch(error){missingMethodError=error.name;}
assert.equal(missingMethodError,'TypeError');
const sourceDestinationChecks={setterMutationFg:mutating.fg,overriddenContentHasUndefinedCombined:overridden.combinedData===undefined,missingMethodError};
const extras=['crates/xterm-core/tests/public_cell_numbers.rs','crates/xterm-core/tests/public_cell_numbers_differential.rs',fixture,translationFixture];
const inputs=captureRustInputs(extras);
const cargo=process.env.CARGO??(process.platform==='win32'?`${process.env.USERPROFILE}/.cargo/bin/cargo.exe`:'cargo');
const run=spawnSync(cargo,['test','-p','xterm-core','--test','public_cell_numbers','--test','public_cell_numbers_differential'],{encoding:'utf8',timeout:180000,maxBuffer:8*1024*1024});
for(const prefix of ['cargo',attempt])for(const stream of ['stdout','stderr'])writeFileSync(`results/public-cell-numbers/${prefix}.${stream}.log`,run[stream]??'');
const changes=changedInputs(inputs,captureRustInputs(extras));
const summary={status:changes.length?'changed-during-run':run.status===0?'passed':'failed',attempt,seed,cases:rows.length,translationCases:translationRows.length,
  sourceHashes,sourceDestinationChecks,rustInputs:inputs,changedDuringRun:changes,fixtureSha256:sha256(fixture),translationFixtureSha256:sha256(translationFixture),node:process.version,
  platform:`${process.platform}-${process.arch}`,exitCode:run.status,error:run.error?.message??null,
  scope:'raw numeric getCell and every exposed cell getter; source line is populated via ordinary integer writes',
  unrun:['real browser numeric boundary','arbitrary target object','nonterminating translateToString inputs','non-numeric coercions']};
for(const [path,hash]of Object.entries(sourceHashes))assert.equal(sha256(path),hash);
for(const path of [summaryPath,`results/public-cell-numbers/${attempt}.json`])writeFileSync(path,JSON.stringify(summary,null,2)+'\n');
console.log(JSON.stringify({...summary,rustInputs:undefined},null,2));
process.exitCode=summary.status==='passed'?0:1;
