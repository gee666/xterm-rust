// Copyright (c) 2019-2026 The xterm.js authors. MIT license.
// Forward-only observers around unchanged cached headless source in owned children.
import { readFile, writeFile, mkdir, stat } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { strict as assert } from 'node:assert';
import { runCommand } from './quality-process.mjs';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const bundle = path.resolve(root, process.env.SOURCE_HEADLESS_BUNDLE ?? 'tmp/native-queue-oracle/terminal.cjs');
await stat(bundle);
const provenanceFile = path.join(root, 'results/native-queue-review/oracle.json');
const provenance = JSON.parse(await readFile(provenanceFile, 'utf8'));
const hash = data => createHash('sha256').update(data).digest('hex');
for (const [source, expected] of Object.entries(provenance.sourceHashes)) {
  assert.equal(hash(await readFile(path.join(root, source))), expected, `cached source provenance drift: ${source}`);
}
const id = new Date().toISOString().replace(/[:.]/g, '-') + '-' + process.pid;
const temporary = path.join(root, 'tmp/decoder-overflow', id);
const output = path.join(root, 'results/decoder-overflow', id);
await mkdir(temporary, { recursive: true });
await mkdir(output, { recursive: true });
const child = path.join(temporary, 'child.mjs');
await writeFile(child, `import {createRequire} from 'node:module';
const require=createRequire(import.meta.url);
const {Terminal}=require(process.argv[2]);
const length=Number(process.argv[3]);
const term=new Terminal({allowProposedApi:true,cols:80,rows:24});
const handler=term._core._inputHandler;
const decoder=handler._stringDecoder;
const decode=decoder.decode;
let decodeCalls=0, parseCalls=0, emptyPrints=0;
decoder.decode=function(input,target){const count=decode.call(this,input,target);
if(decodeCalls++<2)console.log(JSON.stringify({kind:'decode',inputUnits:input.length,capacity:target.length,logicalCount:count}));return count;};
const parser=handler._parser, parse=parser.parse, print=parser._printHandler;
if(typeof print!=='function')throw Error('cached parser has no original print handler');
parser.parse=function(data,length,...rest){if(parseCalls++<2)console.log(JSON.stringify({kind:'parse',capacity:data.length,length}));return parse.call(this,data,length,...rest);};
parser._printHandler=function(data,start,end){if(start===end&&emptyPrints++<2)console.log(JSON.stringify({kind:'emptyPrint',capacity:data.length,start,end,cursorX:term.buffer.active.cursorX,cursorY:term.buffer.active.cursorY}));return print.call(this,data,start,end);};
try{await new Promise(resolve=>term.write('\\ud800',resolve));await new Promise(resolve=>term.write('A'.repeat(length),resolve));console.log(JSON.stringify({kind:'completed',cursorX:term.buffer.active.cursorX,cursorY:term.buffer.active.cursorY}));}finally{term.dispose();}
`);
const outcomes = [];
for (const length of [4095, 4096, 131072]) {
  const log = path.join(output, `length-${length}.log`);
  const execution = await runCommand(process.execPath, [child, bundle, String(length)], {
    cwd: root, env: process.env, log, timeoutMs: 2000,
  });
  const records = (await readFile(log, 'utf8')).split(/\r?\n/).filter(line => line.startsWith('{')).map(line => JSON.parse(line));
  const empty = records.filter(record => record.kind === 'emptyPrint');
  assert.ok(empty.length <= 2, 'observer output must remain bounded');
  const completed = records.some(record => record.kind === 'completed');
  if (length === 4095) {
    assert.equal(execution.status, 'passed');
    assert.equal(completed, true);
    assert.equal(empty.length, 0);
  } else {
    assert.equal(execution.timedOut, true, 'reference overflow is expected not to finish');
    assert.equal(completed, false);
    assert.equal(empty.length, 2);
    assert.equal(empty[0].start, empty[0].end);
    assert.equal(empty[0].start, empty[1].start);
    assert.ok(records.some(record => record.kind === 'decode' && record.logicalCount > record.capacity));
  }
  outcomes.push({ length, execution, records });
}
const report = { scope: 'Original source non-progress witness, not Rust error equivalence',
  node: process.version, platform: process.platform, cachePath: path.relative(root, bundle),
  cacheSha256: hash(await readFile(bundle)), provenance: path.relative(root, provenanceFile),
  sourceHashes: provenance.sourceHashes, rebuiltReference: false,
  observers: 'Decode/parse/print wrappers forward original arguments and return values without changing behavior; at most two empty-print witnesses',
  outcomes, conclusion: 'Physical target overflow leads to repeated zero-length PRINT spans in source. A finite Rust error is a deliberate safety deviation, not demonstrated source equivalence.' };
await writeFile(path.join(output, 'report.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ output: path.relative(root, output), outcomes: outcomes.map(({length,execution,records}) => ({length,status:execution.status,timedOut:execution.timedOut,records})) }, null, 2));
