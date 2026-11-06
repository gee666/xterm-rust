// Require executed constructor mismatches when only FDSelect validation is bypassed.
import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
import {runCommand} from './quality-process.mjs';
const file='crates/xterm-addons/src/ligatures/font_cff_index.rs',out='results/font-cff-fdselect';
const before=await fs.readFile(file),text=before.toString(),needle='fdselect::validate(&data, start, select, glyphs, count)?;';
assert.equal(text.split(needle).length,2);
const sha=b=>createHash('sha256').update(b).digest('hex');
const fixture=JSON.parse(await fs.readFile('crates/xterm-addons/tests/fixtures/font-cff-fdselect.json'));
const designated=fixture.cases.filter(c=>c.addonBad).map(c=>c.name);
assert.equal(designated.length,4);
const report={beforeSha256:sha(before),designated,status:'running'};
await fs.mkdir(out,{recursive:true});
const run=async name=>{
  const args=['test','-j2','-p','xterm-addons','--test','font_cff_fdselect','unchanged_full_source_fdselect_constructors'];
  const command=await runCommand(process.env.CARGO??'cargo',args,{cwd:process.cwd(),env:process.env,log:`${out}/${name}.log`,timeoutMs:900000});
  return {command,raw:await fs.readFile(`${out}/${name}.log`,'utf8')};
};
try {
  // Keep validation referenced. The fixed corpus has glyph1629 or undefined,
  // never0, so only this missing phase changes the observed constructors.
  await fs.writeFile(file,text.replace(needle,`if glyphs == Some(0) { ${needle} }`));
  report.mutantSha256=sha(await fs.readFile(file));
  const {command,raw}=await run('negative');report.command=command;
  assert.equal(command.status,'failed');assert.match(raw,/test result: FAILED\. 0 passed; 1 failed/);
  report.executedMismatches=[...raw.matchAll(/"(select\d+)" expected [^\n]+/g)].map(m=>m[0]);
  for(const name of designated)assert(report.executedMismatches.some(line=>line.startsWith(`"${name}" expected`)),'Designated constructor did not expose bypass '+name);
  assert(report.executedMismatches.length>=100);report.status='passed';
} catch(error) {report.status='failed';report.error=String(error.stack??error);throw error;}
finally {
  await fs.writeFile(file,before);report.restoredSha256=sha(await fs.readFile(file));assert.equal(report.restoredSha256,report.beforeSha256);
  const positive=await run('restored');report.positive=positive.command;
  if(positive.command.status!=='passed')report.status='failed';
  await fs.writeFile(`${out}/negative.json`,JSON.stringify(report,null,2)+'\n');
  assert.equal(positive.command.status,'passed','Positive replay after exact restoration must pass');
}
console.log(JSON.stringify(report));
