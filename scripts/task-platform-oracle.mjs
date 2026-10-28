// Copyright (c) 2016-2026 The xterm.js authors. All rights reserved. MIT license.
import assert from 'node:assert/strict';
import { stripTypeScriptTypes } from 'node:module';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import vm from 'node:vm';
const root = path.resolve(import.meta.dirname, '..');
const out = path.join(root, 'results/task-platform'); mkdirSync(out, { recursive: true });
const attempt = { started:new Date().toISOString(),status:'running',commands:[] };
const saveAttempt = () => writeFileSync(path.join(out,'latest-attempt.json'),JSON.stringify(attempt,null,2)+'\n');
saveAttempt();
process.once('uncaughtExceptionMonitor',error=>{attempt.status='failed';attempt.error=String(error.stack);saveAttempt();});
const hashes = {};
function source(file) {
  const value = readFileSync(path.join(root, 'source_repo/src/common', file), 'utf8');
  hashes[file] = createHash('sha256').update(value).digest('hex'); return value;
}
const erase = text => stripTypeScriptTypes(text.replace(/^import .*;\r?$/gm, '').replace(/^export /gm, ''), { mode: 'transform' });
const queueSource = erase(source('TaskQueue.ts'));
const platformSource = erase(source('Platform.ts'));
const versionSource = erase(source('Version.ts'));
const lineSource = ['buffer/Constants.ts', 'input/TextDecoder.ts', 'buffer/AttributeData.ts', 'buffer/CellData.ts', 'buffer/BufferLine.ts'].map(file => erase(source(file))).join('\n');
const windowsSource = erase(source('WindowsMode.ts'));
const windowsTests = erase(source('WindowsMode.test.ts'));
const windowsContext = vm.createContext({ assert, describe(_name, callback) { callback(); },
  it(name, callback) { callback(); originalNames.push(name); } });
const originalNames = [];
vm.runInContext(lineSource + '\n' + windowsSource + `
class OptionsService { constructor(options) { this.rawOptions = options; } }
class MockLogService {}
// Structural BufferService fixture only. Real service/linefeed integration is
// separate task28 coverage, not established by these unchanged assertions.
class BufferService {
  constructor(options) {
    this.cols = options.rawOptions.cols;
    const rows = Array.from({length: options.rawOptions.rows}, () => new BufferLine(this.cols, CellData.fromCharData([0, '', 1, 0])));
    this.buffer = { ybase: 0, y: 0, lines: { get(i) { return rows[i]; } }, setWrapped(i, value) { if (rows[i]) rows[i]._isWrapped = value; } };
  }
}
` + windowsTests + '\nglobalThis.classes = { BufferLine, CellData, DEFAULT_ATTR_DATA, updateWindowsModeWrappedState };', windowsContext);
vm.runInNewContext(versionSource + '\nif (XTERM_VERSION !== "6.0.0") throw Error("version mismatch");');
const trace = [], expected = [];
let env;
function initialize(idle) {
  let token = 0; const callbacks = new Map(); const events = []; const clock = { value: 0 };
  const request = kind => callback => { const id = ++token; callbacks.set(id, callback); events.push(`request:${id}:${kind}`); return id; };
  const cancel = id => { callbacks.delete(id); events.push(`cancel:${id}`); };
  const context = vm.createContext({ performance: { now: () => clock.value }, setTimeout: request('Priority'), clearTimeout: cancel,
    ...(idle ? { requestIdleCallback: request('Idle'), cancelIdleCallback: cancel } : {}) });
  vm.runInContext(queueSource + '\nglobalThis.Queue = IdleTaskQueue;', context);
  const queue = new context.Queue({ warn: message => events.push(`warn:${message}`) });
  env = { queue, events, clock, callbacks };
}
function enqueue(id, repeats, duration, action) {
  let calls = 0;
  env.queue.enqueue(() => {
    env.events.push(`task:${id}`); env.clock.value += duration;
    if (calls++ === 0) {
      if (action === 2 || action === 3) env.queue.clear();
      if (action === 1 || action === 3) enqueue(id + 10000, 1, 0, 0);
    }
    return calls < repeats;
  });
}
function operation(line) {
  trace.push(line); const f = line.split('\t'); let answer;
  switch (f[0]) {
    case 'I': initialize(f[1] === '1'); break;
    case 'E': case 'S': if (f[0] === 'S') env.queue.clear(); enqueue(+f[1],+f[2],+f[3],+f[4]); break;
    case 'F': env.queue.flush(); break;
    case 'C': env.queue.clear(); break;
    case 'D': case 'Y': {
      const id = env.queue._idleCallback; const callback = env.callbacks.get(id);
      assert(callback, `No pending callback: ${line}`); env.callbacks.delete(id);
      const values = f[1]?.split(',').map(Number); let i = 0;
      callback({ timeRemaining: () => values[Math.min(i++, values.length-1)] }); break;
    }
    case 'P': {
      const navigator = f[2] === '1' ? { userAgent: Buffer.from(f[3],'hex').toString(), platform: Buffer.from(f[4],'hex').toString() } : undefined;
      const context = vm.createContext({ ...(f[1] === '1' ? { process: { title: '' } } : {}), ...(navigator ? { navigator } : {}) });
      try {
        answer = vm.runInContext(platformSource + '\n[isNode,isFirefox,isChrome,isLegacyEdge,isSafari,isMac,isWindows,isLinux,isChromeOS].map(Number).join("")+":"+getSafariVersion()', context);
      } catch (error) { assert.match(String(error), /navigator is not defined/); answer = 'error'; }
      break;
    }
    case 'W': {
      const { BufferLine, CellData, DEFAULT_ATTR_DATA, updateWindowsModeWrappedState } = windowsContext.classes;
      const rows = f[4].split(',').map((end, i) => {
        if (end === '-') return undefined;
        const row = new BufferLine(10, CellData.fromCharData([0,'',1,0]), f[5][i] === '1');
        const codes = end.split('+').map(Number); row.setCellFromCodepoint(9,codes[0],1,DEFAULT_ATTR_DATA);
        for (const cp of codes.slice(1)) row.addCodepointToCell(9,cp,0); return row;
      });
      const get = index => rows[index % rows.length];
      updateWindowsModeWrappedState({ cols: +f[1], buffer: { ybase:+f[2],y:+f[3],lines:{get},setWrapped(i,value) { const row=get(i); if(row) row._isWrapped=value; } } });
      answer = rows.map(row => row ? Number(row.isWrapped) : '-').join(''); break;
    }
    default: throw Error(`unknown command ${f[0]}`);
  }
  expected.push(answer ?? env.events.splice(0).join('|'));
}
// Boundaries are intentional. The random corpus supplements rather than
// replacing equality, backwards clocks, half-rounding and reentrant cases.
for (const idle of [0,1]) {
  operation(`I\t${idle}`);
  for (const duration of [0,-4,1,10,21,21.5,22.5]) {
    operation(`E\t1\t2\t${duration}\t0`); operation('E\t2\t1\t0\t0');
    if (idle) operation('D\t1,1.5,1.499'); else operation('Y');
    operation('F'); operation('C');
  }
  for (const action of [1,2,3]) { operation(`E\t3\t1\t0\t${action}`); operation('F'); }
  operation('S\t4\t1\t0\t0'); operation('S\t5\t3\t0\t0'); operation('F');
  operation('E\t6\t1\t0\t0'); operation(idle ? 'D\t0' : 'Y');
  if (env.queue._idleCallback) operation(idle ? 'D\t0' : 'Y');
  operation('C');
}
const seed = 0x715cad; let state = seed;
function random() { state = (Math.imul(state,1664525)+1013904223) >>> 0; return state; }
operation('I\t1');
for (let i=0;i<1500;i++) {
  switch(random()%6) {
    case 0: case 1: operation(`E\t${i}\t${random()%4+1}\t${(random()%101-20)/2}\t${random()%4}`); break;
    case 2: if (env.queue._idleCallback) operation(`D\t${random()%31},${random()%31},0`); break;
    case 3: operation('F'); break;
    case 4: operation('C'); break;
    case 5: operation(`S\t${i}\t${random()%3+1}\t${random()%30}\t0`); break;
  }
}
operation('F');
const uaParts = ['Firefox','Chrome','Edge','Safari','safari','SAFARI','android','ANDROID','CrOS','_CrOS','éCrOSé','Node.js/24','Version/17.3','Version/x','Version/18','\n','\r','\u2028','x',' '];
const platforms = ['Macintosh','MacIntel','MacPPC','Mac68K','Windows','Win16','Win32','WinCE','Linux x86_64','linux','Mac','node',''];
const encode = text => Buffer.from(text).toString('hex');
for (let i=0;i<2500;i++) {
  const ua = Array.from({length:random()%7},()=>uaParts[random()%uaParts.length]).join(' ');
  operation(`P\t${random()%2}\t${random()%8===0?0:1}\t${encode(ua)}\t${encode(platforms[random()%platforms.length])}`);
}
for (let i=0;i<500;i++) {
  const codes = [0,32,9,97,8192,128512,'97+32','97+769','-'];
  const ends = Array.from({length:5},()=>codes[random()%codes.length]);
  const wraps = Array.from({length:5},()=>random()%2).join('');
  operation(`W\t${[0,9,10,11][random()%4]}\t${random()%3}\t${random()%3}\t${ends.join(',')}\t${wraps}`);
}
function run(command,args,options={}) {
  const result = spawnSync(command,args,{cwd:root,encoding:'utf8',timeout:120000,maxBuffer:16*1024*1024,...options});
  const log = `command-${attempt.commands.length}.log`;
  writeFileSync(path.join(out,log),(result.stdout??'')+(result.stderr??''));
  attempt.commands.push({command,args,cwd:options.cwd??root,timeout_ms:120000,
    status:result.status===0?'passed':'failed',exit_code:result.status,
    log,error:result.error?String(result.error):null}); saveAttempt();
  if(result.error) throw result.error;
  assert.equal(result.status,0,`${command} ${args.join(' ')}\n${result.stdout}\n${result.stderr}`); return result.stdout;
}
const originalDirectory = path.join(root,'source_repo');
run(process.execPath,['bin/esbuild.mjs'],{cwd:originalDirectory});
const originalRaw = run(process.execPath,['node_modules/mocha/bin/mocha.js',
  'out-esbuild/common/WindowsMode.test.js','--reporter','json'],{cwd:originalDirectory});
writeFileSync(path.join(out,'original-windows.json'),originalRaw);
const original = JSON.parse(originalRaw);
assert.equal(original.stats.tests,3); assert.equal(original.stats.passes,3); assert.equal(original.stats.failures,0);
const cargo = process.env.CARGO || 'cargo';
run(cargo,['build','-p','xterm-core','--example','task_platform_trace']);
const binary = path.join(root,'target/debug/examples',process.platform==='win32'?'task_platform_trace.exe':'task_platform_trace');
const actual = run(binary,[],{input:trace.join('\n')+'\n'}).replace(/\r\n/g,'\n').split('\n'); actual.pop();
try { assert.deepStrictEqual(actual,expected); }
catch(error) {
  const mismatch = expected.findIndex((value,index)=>value!==actual[index]);
  writeFileSync(path.join(out,'counterexample.json'),JSON.stringify({seed,mismatch,trace,expected,actual},null,2)); throw error;
}
const implementationHashes = Object.fromEntries([
  'crates/xterm-core/src/task_queue.rs','crates/xterm-core/src/platform.rs',
  'crates/xterm-core/src/windows_mode.rs','crates/xterm-core/src/version.rs',
  'crates/xterm-core/examples/task_platform_trace.rs','scripts/task-platform-oracle.mjs',
].map(file => [file, createHash('sha256').update(readFileSync(path.join(root,file))).digest('hex')]));
const report = { status:'scoped checks passed; full equivalence NOT PROVEN',command:'node scripts/task-platform-oracle.mjs',
  environment:{node:process.version,platform:process.platform,cargo:run(cargo,['--version']).trim(),rustc:run(process.env.RUSTC || 'rustc',['--version']).trim()},sourceHashes:hashes,implementationHashes,
  trace_sha256:createHash('sha256').update(trace.join('\n')+'\n').digest('hex'),
  original:{windows_mode:{discovered:original.stats.tests,executed:original.stats.passes+original.stats.failures,
    passed:original.stats.passes,failed:original.stats.failures,skipped:0,pending:original.stats.pending,unrun:0,
    names:original.tests.map(test=>test.fullTitle),runner:'Actual Mocha suite after upstream esbuild, with real BufferService/OptionsService/BufferLine.'},
    supplemental_structural_fixture:{executed:originalNames.length,passed:originalNames.length,names:originalNames}, 
    task_queue:{discovered:0,executed:0,passed:0,failed:0,skipped:0,unrun:0},platform:{discovered:0,executed:0,passed:0,failed:0,skipped:0,unrun:0}},
  differential:{seed,queue_random_iterations:1500,platform_cases:2500,windows_cases:500,total_operations:trace.length,compared_results:actual.length,mismatches:0},
  gaps:['Live browser timer/idle adapter tested separately in task84.','Real Windows linefeed/resize/copy integration tested separately in task28.','Native Linux/macOS runtime not observed here.','JavaScript exceptions vs Rust callback panics are not yet differential-tested.'] };
writeFileSync(path.join(out,'oracle.json'),JSON.stringify(report,null,2)+'\n');
attempt.status='passed';attempt.completed=new Date().toISOString();saveAttempt();
console.log(JSON.stringify(report,null,2));
