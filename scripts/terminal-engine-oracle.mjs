// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
// Runs the unmodified headless engine, not a mock of its buffer methods.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
process.chdir(resolve(import.meta.dirname, '..'));
const require = createRequire(resolve('source_repo/package.json'));
const { build } = require('esbuild');
mkdirSync('tmp/terminal-engine', {recursive:true});
mkdirSync('results/terminal-engine', {recursive:true});
const production = await build({ entryPoints:['source_repo/src/headless/public/Terminal.ts'],
  outfile:'tmp/terminal-engine/upstream.cjs', bundle:true, platform:'node', format:'cjs',
  target:'node24', tsconfig:'source_repo/src/headless/tsconfig.json', metafile:true, logLevel:'silent' });
const hashes = Object.fromEntries(Object.keys(production.metafile.inputs).sort().map(path =>
  [path,createHash('sha256').update(readFileSync(path)).digest('hex')]));
const {Terminal} = require(resolve('tmp/terminal-engine/upstream.cjs'));
import {hex,snapshot} from './terminal-snapshot.mjs';
const inputs=['abc','\rZ','\n','界','e\u0301','🙂','\x08!','\tX','\x07','\x1b[31mred\x1b[0m',
 '\x1b[2;2H@','\x1b[K','\x1b[2J','\x1b[2@x','\x1b[P','\x1b[2L','\x1b[M','\x1b[S',
 '\x1b[T','\x1b[?7l','\x1b[?7h','\x1b[?1049hALT','\x1b[?1049l','\x1b7saved\x1b8',
 '\x1b]2;title\x07','\x1b[c','\x1b[6n','\x1bP$qm\x1b\\','\x1b)0\x0eq\x0f','\x1b#8',
 '\x1b[2;3r','\x1b[?6h','\x1b[?6l','\x1b[4h','\x1b[4l','\x1b[2b','\x1b[38:2::1:2:3mX',
 '\x1b[?45h\x08','\x1b[3g\t','\x1bH','\x1b[!p','\x1bc'];
const seed=0x4354524d;let random=seed;
function next(){random^=random<<13;random^=random>>>17;random^=random<<5;return random>>>0;}
const traces=inputs.map((text,i)=>({id:`directed-${i}`,cols:5,rows:3,scrollback:6,ops:[['W','initial\r\ntext'],['W',text]]}));
traces.push({id:'split-utf16',cols:5,rows:3,scrollback:6,ops:[['W','\ud83d'],['W','\ude42e'],['W','\u0301']]});
traces.push({id:'split-utf8',cols:5,rows:3,scrollback:6,ops:[['B',[0xf0]],['B',[0x9f,0x99]],['B',[0x82]]]});
for(let i=0;i<128;i++) {
  const ops=[];
  for(let j=0;j<8;j++) {
    const choice=next()%20;
    if(choice===0)ops.push(['R',[2+next()%8,1+next()%4]]);
    else if(choice===1)ops.push(['S',(next()%7)-3]);
    else if(choice===2)ops.push(['C','']);
    else if(choice===3)ops.push(['Q','']);
    else ops.push(['W',inputs[next()%inputs.length]]);
  }
  traces.push({id:`seed-${i}`,cols:2+next()%8,rows:1+next()%4,scrollback:6,ops});
}
const output=[];
for(const trace of traces) {
  const t=new Terminal({cols:trace.cols,rows:trace.rows,scrollback:trace.scrollback,allowProposedApi:true});
  const events=[];
  t.onBell(()=>events.push('B'));t.onCursorMove(()=>events.push('C'));t.onLineFeed(()=>events.push('L'));
  t.onScroll(y=>events.push(`S${y}`));t.onData(s=>events.push('D'+hex(s)));t.onTitleChange(s=>events.push('T'+hex(s)));
  t._core.onRender(e=>events.push(`R${e.start}:${e.end}`));
  for(const [kind,arg] of trace.ops) {
    events.length=0;
    if(kind==='W'||kind==='B')await new Promise(done=>t.write(arg instanceof Array?Uint8Array.from(arg):arg,done));
    else if(kind==='R')t.resize(...arg);
    else if(kind==='S')t.scrollLines(arg);
    else if(kind==='C')t.clear();else if(kind==='Q')t.reset();
    const encoded=kind==='W'?hex(arg):Array.isArray(arg)?arg.join(','):String(arg);
    output.push([trace.id,trace.cols,trace.rows,trace.scrollback,kind,encoded,snapshot(t),events.join('/')].join('\t'));
  }
  t.dispose();
}
const fixture='crates/xterm-core/tests/fixtures/terminal-engine.tsv',text=output.join('\n')+'\n';
if(process.argv.includes('--generate'))writeFileSync(fixture,text);else assert.equal(readFileSync(fixture,'utf8'),text,'whole-terminal source fixture drift');
for(const [path,hash]of Object.entries(hashes))assert.equal(createHash('sha256').update(readFileSync(path)).digest('hex'),hash);
const report={command:`node scripts/terminal-engine-oracle.mjs${process.argv.includes('--generate')?' --generate':''}`,
  node:process.version,platform:`${process.platform}-${process.arch}`,seed,traces:traces.length,snapshots:output.length,
  sourceHashes:hashes,fixture,fixtureSha256:createHash('sha256').update(text).digest('hex'),
  scope:'Both real buffers, all packed cells/UTF16/extended attrs, cursor/margins/modes and ordered source events. Finite corpus, not complete equivalence proof.'};
writeFileSync('results/terminal-engine/oracle.json',JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify({seed,traces:traces.length,snapshots:output.length}));
