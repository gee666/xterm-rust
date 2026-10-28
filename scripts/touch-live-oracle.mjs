// Execute unchanged touch.ts with adversarial host fields. This independent
// loader intentionally leaves the accepted normal-policy oracle/fixtures intact.
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
import vm from 'node:vm';
const require = createRequire(import.meta.url);
const esbuild = require('../source_repo/node_modules/esbuild');
const build = esbuild.buildSync({ entryPoints: ['source_repo/src/browser/scrollable/touch.ts'],
  bundle: true, write: false, format: 'cjs', platform: 'node', metafile: true,
  tsconfigRaw: { compilerOptions: { experimentalDecorators: true, useDefineForClassFields: false } } });
const driverPath = 'tools/touch_live/driver.js';
const driver = await readFile(driverPath, 'utf8');
function makeHost() {
  const native = new WeakMap();
  const host = { log: [], labels: new WeakMap(), meta: new WeakMap(), time: 1000,
    tapTime: undefined, recording: false, frames: new Map(), next: 0 };
  const error = value => {
    if (!host.recording) throw value;
    const tag = host.labels.get(value);
    host.log.push('error:' + (tag ? '@' + tag : value?.name ?? typeof value));
  };
  class Node {
    constructor(parent = null, element = true) { this.parentNode = parent; this.element = element; this.parentElement = parent?.element ? parent : null; this.listeners = new Map(); }
    contains(node) { for (; node; node = node.parentNode) if (node === this) return true; return false; }
    addEventListener(type, callback) {
      const entries = this.listeners.get(type) ?? [];
      if (!entries.some(entry => entry.callback === callback && entry.active)) entries.push({ callback, active:true });
      this.listeners.set(type, entries);
    }
    removeEventListener(type, callback) {
      for (const entry of this.listeners.get(type) ?? []) if (entry.callback === callback) entry.active = false;
    }
    dispatchEvent(event) {
      const state = native.get(event);
      for (const entry of [...(this.listeners.get(state.kind) ?? [])]) {
        if (!entry.active) continue;
        try { entry.callback(event); } catch (value) { error(value); }
      }
      return !state.prevented;
    }
  }
  function event() {
    const result = {};
    const state = { kind:'', bubbles:false, cancelable:false, prevented:false };
    native.set(result, state);
    Object.defineProperties(result, {
      type: { configurable:true, get:()=>state.kind },
      bubbles: { get:()=>state.bubbles }, cancelable: { get:()=>state.cancelable },
      defaultPrevented: { get:()=>state.prevented }
    });
    result.initEvent = (kind,bubbles,cancelable) => Object.assign(state, { kind,bubbles,cancelable });
    result.preventDefault = () => { if (state.cancelable) state.prevented = true; };
    result.stopPropagation = () => {};
    return result;
  }
  host.document = new Node(null,false);
  host.html = new Node(host.document); host.body = new Node(host.html);
  host.root = new Node(host.body); host.child = new Node(host.root);
  host.leaf = new Node(host.child); host.outside = new Node(host.body);
  for (const label of ['document','html','body','root','child','leaf','outside']) host.labels.set(host[label], label);
  host.document.createEvent = event;
  host.installCreate = factory => { host.document.createEvent = type => type === 'CustomEvent' ? factory(event()) : event(); };
  host.input = kind => { const result = event(); result.initEvent(kind,false,true); return result; };
  host.dispatch = event => host.document.dispatchEvent(event);
  host.listen = (node, kind, callback) => node.addEventListener(kind,callback);
  host.flush = () => { const frames = [...host.frames.values()]; host.frames.clear(); for (const callback of frames) callback(0); };
  host.pending = () => host.frames.size;
  const clock = (kind, hook, value) => {
    if (host.recording) host.log.push('clock:' + kind + ':enter');
    if (host[hook]) host[hook]();
    const result = value();
    if (host.recording) host.log.push('clock:' + kind + ':' + host.encode(result));
    return result;
  };
  class Date {
    constructor() { if (host.recording) host.log.push('clock:new'); }
    static now() { return clock('now','nowHook',()=>host.time); }
    getTime() { return clock('tap','tapHook',()=>host.tapTime ?? host.time); }
  }
  const console = {
    warn(text, touch) { if (host.recording) host.log.push('warn:' + text + ':' + host.encode(touch)); },
    error
  };
  host.window = { document:host.document, ontouchstart:null,
    requestAnimationFrame(callback) { const id=++host.next; host.frames.set(id,callback); return id; },
    cancelAnimationFrame(id) { host.frames.delete(id); }
  };
  return { host, Node, Date, console };
}
const namesContext = vm.createContext({});
vm.runInContext(driver + ';globalThis.names=TOUCH_LIVE_CASES;', namesContext);
const names = [...namesContext.names];
const directed = names.length;
const seed = 0x746f7563;
let state = seed;
for (let index=0;index<64;index++) {
  state ^= state<<13; state ^= state>>>17; state ^= state<<5;
  names.push('seed:' + (state>>>0));
}
const cases = [];
for (const name of names) {
  const fixture = makeHost();
  const exports = {};
  const context = vm.createContext({ ...fixture, window:fixture.host.window, document:fixture.host.document,
    navigator:{ maxTouchPoints:1 }, exports, module:{exports} });
  vm.runInContext(build.outputFiles[0].text, context, { filename:'original-touch-live.cjs' });
  vm.runInContext(driver + ';globalThis.run=runTouchScenario;', context, { filename:driverPath });
  const gesture = new context.module.exports.Gesture();
  const targets = name === 'initial-depth-reverse' ? ['child','root'] : ['root','child'];
  for (const target of targets) gesture._targets.push(fixture.host[target]);
  const result = context.run(fixture.host,name);
  cases.push({ name, targets, log:Array.from(result) });
  gesture.dispose();
}
const fixturePath = 'crates/xterm-web/tests/fixtures/touch-live.json';
const corpus = JSON.stringify(cases, null, 2) + '\n';
if (process.argv.includes('--generate')) {
  await mkdir('crates/xterm-web/tests/fixtures', {recursive:true});
  await writeFile(fixturePath,corpus);
} else {
  const current = (await readFile(fixturePath,'utf8')).replaceAll('\r\n','\n');
  if (current !== corpus) throw new Error('touch-live fixture drift; inspect source/counterexample, do not overwrite failures');
}
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const sources = {};
for (const path of Object.keys(build.metafile.inputs).sort()) sources[path] = hash(await readFile(path));
const report = { command:'node scripts/touch-live-oracle.mjs', node:process.version, platform:process.platform,
  sources, driverSha256:hash(driver), fixtureSha256:hash(corpus), seed:'0x' + seed.toString(16),
  directed, seeded:64, cases:cases.length, observations:cases.reduce((sum,entry)=>sum+entry.log.length,0),
  originalTouchSpecificSuites:0, scope:'Actual original Gesture/DOM scheduler with mock Node/Event and controlled clock/RAF; no browser pass claimed',
  hostSubstitutions:['mock DOM containment/listeners/CustomEvent records','controlled Date and RAF','fresh constructor/private target registration per scenario'],
  overallEquivalence:'NOT PROVEN', status:process.argv.includes('--generate')?'generated':'matched' };
await mkdir('results/touch-live',{recursive:true});
await writeFile('results/touch-live/oracle.json',JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(report,null,2));
