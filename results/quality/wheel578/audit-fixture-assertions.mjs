import fs from 'node:fs';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
const root='crates/xterm-web/tests/', output='results/quality/wheel578';
const compact=text=>Array.from(text).filter(character=>character.trim()!=='').join('');
function literals(text){
 const values=[];
 for(let index=0;index<text.length;index++){
  if(text[index]!=='"')continue;
  const start=index;
  for(index++;index<text.length;index++){
   if(text.charCodeAt(index)===92){index++;continue;}
   if(text[index]==='"'){values.push(text.slice(start,index+1));break;}
  }
 }
 return values;
}
const reports=[];
for(const name of ['wasm_terminal_wheel','wasm_utility_bindings','wasm_socket_view']){
 const before=fs.readFileSync(`${output}/${name}.before.rs`,'utf8');
 const fixed=fs.readFileSync(`${root}${name}.rs`,'utf8');
 let expected=compact(before);
 if(name==='wasm_terminal_wheel'){
  const callback='attach_custom_wheel_event_handler(callback.as_ref().unchecked_ref::<Function>().clone(),);';
  assert.equal(expected.split(callback).length-1,4);
  expected=expected.replaceAll(callback,'attach_custom_wheel_event_handler(callback.as_ref().unchecked_ref::<Function>().clone().into(),).unwrap();');
  expected=expected.replace('attach_custom_wheel_event_handler(replacement_fn.clone());','attach_custom_wheel_event_handler(replacement_fn.clone().into()).unwrap();');
 }else{
  expected=expected.replace('addon.dispose();','addon.dispose().unwrap();');
  if(name==='wasm_utility_bindings')expected=expected.replace('AttachAddon::new(socket.unchecked_into(),options.into()).unwrap().dispose();','AttachAddon::new(socket.unchecked_into(),options.into()).unwrap().dispose().unwrap();');
 }
 assert.equal(compact(fixed),expected);
 assert.deepEqual(literals(fixed),literals(before));
 const digest=text=>createHash('sha256').update(text).digest('hex');
 reports.push({test:root+name+'.rs',beforeSha256:digest(before),fixedSha256:digest(fixed),stringLiteralsUnchanged:true,allOriginalTokensUnchangedApartFromExplicitBoundaryAdapters:true});
}
const serialized=JSON.stringify(reports,null,2)+'\n';
if(process.argv.includes('--write'))fs.writeFileSync(output+'/fixture-integrity.json',serialized);
else assert.equal(fs.readFileSync(output+'/fixture-integrity.json','utf8'),serialized);
console.log(serialized);
