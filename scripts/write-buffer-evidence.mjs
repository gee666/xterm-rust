// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
// Run after the documented commands, retaining scope and source identities.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
const root=resolve(import.meta.dirname,'..');
const read=path=>readFileSync(resolve(root,path),'utf8');
const oracle=JSON.parse(read('results/write-buffer/oracle.json'));
assert.equal(oracle.baseline.discovered,19);
assert.equal(oracle.baseline.passed,19);
assert.equal(oracle.baseline.failed,0);
assert.equal(oracle.steps,8192);
const tests=read('results/write-buffer/final-tests.log');
assert.match(tests,/test result: ok\. 42 passed; 0 failed; 0 ignored/);
const clippy=read('results/write-buffer/final-clippy.log');
assert.match(clippy,/Finished `dev` profile/);
assert.doesNotMatch(clippy,/^error[\[:]/m);
const files=[
 'crates/xterm-core/src/input/write_buffer.rs',
 'crates/xterm-core/src/input/write_buffer_drain.rs',
 'crates/xterm-core/src/input/write_buffer_types.rs',
 'crates/xterm-core/tests/write_buffer.rs',
 'crates/xterm-core/tests/write_buffer_support/mod.rs',
 'scripts/write-buffer-oracle.mjs',
 'scripts/write-buffer-evidence.mjs',
 ...readdirSync(resolve(root,'crates/xterm-core/tests/write_buffer_cases')).filter(f=>f.endsWith('.rs')).map(f=>'crates/xterm-core/tests/write_buffer_cases/'+f),
 'crates/xterm-core/tests/fixtures/write-buffer-traces.tsv',
 'results/write-buffer/oracle.json',
 'results/write-buffer/final-oracle.log',
 'results/write-buffer/final-tests.log',
 'results/write-buffer/final-clippy.log',
 'results/write-buffer/final-fmt.log',
];
const hashes=Object.fromEntries(files.map(file=>[file,createHash('sha256').update(readFileSync(resolve(root,file))).digest('hex')]));
const report={
 scope:'Portable WriteBuffer implementation only',overallEquivalence:'NOT PROVEN',
 platform:process.platform,node:process.version,
 rustc:execFileSync('rustc',['--version'],{encoding:'utf8'}).trim(),
 cargo:execFileSync('cargo',['--version'],{encoding:'utf8'}).trim(),
 baseline:{executed:19,passed:19,failed:0,skipped:0,unrun:0},
 rust:{executed:42,passed:42,failed:0,ignored:0},
 differential:{seed:oracle.seed,traces:128,steps:8192,scope:'Synchronous and timer operations; controlled async tests are separate'},
 extraOriginalAssertions:{rejectionOrder:oracle.rejectionOrder,externalViews:oracle.externalViews},
 commands:[
  {command:'node scripts/write-buffer-oracle.mjs',log:'results/write-buffer/final-oracle.log',exitCode:0},
  {command:'cargo test -p xterm-core --test write_buffer',log:'results/write-buffer/final-tests.log',exitCode:0},
  {command:'cargo clippy -p xterm-core --test write_buffer -- -D warnings',log:'results/write-buffer/final-clippy.log',exitCode:0},
  {command:'rustfmt --edition 2021 --check crates/xterm-core/src/input/write_buffer.rs crates/xterm-core/src/input/write_buffer_drain.rs crates/xterm-core/src/input/write_buffer_types.rs crates/xterm-core/tests/write_buffer.rs',log:'results/write-buffer/final-fmt.log',exitCode:0},
 ],
 hashes,
 retainedFailures:['results/write-buffer/build-failure-lifecycle-terminal.log','results/write-buffer/build-failure-shared-api.log','results/write-buffer/validated-tests-v2.log','results/write-buffer/adapter-bridge-clippy.log'],
 unrun:['Native Linux runtime','Native macOS runtime','Integrated native queue task128','Integrated browser queue task118','Workspace-wide quality gates'],
};
writeFileSync(resolve(root,'results/write-buffer/verification.json'),JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify({baseline:report.baseline,rust:report.rust,differential:report.differential,overallEquivalence:report.overallEquivalence}));
