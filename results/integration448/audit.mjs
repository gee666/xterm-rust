// Reuse integration438's exact-blob audit. Derive every combined file from the
// preserved base and reviewed producer edits, not from the assembled checkout.
import fs from 'node:fs';
import crypto from 'node:crypto';
import {execFileSync} from 'node:child_process';
const git=(...args)=>execFileSync(process.env.GIT??'git',args,{timeout:30000,maxBuffer:16*1024*1024});
const text=(...args)=>git(...args).toString().trim();
const sha=bytes=>crypto.createHash('sha256').update(bytes).digest('hex');
const base='3aa04692a409bc147670d7c61a9e2ac37b5ea9f8';
const batches=[
 {task:420,chain:['b85e9b61','f244d9f2','39726ee3','34bca88d','26277170','67cd67e6','119070bb','d2e87757']},
 {task:432,chain:['cef749ae','7d302606','9c7a9c0b']},
 {task:436,chain:['0cc7446b','a698c945','796b4601','5e1207d6','1b52759c']},
 {task:437,chain:['532551a2','e7a7be6a','4576a356','c4b23460']},
 {task:439,chain:['95d7d723','15543704','522deaf2','3fed85ab','ab6dad9e']},
 {task:440,chain:['2d0258cb','73c126b0','0efbd4bb','a4d86734']},
 {task:441,chain:['40c65302','f68b3f50','0c9d137e']},
 {task:438,chain:['f775f0b7','af02a450'],receiptOnly:true},
 {task:452,chain:['bc80e585','5da1dc80','f76f70ef']},
 {task:455,chain:['73fa8f06','a128814f']},
];
const combined=new Map();
function compose(file,changes){
 let value=git('show',`${base}:${file}`).toString();
 for(const [before,after]of changes){
  if(value.split(before).length!==2)throw Error(`Ambiguous combination ${file}: ${before}`);
  value=value.replace(before,after);
 }
 combined.set(file,Buffer.from(value));
}
compose('crates/xterm-addons/src/ligatures/font_bytes.rs',[
 ['use std::{borrow::Cow, fmt};','use std::{borrow::Cow, fmt};\n#[path = "font_hint_bytes.rs"]\nmod hints;'],
 ['    Bounds,','    Bounds,\n    ArrayLength,'],
 ['            Self::Bounds => f.write_str("Font table offset is outside the input"),','            Self::Bounds => f.write_str("Font table offset is outside the input"),\n            Self::ArrayLength => f.write_str("Invalid array length"),'],
 ['            match tag {\n                b"glyf" => glyf = true,','            match tag {\n                b"cvt " | b"fpgm" | b"prep" => {\n                    hints::validate(reader, record, woff, tag == b"cvt ")?;\n                    continue;\n                }\n                b"glyf" => glyf = true,'],
]);
compose('crates/xterm-font-addons/src/ligature_state.rs',[
 ['        FontError::Bounds => js_sys::RangeError::new(&error.to_string()).into(),','        FontError::Bounds | FontError::ArrayLength => {\n            js_sys::RangeError::new(&error.to_string()).into()\n        }'],
]);
const image='crates/xterm-web/src/image/';
compose(image+'addon.rs',[
 ['mod protocols;','mod protocols;\nmod storage_limit;'],
 ['    defaults: ImageOptions,','    defaults: ImageOptions,\n    storage_limit: Rc<RefCell<JsValue>>,'],
]);
const binding=image+'addon/bindings.rs';
const bindingBase=git('show',`${base}:${binding}`).toString();
const bindingProducer=git('show',`ab6dad9e:${binding}`).toString();
function setter(value){
 const start=value.indexOf('    pub fn set_storage_limit(');
 const end=value.indexOf('    #[wasm_bindgen(getter, js_name = storageUsage)]',start);
 if(start<0||end<start)throw Error('Missing complete storage-limit setter');
 return value.slice(start,end);
}
compose(binding,[
 ['            placeholder,\n        } = options::read','            placeholder,\n            storage_limit,\n        } = options::read'],
 ['        let placeholder_value = Rc::new(RefCell::new(placeholder));','        let placeholder_value = Rc::new(RefCell::new(placeholder));\n        let storage_limit = Rc::new(RefCell::new(storage_limit));'],
 ['            placeholder_value.clone(),','            placeholder_value.clone(),\n            storage_limit.clone(),'],
 ['                defaults,','                defaults,\n                storage_limit,'],
 ['    /// Live read-only options. Placeholder values retain their JavaScript\n    /// identity; other fields remain typed projections, not a writable record.','    /// Live read-only options. Placeholder and storage-limit values retain their\n    /// JavaScript identity; other fields remain typed projections.'],
 [setter(bindingBase),setter(bindingProducer)],
]);
compose(image+'addon/option_inspection.rs',[
 ['//! Read-only diagnostics, not a writable JS option bag. Placeholder inspection\n//! reads the same raw value as the public property; other options remain typed.','//! Read-only diagnostics, not a writable JS option bag. Placeholder and storage\n//! limit inspection retain their raw public values; other options remain typed.'],
 ['    placeholder: Rc<RefCell<JsValue>>,','    placeholder: Rc<RefCell<JsValue>>,\n    storage_limit: Rc<RefCell<JsValue>>,'],
 ['    pub(super) fn new(options: Rc<Cell<ImageOptions>>, placeholder: Rc<RefCell<JsValue>>) -> Self {','    pub(super) fn new(\n        options: Rc<Cell<ImageOptions>>,\n        placeholder: Rc<RefCell<JsValue>>,\n        storage_limit: Rc<RefCell<JsValue>>,\n    ) -> Self {'],
 ['            placeholder,','            placeholder,\n            storage_limit,'],
 ['            "storageLimit" => options.storage_limit.into(),','            "storageLimit" => self.storage_limit.borrow().clone(),'],
]);
compose(image+'addon/options.rs',[
 ['    pub placeholder: JsValue,','    pub placeholder: JsValue,\n    pub storage_limit: JsValue,'],
 ['        defaults: typed(&defaults)?,','        defaults: typed(&defaults)?,\n        storage_limit: js::get(&current, "storageLimit")?,'],
 ['        storage_limit: numeric("storageLimit")?,','        // Storage owns delayed raw-value conversion, not this typed projection.\n        storage_limit: ImageOptions::default().storage_limit,'],
]);
compose('crates/xterm-web/tests/image_addon_browser.rs',[
 ['mod raw_placeholder;','mod raw_placeholder;\n#[path = "image_addon_support/raw_storage_limit.rs"]\nmod raw_storage_limit;'],
 ['rust.set_storage_limit(64.0)','rust.set_storage_limit(64.0.into())'],
 ['pair.addon.set_storage_limit(limit)','pair.addon.set_storage_limit(limit.into())'],
 ['mod support;','mod support;\n#[path = "image_addon_support/visibility.rs"]\nmod visibility;'],
]);
compose('crates/xterm-web/tests/image_addon_support/options.rs',[
 ['pair.addon.set_storage_limit(64.0)','pair.addon.set_storage_limit(64.0.into())'],
 ['pair.addon.set_storage_limit(-1.0)','pair.addon.set_storage_limit((-1.0).into())'],
]);
compose('scripts/image-canvas-check.mjs',[
 ["'crates/xterm-web/tests/image_addon_support/raw_placeholder.rs',","'crates/xterm-web/tests/image_addon_support/raw_placeholder.rs', 'crates/xterm-web/tests/image_addon_support/raw_storage_limit.rs', 'crates/xterm-web/tests/image_addon_support/visibility.rs', 'crates/xterm-addons/src/image/storage/mod.rs', 'crates/xterm-addons/src/image/storage/budget.rs', 'crates/xterm-addons/tests/image_storage_cases/budget.rs',"],
]);
const report={base,assembly:text('rev-parse','HEAD'),batches:[],sourcePins:[],receiptPins:[],preservedBasePins:[],overlaps:[],status:'preparing'};
const owners=new Map();
const fixtureOverlaps=new Set(['crates/xterm-web/tests/image_addon_browser.rs','scripts/image-canvas-check.mjs']);
const source=path=>['crates/','scripts/','.github/','.pi/swarm/skills/'].some(prefix=>path.startsWith(prefix));
for(const batch of batches){
 const chain=batch.chain.map(ref=>text('rev-parse',ref)),tip=chain.at(-1);
 const files=[...new Set(chain.flatMap(ref=>text('diff-tree','--root','--no-commit-id','--name-only','-r',ref).split('\n').filter(Boolean)))];
 if(batch.receiptOnly&&files.some(source))throw Error('Receipt-only batch changes source');
 report.batches.push({...batch,chain,tip,files:files.length});
 for(const file of files.filter(source)){
  if(owners.has(file)){
   if(batch.task!==455||!fixtureOverlaps.has(file))throw Error(`Unexpected producer overlap ${file}`);
   report.overlaps.push({path:file,first:owners.get(file),second:batch.task,approved:true});
  }
  owners.set(file,batch.task);
  const producer=git('show',`${tip}:${file}`),expected=combined.get(file)??producer,actual=git('show',`HEAD:${file}`),working=fs.readFileSync(file);
  const eolOnly=file.endsWith('.md')&&expected.toString().replaceAll('\r\n','\n')===working.toString().replaceAll('\r\n','\n');
  report.sourcePins.push({task:batch.task,path:file,producerSha256:sha(producer),expectedSha256:sha(expected),headSha256:sha(actual),composed:combined.has(file),checkoutEolOnly:!expected.equals(working)&&eolOnly,matches:expected.equals(actual)&&(expected.equals(working)||eolOnly)});
 }
 for(const file of files.filter(file=>file.startsWith('results/'))){
  const expected=git('show',`${tip}:${file}`),actual=git('show',`HEAD:${file}`);
  report.receiptPins.push({task:batch.task,path:file,expectedSha256:sha(expected),headSha256:sha(actual),matches:expected.equals(actual)});
 }
}
const previous=JSON.parse(git('show',`${base}:results/integration438/manifest.json`));
for(const {path:file}of previous.sourcePins){
 const updated=report.sourcePins.find(pin=>pin.path===file);
 const expectedSha256=updated?.expectedSha256??sha(git('show',`${base}:${file}`));
 const headSha256=sha(git('show',`HEAD:${file}`));
 report.preservedBasePins.push({path:file,updatedByAcceptedTask:updated?.task??null,expectedSha256,headSha256,matches:expectedSha256===headSha256});
}
report.status=[...report.sourcePins,...report.receiptPins,...report.preservedBasePins].every(p=>p.matches)?'passed':'failed';
if(process.argv.includes('--write'))fs.writeFileSync('results/integration448/manifest.json',JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify({status:report.status,sourcePins:report.sourcePins.length,composedFiles:combined.size,receiptPins:report.receiptPins.length,preservedBasePins:report.preservedBasePins.length,mismatches:[...report.sourcePins,...report.receiptPins,...report.preservedBasePins].filter(p=>!p.matches)}));
if(report.status!=='passed')process.exitCode=1;
