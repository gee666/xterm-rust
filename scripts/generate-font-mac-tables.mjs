// Derive only the source cmap-format0 decoding data, not executable shaping.
import fs from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
const sourcePath=process.env.OPENTYPE_SOURCE??'source_repo/node_modules/opentype.js/dist/opentype.mjs';
const source=await fs.readFile(sourcePath,'utf8');
function table(name) {
 const start=source.indexOf('var '+name+' = ');
 if(start<0)throw new Error('Missing source table '+name);
 const end=source.indexOf('\n};',start);
 return new Function(source.slice(start,end+3)+';return '+name+';')();
}
const maps=table('eightBitMacEncodings');
const languages=table('macLanguageEncodings');
const scripts=table('macScriptEncodings');
const identifier=name=>name.toUpperCase().replaceAll('-','_');
let output='// Generated from opentype.js decoding data. See licenses/opentype-js-MIT.txt.\n';
output+='// Source SHA256 '+createHash('sha256').update(source).digest('hex')+'\n';
output+='pub fn mac_encoding(platform: u16, encoding: u16, language: i32) -> Option<&\'static [u16]> {\n';
output+='    if platform != 1 { return None; }\n    match language {\n';
for(const [id,name]of Object.entries(languages))if(maps[name])output+=`        ${id} => Some(&${identifier(name)}),\n`;
output+='        _ => match encoding {\n';
for(const [id,name]of Object.entries(scripts))if(maps[name])output+=`            ${id} => Some(&${identifier(name)}),\n`;
output+='            _ => None,\n        },\n    }\n}\n';
for(const [name,text]of Object.entries(maps)) {
 output+=`const ${identifier(name)}: [u16; ${text.length}] = [\n`;
 for(let i=0;i<text.length;i+=8)output+='    '+Array.from({length:Math.min(8,text.length-i)},(_,j)=>'0x'+text.charCodeAt(i+j).toString(16).padStart(4,'0')).join(', ')+',\n';
 output+='];\n';
}
const formatted=spawnSync(process.env.RUSTFMT??'rustfmt',['--edition','2021'],{input:output,encoding:'utf8',timeout:10000});
if(formatted.error)throw formatted.error;
if(formatted.status!==0)throw new Error('rustfmt failed: '+formatted.stderr);
output=formatted.stdout;
const target='crates/xterm-addons/src/ligatures/font_mac_tables.rs';
if(process.argv.includes('--generate'))await fs.writeFile(target,output);
else if(await fs.readFile(target,'utf8')!==output)throw new Error('Font Mac decoding table drift; inspect before --generate');
// Name strings are decoded only for recognized languages. Keep membership data
// separate from text decoding; validation does not need translated names.
let names='// Language membership from opentype.js. See licenses/opentype-js-MIT.txt.\n';
names+='// Source SHA256 '+createHash('sha256').update(source).digest('hex')+'\n';
for(const [name,key]of [['mac','macLanguages'],['windows','windowsLanguages']]) {
 const ids=Object.keys(table(key)).map(Number).sort((a,b)=>a-b);
 names+=`pub fn ${name}(id: u16) -> bool { ${name.toUpperCase()}.binary_search(&id).is_ok() }\n`;
 names+=`const ${name.toUpperCase()}: &[u16] = &[${ids.join(', ')}];\n`;
}
const nameOutput=spawnSync(process.env.RUSTFMT??'rustfmt',['--edition','2021'],{input:names,encoding:'utf8',timeout:10000});
if(nameOutput.error||nameOutput.status!==0)throw new Error('Name language rustfmt failed: '+nameOutput.stderr);
const nameTarget='crates/xterm-addons/src/ligatures/font_name_languages.rs';
if(process.argv.includes('--generate'))await fs.writeFile(nameTarget,nameOutput.stdout);
else if(await fs.readFile(nameTarget,'utf8')!==nameOutput.stdout)throw new Error('Font name language drift; inspect before --generate');
console.log(JSON.stringify({source:sourcePath,sourceSha256:createHash('sha256').update(source).digest('hex'),target,nameTarget,tables:Object.keys(maps).length,entries:Object.values(maps).reduce((sum,value)=>sum+value.length,0)}));
