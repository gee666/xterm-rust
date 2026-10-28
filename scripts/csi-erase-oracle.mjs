// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
// Full-cell erase traces use the actual unchanged headless terminal engine.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { hex, snapshot } from './terminal-snapshot.mjs';
import { loadTerminalReference } from './terminal-reference.mjs';
const { Terminal, sources } = await loadTerminalReference();
const traces = [];
const add = (ops, extra = {}) => traces.push({ cols: 8, rows: 4, scrollback: 6, ops, ...extra });
const commands = ['J','K','X'];
for (const command of commands) for (const parameter of ['', '0', '1', '2', '3', '9', '2147483647']) {
  for (const selective of command === 'X' ? [false] : [false,true]) {
    for (const x of [0,3,8,10]) {
      add([['W','a'.repeat(32)],['M',0],['M',2],['C',[x,2]],['W',`\x1b[${selective?'?':''}${parameter}${command}`]]);
    }
  }
}
// Protection, BCE, extended styles, wide/combined cells and wrapped boundaries.
for (const command of ['J','K']) for (const parameter of [0,1,2,3,9]) for (const x of [0,1,3,7,8]) {
  add([['W','\x1b[41;1mα界e\u0301\x1b[1"qPQ\x1b[2"qR\x1b[0m\r\nsecond\r\nthird'],['C',[x,0]],['W',`\x1b[?${parameter}${command}`],['W',`\x1b[${parameter}${command}`]]);
}
for (const value of ['',0,1,2,3,9]) add([['W',`\x1b[${value}"q`],['W','abc'],['C',[0,0]],['W','\x1b[?2K']]);
for (const last of [0,1,2,3]) for (const selective of [false,true]) {
  add([['O',1],['C',[0,last]],['W','last'],['M',last],['C',[3,1]],['W',`\x1b[${selective?'?':''}2J`]]);
}
add([['W','\x1b[41m'],['O',1],['W','\x1b[2J']]);
for (const alternative of [false,true]) for (const selective of [false,true]) {
  const ops=[['W','0\r\n1\r\n2\r\n3\r\n4\r\n5'],['M',0],['M',3],['S',1]];
  if (alternative) ops.push(['W','\x1b[?1049h'],['A',1]);
  ops.push(['W',`\x1b[${selective?'?':''}3J`]);add(ops);
}
const seed=0xe2a5e111;let random=seed;
function next(){random^=random<<13;random^=random>>>17;random^=random<<5;return random>>>0;}
for(let i=0;i<128;i++) {
  const ops=[['W','one\r\ntwo界\r\nthree\r\nfour'],['M',next()%4]];
  for(let step=0;step<12;step++) {
    const choices=['\x1b[1"qP\x1b[2"qU','\x1b[44;3mX\x1b[0m','\r\nscroll','e\u0301🙂'];
    if(next()%4===0) ops.push(['W',choices[next()%choices.length]]);
    else {ops.push(['C',[next()%10,next()%4]]);const cmd=commands[next()%3];ops.push(['W',`\x1b[${cmd!=='X'&&next()%2?'?':''}${next()%5}${cmd}`]);}
  }
  add(ops);
}
const output=[];let writeCalls=0,markerCallbacks=0;
for(let index=0;index<traces.length;index++) {
  const trace=traces[index],t=new Terminal({cols:trace.cols,rows:trace.rows,scrollback:trace.scrollback,allowProposedApi:true}),c=t._core,markers=[],events=[];
  t.onBell(()=>events.push('B'));t.onCursorMove(()=>events.push('C'));t.onLineFeed(()=>events.push('L'));t.onScroll(y=>events.push(`S${y}`));
  t.onData(s=>events.push('D'+hex(s)));t.onTitleChange(s=>events.push('T'+hex(s)));c.onRender(e=>events.push(`R${e.start}:${e.end}`));
  for(const [kind,value]of trace.ops) {
    events.length=0;
    if(kind==='W') {await new Promise(done=>t.write(value,done));writeCalls++;}
    else if(kind==='C') {c.buffers.active.x=value[0];c.buffers.active.y=value[1];}
    else if(kind==='O')t.options.scrollOnEraseInDisplay=!!value;
    else if(kind==='S')c._bufferService.isUserScrolling=!!value;
    else if(kind==='M') {const marker=c.buffers.active.addMarker(value),label=markers.length;markers.push(marker);marker.onDispose(()=>{markerCallbacks++;events.push(`M${label}:${marker.line}`);});}
    else if(kind==='A') {const b=c.buffers.active;b.lines.push(b.getBlankLine(c._inputHandler.getAttrData()));b.ybase=1;}
    const encoded=kind==='W'?hex(value):Array.isArray(value)?value.join(','):String(value);
    const marks=markers.map((m,i)=>`${i}:${m.line}:${+m.isDisposed}`).join(';');
    output.push([index,kind,encoded,snapshot(t),+c._bufferService.isUserScrolling,marks,events.join('/')].join('\t'));
  }
  t.dispose();
}
const corpus=output.join('\n')+'\n',fixture='crates/xterm-core/tests/fixtures/csi-erase.tsv';
if(process.argv.includes('--generate'))await writeFile(fixture,corpus);else assert.equal((await readFile(fixture,'utf8')).replace(/\r\n/g,'\n'),corpus,'erase corpus changed');
const result={node:process.version,platform:process.platform,seed,traces:traces.length,snapshots:output.length,writeCalls,markerCallbacks,sources,fixture,fixtureSha256:createHash('sha256').update(corpus).digest('hex'),status:process.argv.includes('--generate')?'generated':'matched',scope:'Real both-buffer fullcells/state/modes, all erase/protection parameter branches, BCE, marker disposal/order and user-scroll flags. Finite differential corpus; not whole terminal equivalence.'};
await mkdir('results/csi-erase',{recursive:true});await writeFile('results/csi-erase/oracle.json',JSON.stringify(result,null,2)+'\n');console.log(JSON.stringify({traces:traces.length,snapshots:output.length,writeCalls,markerCallbacks}));
