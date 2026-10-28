// Dynamic-key, identity and reentrant SortedList oracle. Source remains untouched.
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import vm from 'node:vm';

const root = fileURLToPath(new URL('../', import.meta.url));
const source = readFileSync(path.join(root, 'source_repo/src/common/SortedList.ts'), 'utf8');
class IdleTaskQueue {
  tasks = [];
  enqueue(task) { this.tasks.push(task); }
  clear() { this.tasks = []; }
  flush() { while (this.tasks.length) { this.tasks[0](); this.tasks.shift(); } }
}
const context = vm.createContext({ IdleTaskQueue });
vm.runInContext(stripTypeScriptTypes(source.replace(/^import .*;\r?$/gm, '').replace(/^export /gm, ''), { mode: 'transform' }) + '\nglobalThis.List = SortedList;', context);
const lists = [new context.List(value => value.key, {}), new context.List(value => value.key, {})];
const entries = new Map();
const cursors = new Map();
const trace = [];
function number(value) { return value === 'u' ? undefined : Number(value); }
function op(command, which = 0, id = '-', arg = '-') {
  const list = lists[which];
  let actual = 'ok';
  const visited = [];
  try {
    switch (command) {
      case 'I':
        if (!entries.has(id)) entries.set(id, { key: number(arg), id });
        list.insert(entries.get(id)); break;
      case 'M': entries.get(id).key = number(arg); break;
      case 'D': actual = String(list.delete(entries.get(id))); break;
      case 'V': actual = Array.from(list.values(), value => value.id).join(','); break;
      case 'C': list.clear(); break;
      case 'G': cursors.set(id, list.getKeyIterator(number(arg))); break;
      case 'N': actual = cursors.get(id).next().value?.id ?? 'undefined'; break;
      case 'F':
        list.forEachByKey(number(id), value => {
          // Mutable unsorted keys plus nested use of the source's global work
          // index can revisit entries forever. A throwing workload callback
          // bounds the trace without changing the source algorithm.
          if (visited.length >= 128) throw new Error('callback cap');
          visited.push(value.id);
          if (arg === 'clear') list.clear();
          if (arg === 'delete') list.delete(value);
          if (arg === 'delete-read') { list.delete(value); Array.from(list.values()); }
          if (arg === 'nested') list.forEachByKey(number(id), inner => visited.push(`inner_${inner.id}`));
          if (arg === 'fail') throw new Error('callback failure');
        });
        actual = visited.join(','); break;
      default: assert.fail(`Unknown command ${command}`);
    }
  } catch (error) { actual = `${error.name}:${visited.join(',')}`; }
  trace.push([command, which, id, arg, actual].join('\t'));
}
// Directed source bugs and shared-index behavior, retained before randomness.
for (const row of [
  ['I',0,'a','1'],['I',0,'b','1'],['I',0,'c','1'],['G',0,'cursor','1'],['N',0,'cursor'],
  ['D',0,'a'],['V'],['N',0,'cursor'],['C'],['N',0,'cursor'],
  ['I',0,'two','2'],['V'],['I',0,'one','1'],['G',0,'failed','1'],['N',0,'failed'],['N',0,'failed'],
  ['V'],['I',0,'three','3'],['V'],['C'],['I',0,'one','1'],['I',0,'two','2'],['V'],
  ['M',0,'one','5'],['D',0,'two'],['D',0,'one'],['V'],['C'],
  ['I',0,'a','1'],['I',0,'b','1'],['I',0,'c','1'],['F',0,'1','delete-read'],['C'],
  ['I',0,'a','1'],['I',0,'b','1'],['F',0,'1','nested'],['F',0,'1','fail'],['V'],['C'],
  ['I',0,'u','u'],['D',0,'u'],['G',0,'undefined_key','NaN'],['N',0,'undefined_key'],['V'],['C'],
  ['I',0,'a','1'],['I',0,'b','1'],['I',0,'c','1'],['I',1,'x','1'],
  ['G',0,'outer','1'],['G',1,'inner','1'],['N',0,'outer'],['N',0,'inner'],['N',0,'inner'],['N',0,'outer'],
]) op(...row);
const seed = 0x571de;
let state = seed;
function random() { state ^= state << 13; state ^= state >>> 17; state ^= state << 5; return state >>> 0; }
const ids = [...entries.keys()];
const cursorIds = [...cursors.keys()];
for (let i = 0; i < 4000; i++) {
  const which = random() % 2;
  const key = ['-2','-1','0','1','2','NaN','u','Infinity','-Infinity'][random() % 9];
  switch (random() % 9) {
    case 0: case 1: { const id = `entry_${i}`; ids.push(id); op('I',which,id,key); break; }
    case 2: op('M',which,ids[random() % ids.length],key); break;
    case 3: op('D',which,ids[random() % ids.length]); break;
    case 4: op('V',which); break;
    case 5: op('C',which); break;
    case 6: { const id = `cursor_${i}`; cursorIds.push(id); op('G',which,id,key === 'u' ? 'NaN' : key); break; }
    case 7: op('N',which,cursorIds[random() % cursorIds.length]); break;
    case 8: op('F',which,key === 'u' ? 'NaN' : key,['clear','delete','delete-read','nested','fail'][random()%5]); break;
  }
}
const fixture = `#cases\t${trace.length}\n${trace.join('\n')}\n`;
const fixturePath = 'crates/xterm-core/tests/fixtures/collections-dynamic.tsv';
writeFileSync(path.join(root, fixturePath), fixture);
const cargo = process.env.CARGO || 'cargo';
const version = command => spawnSync(command, ['--version'], {cwd:root,encoding:'utf8',timeout:10000}).stdout?.trim();
const result = spawnSync(cargo, ['test','-p','xterm-core','--test','collections_dynamic_oracle'], { cwd: root, encoding: 'utf8', timeout: 120000, maxBuffer: 2**20 });
const outputDir = path.join(root,'results/collections');
mkdirSync(outputDir,{recursive:true});
writeFileSync(path.join(outputDir,'dynamic-rust.log'), (result.stdout || '') + (result.stderr || ''));
const report = { command:'node scripts/collections-dynamic-oracle.mjs',platform:process.platform,node:process.version,cargo:version(cargo),rustc:version(process.env.RUSTC || 'rustc'),seed,
  iterations:4000,operations:trace.length,source_sha256:createHash('sha256').update(source).digest('hex'),
  fixture_sha256:createHash('sha256').update(fixture).digest('hex'),fixture:fixturePath,
  rust_exit:result.status,rust_error:result.error?.message,passed:!result.error && result.status===0,
  scope:['mutable/undefined/NaN keys','identity deletion','getter failures and recovery','lazy cursors and generator close','shared work-index context','callback mutation/nesting/failures'],
  gaps:['Controlled queue; real browser idle scheduling remains task83.','Non-total NaN sorting order is implementation-defined by ECMAScript.'] };
const mismatch = result.stdout?.match(/row (\d+): ([^\r\n]+)/);
if (mismatch) {
  writeFileSync(path.join(outputDir,'dynamic-counterexample.json'), JSON.stringify({seed,index:Number(mismatch[1]),operation:mismatch[2],trace:trace.slice(0,Number(mismatch[1])+1),stdout:result.stdout,stderr:result.stderr},null,2)+'\n');
}
writeFileSync(path.join(outputDir,'dynamic-oracle.json'),JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(report,null,2));
if (!report.passed) { process.exitCode=1; console.error(result.stdout,result.stderr,result.error || ''); }
