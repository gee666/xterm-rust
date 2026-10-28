// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
// Preserve all erase escape fixtures; record the upstream's explicit EL skip.
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { loadTerminalReference } from './terminal-reference.mjs';
import { hex } from './terminal-snapshot.mjs';
const { Terminal, sources }=await loadTerminalReference();
const names=['t0054-ECH','t0055-EL','t0056-ED','t0057-ED3','t0091-alt_screen_ED3'];
const rows=[];const outcomes=[];
for(const name of names) {
  const path=`source_repo/test/fixtures/escape_sequence_files/${name}`;
  const input=await readFile(`${path}.in`),expected=await readFile(`${path}.text`);
  const terminal=new Terminal({cols:80,rows:25,scrollback:1000,allowProposedApi:true});
  // Terminal2.test.ts writes through a PTY with output postprocessing. ONLCR
  // turns each LF into CR LF. These fixtures do not change terminal termios.
  const text=input.toString('utf8').replaceAll('\n','\r\n');
  await new Promise(done=>terminal.write(text,done));
  const buffer=terminal._core.buffers.active;
  let actual='';for(let row=buffer.ybase;row<buffer.ybase+25;row++)actual+=buffer.lines.get(row).translateToString(true).replace(/\s+$/,'')+'\n';
  const legacy=expected.toString('utf8').split('\n').map(line=>line.replace(/\s+$/,'')).join('\n');
  const matchesLegacy=actual===legacy;
  const upstreamSkip=name==='t0055-EL';
  if(!upstreamSkip)assert.equal(actual,legacy,`${name}: applicable legacy fixture assertion failed`);
  rows.push([name,hex(actual),+matchesLegacy,+upstreamSkip].join('\t'));
  outcomes.push({name,inputSha256:createHash('sha256').update(input).digest('hex'),expectedSha256:createHash('sha256').update(expected).digest('hex'),matchesLegacy,upstreamSkip,skipReason:upstreamSkip?'Terminal2.test.ts explicitly skips EL because cursor at cols differs from xterm. Rust must still match the unchanged JS engine output.':null});
  if(process.argv.includes('--generate')) {
    await mkdir('crates/xterm-core/tests/fixtures/erase-vt',{recursive:true});
    await writeFile(`crates/xterm-core/tests/fixtures/erase-vt/${name}.in`,input);
    await writeFile(`crates/xterm-core/tests/fixtures/erase-vt/${name}.text`,expected);
  } else {
    assert.deepEqual(await readFile(`crates/xterm-core/tests/fixtures/erase-vt/${name}.in`),input);
    assert.deepEqual(await readFile(`crates/xterm-core/tests/fixtures/erase-vt/${name}.text`),expected);
  }
  terminal.dispose();
}
const fixture='crates/xterm-core/tests/fixtures/erase-vt.tsv',corpus=rows.join('\n')+'\n';
if(process.argv.includes('--generate'))await writeFile(fixture,corpus);else assert.equal((await readFile(fixture,'utf8')).replace(/\r\n/g,'\n'),corpus);
await mkdir('results/csi-erase',{recursive:true});
await writeFile('results/csi-erase/vt-fixtures.json',JSON.stringify({node:process.version,platform:process.platform,sources,dimensions:[80,25],scrollback:1000,ptyMapping:'ONLCR LF->CRLF applied explicitly, not a native PTY execution claim',sourceSuite:'source_repo/src/browser/Terminal2.test.ts; this source suite skips Windows and explicitly skips EL',outcomes,fixtureSha256:createHash('sha256').update(corpus).digest('hex')},null,2)+'\n');
console.log(JSON.stringify(outcomes));
