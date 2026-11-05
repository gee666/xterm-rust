// Verify this accepted batch's exact files and the minimal GDEF/avar composition.
import fs from 'node:fs';
import crypto from 'node:crypto';
import {spawnSync} from 'node:child_process';
const git=(...args)=>{
 const result=spawnSync('git',args,{encoding:null,timeout:30000,maxBuffer:16*1024*1024});
 if(result.status!==0)throw new Error(result.stderr.toString());
 return result.stdout;
};
const text=(...args)=>git(...args).toString().trim();
const hash=bytes=>crypto.createHash('sha256').update(bytes).digest('hex');
const batches=[
 {task:423,chain:['1ed6d94c','77bcf1d2','dc36569d','e7923bbd','d3e0b99a','9d4572e2']},
 {task:427,chain:['c13e7b10','b7d9aa48','e6a80015']},
 {task:428,chain:['c03ac749','2da1e625','b0b5a96e','8dc5ca38','8a0a625c','a3ef7596']},
 {task:431,chain:['b51b383d','b02c70c5','52bb3312','d74dd89a']},
 {task:433,chain:['982e272e','4dfbb989','a5aacf3c']},
 {task:434,chain:['6e43959f','538ea14f','f4d812ee','ca9a0623','71eba192']},
 {task:435,chain:['b7b1e881','92e678dc'],receiptOnly:true},
];
const font='crates/xterm-addons/src/ligatures/';
function compose(name,changes){
 let value=git('show',`71eba192:${font}${name}`).toString();
 for(const [before,after]of changes){
  if(value.split(before).length!==2)throw new Error(`Ambiguous composition in ${name}`);
  value=value.replace(before,after);
 }
 return Buffer.from(value);
}
const combined=new Map([
 [font+'font.rs',compose('font.rs',[
  ['mod font_fvar;','mod font_fvar;\n#[path = "font_gdef.rs"]\nmod font_gdef;'],
  ['        let current = refreshed.as_deref().unwrap_or(bytes);','        if let Some(table) = tables.gdef {\n            font_gdef::validate(&table, bytes, &mut refreshed, &mut warning)?;\n        }\n        let current = refreshed.as_deref().unwrap_or(bytes);'],
 ])],
 [font+'font_bytes.rs',compose('font_bytes.rs',[
  ['    KernVersion(u16),','    KernVersion(u16),\n    CoverageFormat { offset: usize },'],
  ['            Self::Invalid(reason) | Self::Type(reason) => f.write_str(reason),','            Self::CoverageFormat { offset } => {\n                write!(f, "0x{offset:x}: Coverage format must be 1 or 2.")\n            }\n            Self::Invalid(reason) | Self::Type(reason) => f.write_str(reason),'],
  ["impl<'a> DeferredTable<'a> {\n    pub fn load","impl<'a> DeferredTable<'a> {\n    pub fn offset(&self) -> Result<usize, FontError> {\n        self.record.offset32(if self.woff { 4 } else { 8 })\n    }\n\n    pub fn load"],
  ["    pub gsub: Option<DeferredTable<'a>>,","    pub gsub: Option<DeferredTable<'a>>,\n    pub gdef: Option<DeferredTable<'a>>,"],
  ['        let mut gsub = None;','        let mut gsub = None;\n        let mut gdef = None;'],
  ['b"name" | b"GSUB" | b"hmtx"','b"name" | b"GSUB" | b"GDEF" | b"hmtx"'],
  ['                        b"GSUB" => gsub = Some(record),','                        b"GSUB" => gsub = Some(record),\n                        b"GDEF" => gdef = Some(record),'],
  ['            gsub: gsub.map(|record| DeferredTable {','            gdef: gdef.map(|record| DeferredTable {\n                reader,\n                record,\n                woff,\n            }),\n            gsub: gsub.map(|record| DeferredTable {'],
 ])],
 [font+'font_warning.rs',compose('font_warning.rs',[
  ['    MultipleKernSubtables,','    MultipleKernSubtables,\n    /// The parser warns and retains an otherwise unsupported ClassDef value.\n    InvalidClassDefinition { offset: usize, format: u16 },'],
  ['            Self::AvarMissingFvar =>','            Self::InvalidClassDefinition { offset, format } => format!(\n                "0x{offset:x}: This font file uses an invalid ClassDef format of {format}. It might be corrupted and should be reacquired if it doesn\'t display as intended."\n            ).into(),\n            Self::AvarMissingFvar =>'],
 ])],
]);
const shared=new Set([...combined.keys(),'crates/xterm-addons/tests/support/font_metadata_data.rs','crates/xterm-font-addons/src/ligature_state.rs']);
const source=path=>['crates/','scripts/','.github/','.pi/swarm/skills/'].some(prefix=>path.startsWith(prefix));
const report={base:'dcde3e9ca10e21ebdfd5babddb5dbeccbb259dcb',assembledHead:text('rev-parse','HEAD'),batches:[],sourcePins:[],receiptPins:[],overlaps:[],documentationUnion:['docs/browser-serialize.md retains410/416/421 plus427 CircularList alias scope'],status:'preparing'};
const ownership=new Map();
for(const batch of batches){
 const chain=batch.chain.map(ref=>text('rev-parse',ref));
 const files=[...new Set(chain.flatMap(ref=>text('diff-tree','--root','--no-commit-id','--name-only','-r',ref).split('\n').filter(Boolean)))];
 const selected=files.filter(source),tip=chain.at(-1);
 if(batch.receiptOnly&&selected.length)throw new Error('Receipt-only chain changes source');
 report.batches.push({...batch,chain,tip,files:files.length,sourceFiles:selected.length});
 for(const file of selected){
  if(ownership.has(file))report.overlaps.push({file,first:ownership.get(file),second:batch.task,approved:shared.has(file)});
  ownership.set(file,batch.task);
  const original=git('show',`${tip}:${file}`),expected=combined.get(file)??original;
  const actual=git('show',`HEAD:${file}`),working=fs.readFileSync(file);
  const checkoutEolOnly=file.endsWith('.md')&&expected.toString().replaceAll('\r\n','\n')===working.toString().replaceAll('\r\n','\n');
  report.sourcePins.push({task:batch.task,path:file,tip,producerSha256:hash(original),expectedUnionSha256:hash(expected),headSha256:hash(actual),workingSha256:hash(working),composed:combined.has(file),checkoutEolOnly:!expected.equals(working)&&checkoutEolOnly,matches:expected.equals(actual)&&(expected.equals(working)||checkoutEolOnly)});
 }
 for(const file of files.filter(file=>file.startsWith('results/'))){
  const expected=git('show',`${tip}:${file}`),actual=git('show',`HEAD:${file}`);
  report.receiptPins.push({task:batch.task,path:file,expectedSha256:hash(expected),headSha256:hash(actual),matches:expected.equals(actual)});
 }
}
report.status=report.sourcePins.every(p=>p.matches)&&report.receiptPins.every(p=>p.matches)&&report.overlaps.every(p=>p.approved)?'passed':'failed';
fs.mkdirSync('results/integration438',{recursive:true});
if(process.argv.includes('--write'))fs.writeFileSync('results/integration438/manifest.json',JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify({status:report.status,sourcePins:report.sourcePins.length,uniqueSourceFiles:ownership.size,receipts:report.receiptPins.length,overlaps:report.overlaps,sourceMismatches:report.sourcePins.filter(p=>!p.matches),receiptMismatches:report.receiptPins.filter(p=>!p.matches)}));
if(report.status!=='passed')process.exitCode=1;
