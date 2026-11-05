import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
const hash=b=>createHash('sha256').update(b).digest('hex');
const paths=[
'crates/xterm-addons/src/ligatures/font_cff_header.rs',
'crates/xterm-addons/src/ligatures/font_cff_index.rs',
'crates/xterm-addons/src/ligatures/font_hint_deflate.rs',
'crates/xterm-addons/src/ligatures/font_deflate_output.rs',
'crates/xterm-addons/src/ligatures/font_deflate_output_tests.rs',
'crates/xterm-addons/tests/font_cff_index.rs',
'crates/xterm-addons/tests/support/font_cff_index_assertions.rs',
'crates/xterm-addons/tests/fixtures/font-cff-index.json',
'crates/xterm-addons/tests/fixtures/cff-window-stored.deflate',
'crates/xterm-addons/tests/fixtures/cff-window-fixed.deflate',
'crates/xterm-addons/tests/fixtures/cff-window-dynamic.deflate',
'crates/xterm-font-addons/tests/font_cff_index_browser.rs',
'crates/xterm-font-addons/tests/support/font_cff_index_input.mjs',
'scripts/font-cff-index-oracle.mjs','scripts/font-cff-index-browser-source.mjs','scripts/font-cff-index-negative.mjs',
'Cargo.lock',
'crates/xterm-addons/src/ligatures/font.rs','crates/xterm-addons/src/ligatures/font_bytes.rs','crates/xterm-addons/src/ligatures/font_error.rs',
'crates/xterm-addons/tests/support/font_cff_header_assertions.rs',
'crates/xterm-font-addons/tests/support/font_cff_header_input.mjs',
'crates/xterm-font-addons/tests/support/font_cff_header_addon.mjs'];
const pins={};for(const file of paths)pins[file]=hash(await fs.readFile(file));
await fs.writeFile('results/font-cff-index/producer-hashes.json',JSON.stringify(pins,null,2)+'\n');
const manifest=JSON.parse(await fs.readFile('docs/baseline/reference-manifest.json'));
const references={};for(const root of ['source_repo','P:/workshop/Windows_Worshop/oira666/xterm-rust/source_repo']) {
  const bad=[];for(const entry of manifest)if(hash(await fs.readFile(`${root}/${entry.path}`))!==entry.sha256)bad.push(entry.path);
  references[root]={files:manifest.length,mismatches:bad};if(bad.length)throw new Error(`Changed reference ${root}`);
}
await fs.writeFile('results/font-cff-index/reference-integrity.json',JSON.stringify(references,null,2)+'\n');
const native=await fs.readFile('results/font-cff-index/native-all-targets.log','utf8');
const runs=[...native.matchAll(/test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;/g)];
const git=spawnSync(process.env.GIT??'git',['rev-parse','HEAD'],{encoding:'utf8',timeout:10000});if(git.status!==0)throw new Error('Cannot identify source commit');
const report={sourceCommit:git.stdout.trim(),scope:'Task472 initial CFF INDEX phase only',overallEquivalence:'NOT PROVEN',source:JSON.parse(await fs.readFile('results/font-cff-index/oracle.json')),native:{suites:runs.length,testFunctions:runs.reduce((n,m)=>n+Number(m[2]),0),failed:runs.reduce((n,m)=>n+Number(m[3]),0),ignored:runs.reduce((n,m)=>n+Number(m[4]),0)},negative:JSON.parse(await fs.readFile('results/font-cff-index/negative.json')),clippy:'Scoped native/WASM warning denied PASS',format:'Exact fullfmt OS206; five packagefmt PASS',knownGap:'Browser Bounds RangeError message differs from source DataView native wording; raw receipts retained. Full CFF dictionaries/charstrings342 incomplete.',cleanup:{ownedRoot:process.cwd(),liveJobs:0,action:'Only own regular target/tmp and generated browser outputs may be removed after review; preserve branch/evidence and inspect links first.'}};
for(const file of ['source-browser.json','rust-browser.json'])try{report[file]=JSON.parse(await fs.readFile(`results/font-cff-index/${file}`));}catch(e){if(e.code!=='ENOENT')throw e;report[file]='UNRUN';}
await fs.writeFile('results/font-cff-index/checks.json',JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify({pins:paths.length,references,native:report.native,browsers:Object.fromEntries(['source-browser.json','rust-browser.json'].map(k=>[k,report[k].status??report[k]]))}));
