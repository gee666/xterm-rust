// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Runs unchanged upstream assertions and emits DCS traces.
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import path from 'node:path';
import vm from 'node:vm';
import { parserOracle } from './parser-oracle-common.mjs';
import { canonicalTsvRecords } from './tsv-records.mjs';

const oracle = await parserOracle('Dcs');
const { Parser: DcsParser, Params } = oracle.context;
const utf32ToString = (data, start, end) => oracle.utf32String(Array.from(data.slice(start, end)));
const units = text => Array.from({ length: text.length }, (_, i) => text.charCodeAt(i)).join(',');
// Independent review counterexamples. All four source callback loops throw
// when a callback removes entries needed by the reverse traversal.
const mutationRegressions = [];
for (const stage of ['hook', 'put', 'unhook', 'reset']) {
  const parser = new DcsParser();
  const tokens = [];
  const remove = current => {
    if (current === stage) for (const token of tokens) token.dispose();
  };
  const plain = { hook() {}, put() {}, unhook() { return false; } };
  tokens.push(parser.registerHandler(0x2b70, { ...plain }));
  tokens.push(parser.registerHandler(0x2b70, { ...plain }));
  tokens.push(parser.registerHandler(0x2b70, {
    hook() { remove('hook'); }, put() { remove('put'); },
    unhook(success) { remove(success ? 'unhook' : 'reset'); return false; }
  }));
  let observed;
  try {
    parser.hook(0x2b70, Params.fromArray([1, 2, 3]));
    if (stage === 'put') parser.put(Uint32Array.of(65), 0, 1);
    if (stage === 'unhook') parser.unhook(true);
    if (stage === 'reset') parser.reset();
  } catch (error) { observed = error; }
  assert.equal(observed?.name, 'TypeError', `missing source mutation error at ${stage}`);
  mutationRegressions.push({ stage, error: observed.name, message: observed.message, passed: true });
}
const callbackRegressions = [];
{
  const parser = new DcsParser();
  const calls = [];
  parser.registerHandler(0x2b70, { hook() {}, put() {}, unhook(success) { calls.push(`older:${success}`); return true; } });
  parser.registerHandler(0x2b70, {
    hook() {}, put() {},
    unhook(success) { calls.push(`newer:${success}`); if (success) throw new Error('callback failure'); return false; }
  });
  parser.hook(0x2b70, Params.fromArray([1]));
  assert.throws(() => parser.unhook(true), /callback failure/);
  assert.deepEqual(calls, ['newer:true']);
  parser.reset();
  assert.deepEqual(calls, ['newer:true', 'newer:false', 'older:false']);
  callbackRegressions.push({ name: 'exception stops bubbling, reset recovers', passed: true });
}
{
  const DcsHandler = vm.runInContext('DcsHandler', oracle.context);
  const calls = [];
  const handler = new DcsHandler((data, params) => {
    calls.push([units(data), JSON.stringify(params.toArray())]);
    if (calls.length === 1) throw new Error('callback failure');
    return true;
  });
  handler.hook(Params.fromArray([1]));
  handler.put(Uint32Array.of(65), 0, 1);
  assert.throws(() => handler.unhook(true), /callback failure/);
  handler.put(Uint32Array.of(66), 0, 1);
  assert.equal(handler.unhook(true), true);
  assert.deepEqual(calls, [['65', '[1]'], ['65,66', '[1]']]);
  callbackRegressions.push({ name: 'exception retains buffered data and Params', passed: true });
}
{
  const DcsHandler = vm.runInContext('DcsHandler', oracle.context);
  const calls = [];
  const handler = new DcsHandler((data, params) => {
    calls.push([units(data), JSON.stringify(params.toArray())]);
    const fail = calls.length === 1;
    return oracle.resolvedPromise(() => { if (fail) throw new Error('callback rejection'); return true; });
  });
  handler.hook(Params.fromArray([1]));
  handler.put(Uint32Array.of(65), 0, 1);
  await assert.rejects(handler.unhook(true), /callback rejection/);
  handler.put(Uint32Array.of(66), 0, 1);
  assert.equal(await handler.unhook(true), true);
  assert.equal(await handler.unhook(true), true);
  assert.deepEqual(calls, [['65', '[1]'], ['65,66', '[1]'], ['', '[0]']]);
  callbackRegressions.push({ name: 'rejected future retains data until successful resolution', passed: true });
}
{
  const parser = new DcsParser();
  const calls = [];
  let fail = true;
  parser.registerHandler(0x2b70, { hook() {}, put() {}, unhook(success) { calls.push(`older:${success}`); return true; } });
  parser.registerHandler(0x2b70, { hook() {}, put() {}, unhook() { if (fail) { fail = false; throw new Error('reset failure'); } return false; } });
  parser.hook(0x2b70, Params.fromArray([1]));
  assert.throws(() => parser.reset(), /reset failure/);
  assert.deepEqual(calls, []);
  parser.reset();
  assert.deepEqual(calls, ['older:false']);
  callbackRegressions.push({ name: 'reset exception stops older cleanup, retry recovers', passed: true });
}
{
  const parser = new DcsParser();
  const calls = [];
  parser.registerHandler(0x2b70, {
    hook() {
      calls.push('register:H');
      parser.registerHandler(0x2b70, {
        hook() { calls.push('new:H'); }, put() { calls.push('new:P:65'); },
        unhook(success) { calls.push(`new:U:${success}`); return true; }
      });
    },
    put() { calls.push('register:P'); }, unhook(success) { calls.push(`register:U:${success}`); return false; }
  });
  parser.hook(0x2b70, Params.fromArray([1]));
  parser.put(Uint32Array.of(65), 0, 1);
  parser.unhook(true);
  assert.deepEqual(calls, ['register:H', 'new:P:65', 'register:P', 'new:U:true', 'register:U:false']);
  callbackRegressions.push({ name: 'callback registration mutates live list without repeated hook', passed: true });
}
const fallbackRegressions = [];
for (const phase of ['HOOK', 'PUT', 'UNHOOK']) {
  const parser = new DcsParser();
  const cause = new Error('exact diagnostic failure');
  const reports = [];
  let fail = true;
  parser.setHandlerFallback((ident, action) => {
    reports.push(ident);
    if (action === phase && fail) { fail = false; throw cause; }
  });
  let observed;
  const hook = () => parser.hook(0x2b70, Params.fromArray([1]));
  try {
    hook();
    if (phase === 'PUT') parser.put(Uint32Array.of(65), 0, 1);
    if (phase === 'UNHOOK') { parser.put(Uint32Array.of(65), 0, 1); parser.unhook(true); }
  } catch (error) { observed = error; }
  assert.equal(observed, cause, `fallback ${phase} must preserve exception identity`);
  parser.put(Uint32Array.of(66), 0, 1);
  parser.unhook(true);
  parser.put(new Uint32Array(), 0, 0);
  assert.equal(reports.at(-1), 0);
  assert(reports.slice(0, -1).every(ident => ident === 0x2b70));
  fallbackRegressions.push({ phase, identReports: reports, preservedCause: true, passed: true });
}
{
  const parser = new DcsParser();
  const cause = new Error('resumed failure');
  const calls = [];
  for (const [name,mode] of [['old','plain'],['middle','throw'],['new','pending']]) parser.registerHandler(0x2b70, {
    hook() {}, put() {}, unhook(success) {
      const report = () => { calls.push(`${name}:U:${success}`); return false; };
      if (mode === 'pending') return oracle.resolvedPromise(report);
      report(); if (mode === 'throw' && success) throw cause; return false;
    }
  });
  parser.hook(0x2b70, Params.fromArray([1]));
  assert.equal(await parser.unhook(true), false);
  assert.throws(() => parser.unhook(true, false), error => error === cause);
  assert.equal(parser._stack.paused, false);
  parser.reset(); await oracle.resolvedPromise(() => {});
  assert.deepEqual(calls, ['new:U:true','middle:U:true','middle:U:false','old:U:false','new:U:false']);
  callbackRegressions.push({ name: 'resumed lower failure clears inner pause before reset', calls, passed: true });
}
const rawStreamingRegressions = [];
for (const phase of ['HOOK', 'PUT']) {
  const parser = new DcsParser();
  const cause = new Error('exact raw failure');
  const calls = [];
  for (const name of ['old', 'new']) parser.registerHandler(0x2b70, {
    hook() { calls.push(`${name}:H`); if (name === 'new' && phase === 'HOOK') throw cause; },
    put() { calls.push(`${name}:P`); if (name === 'new' && phase === 'PUT') throw cause; },
    unhook(success) { calls.push(`${name}:U:${success}`); return false; }
  });
  let observed;
  try { parser.hook(0x2b70, Params.fromArray([1])); if (phase === 'PUT') parser.put(Uint32Array.of(65,66),0,2); }
  catch (error) { observed = error; }
  assert.equal(observed, cause);
  const expected = phase === 'HOOK' ? ['new:H'] : ['new:H','old:H','new:P'];
  assert.deepEqual(calls, expected);
  parser.reset();
  assert.deepEqual(calls, [...expected,'new:U:false','old:U:false']);
  rawStreamingRegressions.push({ phase, calls, preservedCause: true, passed: true });
}
const disposalRegressions = [];
for (const phase of ['HOOK','PUT','UNHOOK','RESET']) {
  const parser = new DcsParser();
  const current = method => { if(method===phase) parser.dispose(); };
  parser.registerHandler(0x2b70,{hook(){},put(){},unhook(){return false;}});
  parser.registerHandler(0x2b70,{
    hook(){current('HOOK');},put(){current('PUT');},unhook(success){current(success?'UNHOOK':'RESET');return false;}
  });
  assert.throws(()=>{
    parser.hook(0x2b70,Params.fromArray([1]));
    if(phase==='PUT') parser.put(Uint32Array.of(65),0,1);
    if(phase==='UNHOOK') parser.unhook(true);
    if(phase==='RESET') parser.reset();
  },{name:'TypeError'});
  disposalRegressions.push({phase,error:'TypeError',passed:true});
}
{
  const parser=new DcsParser(); const calls=[];
  const fresh={hook(p){calls.push(`new:H:${p.toArray()}`);},put(data){calls.push(`new:P:${data[0]}`);},unhook(success){calls.push(`new:U:${success}`);return true;}};
  parser.registerHandler(0x2b70,{hook(){calls.push('H');parser.dispose();parser.registerHandler(0x2b70,fresh);},put(){calls.push('old:P');},unhook(){return false;}});
  parser.hook(0x2b70,Params.fromArray([1]));parser.put(Uint32Array.of(65),0,1);
  assert.deepEqual(calls,['H']);
  parser.hook(0x2b70,Params.fromArray([2]));parser.put(Uint32Array.of(66),0,1);parser.unhook(true);
  assert.deepEqual(calls,['H','new:H:2','new:P:66','new:U:true']);
  parser.dispose();parser.hook(0x2b70,Params.fromArray([3]));parser.put(Uint32Array.of(67),0,1);
  assert.equal(calls.length,4);
  disposalRegressions.push({phase:'post-dispose-registration',calls,passed:true});
}
{
  const parser=new DcsParser();const calls=[];
  parser.setHandlerFallback((id)=>{calls.push(`fallback:${id}`);parser.dispose();parser.registerHandler(0x2b70,{hook(){calls.push('new:H');},put(){calls.push('new:P');},unhook(){return true;}});});
  parser.hook(0x2b70,Params.fromArray([1]));parser.put(Uint32Array.of(65),0,1);
  assert.deepEqual(calls,['fallback:11120']);
  parser.hook(0x2b70,Params.fromArray([2]));parser.put(Uint32Array.of(66),0,1);
  assert.deepEqual(calls,['fallback:11120','new:H','new:P']);
  disposalRegressions.push({phase:'fallback-self-dispose',calls,passed:true});
}
const seed = 0xdcc512;
let state = seed;
function next() {
  state ^= state << 13; state ^= state >>> 17; state ^= state << 5;
  return state >>> 0;
}
const traces = [];
for (let iteration = 0; iteration < 512; iteration++) {
  const parser = new DcsParser();
  const reports = [];
  const operations = [];
  const disposables = [];
  const asynchronous = iteration % 2 === 0;
  parser.setHandlerFallback((ident, action, data) => reports.push(`${ident}:${
    action === 'HOOK' ? `H:${data.toArray().join(',')}` :
      action === 'PUT' ? `P:${units(data)}` : `U:${data}`
  }`));
  function register() {
    const label = `h${disposables.length}`;
    const handled = !!(next() % 2);
    const isAsync = asynchronous && !!(next() % 2);
    operations.push(`r:${label}:${+handled}:${+isAsync}`);
    const handler = {
      hook(params) { reports.push(`${label}:H:${params.toArray().join(',')}`); },
      put(data, start, end) { reports.push(`${label}:P:${units(utf32ToString(data, start, end))}`); },
      unhook(success) {
        if (isAsync) return oracle.resolvedPromise(() => {
          reports.push(`${label}:U:${success}`); return handled;
        });
        reports.push(`${label}:U:${success}`); return handled;
      }
    };
    disposables.push(parser.registerHandler(0x2b70, handler));
  }
  for (let i = next() % 6; i > 0; i--) register();
  for (let round = 0; round < 4; round++) {
    operations.push('h'); parser.hook(0x2b70, Params.fromArray([1, 2, 3]));
    for (let chunk = 0; chunk < 5; chunk++) {
      const choice = next() % 10;
      if (choice === 0) { operations.push('c'); parser.clearHandler(0x2b70); }
      if (choice === 1) register();
      if (choice === 2 && disposables.length) {
        const index = next() % disposables.length;
        operations.push(`d:${index}`); disposables[index].dispose();
      }
      const data = Array.from({ length: next() % 8 }, () => [0, 65, 0xd800, 0x10000, 0x10ffff, 0xffffffff][next() % 6]);
      operations.push(`p:${data.join(',')}`);
      parser.put(Uint32Array.from(data), 0, data.length);
    }
    if (!asynchronous && next() % 4 === 0) {
      operations.push('x'); parser.reset();
    } else {
      const success = !!(next() % 2);
      operations.push(`u:${+success}`);
      let previous = true;
      let result;
      while ((result = parser.unhook(success, previous))) previous = await result;
    }
  }
  traces.push(`${operations.join('|')}\t${reports.join('|')}`);
}
const corpus = `${traces.join('\n')}\n`;
const fixture = 'crates/xterm-core/tests/fixtures/dcs-oracle.tsv';
await mkdir(path.dirname(fixture), { recursive: true });
if (process.argv.includes('--generate')) await writeFile(fixture, corpus);
const fixtureBytes = await readFile(fixture);
// Numeric TSV fields contain no literal newlines; normalize record endings only.
assert.equal(canonicalTsvRecords(fixtureBytes.toString('utf8')), corpus, 'DCS corpus differs from upstream');
const report = {
  node: process.version, sources: oracle.sources, seed, traces: traces.length, mutationRegressions, callbackRegressions, fallbackRegressions, rawStreamingRegressions, disposalRegressions,
  corpusSha256: createHash('sha256').update(corpus).digest('hex'),
  fixtureRawSha256: createHash('sha256').update(fixtureBytes).digest('hex'),
  upstream: { ...oracle.baseline, executed: oracle.baseline.discovered, unrun: 0 }
};
await mkdir('results/dcs', { recursive: true });
await writeFile('results/dcs/oracle.json', `${JSON.stringify(report, null, 2)}\n`);
console.log(JSON.stringify({ ...report, upstream: { ...report.upstream, outcomes: undefined }, sources: undefined }, null, 2));
assert.equal(report.upstream.failed, 0);
assert.equal(report.upstream.discovered, 20, 'upstream test inventory changed');
