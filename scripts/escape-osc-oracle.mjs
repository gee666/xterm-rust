// Copyright (c) 2018-2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Exact OSC-through-escape-parser callback traces.
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { escapeReference } from './escape-reference.mjs';
import { utf16Hex } from './parser-oracle-common.mjs';

const reference = await escapeReference();
const modes = [[], [[true, false]], [[false, false], [false, true], [true, false]],
  [[true, true], [false, false], [false, true]], [[false, false], [false, true]]];
const lines = [];
let pendingCount = 0;
let resetCount = 0;
async function trace(name, mode, operations) {
  const parser = new reference.context.EscapeSequenceParser();
  const log = [];
  parser.setPrintHandler((data, start, end) => log.push(`t:${utf16Hex(reference.utf32String(data.slice(start, end)))}`));
  parser.setExecuteHandlerFallback(code => log.push(`c:${code}`));
  parser.setEscHandlerFallback(id => log.push(`e:${id}`));
  parser.setCsiHandlerFallback((id, params) => log.push(`i:${id}:${JSON.stringify(params.toArray())}`));
  parser.setOscHandlerFallback((id, action, data) => log.push(`f:${id}:${action === 'START' ? 's' : action === 'END' ? `e:${Number(data)}` : `p:${utf16Hex(data)}`}`));
  for (const [name, [handled, asynchronous]] of modes[mode].entries()) {
    parser.registerOscHandler(2, {
      start() { log.push(`h:${name}`); },
      put(data, start, end) { log.push(`p:${name}:${utf16Hex(reference.utf32String(data.slice(start, end)))}`); },
      end(success) {
        const run = () => { log.push(`e:${name}:${Number(success)}`); return handled; };
        return asynchronous ? reference.resolvedPromise(run) : run();
      },
    });
  }
  for (const operation of operations) {
    if (operation === 'x') {
      parser.reset(); resetCount++;
      await reference.resolvedPromise(() => {});
    } else {
      const resetPending = !Array.isArray(operation);
      const chunk = Uint32Array.from(resetPending ? operation.pausedReset : operation);
      let previous;
      if (resetPending) {
        const pending = parser.parse(chunk, chunk.length);
        assert.ok(pending, 'pause/reset scenario must reach a pending handler');
        log.push('w'); pendingCount++;
        parser.reset(); resetCount++;
        previous = await pending;
        await reference.resolvedPromise(() => {});
      }
      let result;
      while (result = parser.parse(chunk, chunk.length, previous)) {
        log.push('w'); pendingCount++;
        previous = await result;
      }
      // A defined executor turn includes background cleanup promises created by
      // a nested OSC start, even when this parse call itself completed sync.
      await reference.resolvedPromise(() => {});
    }
    log.push(`s:${parser.currentState}:${parser.precedingJoinState}:${parser._collect}:${parser._params.toArray().join(',')}`);
  }
  lines.push([name, mode, operations.map(op => op === 'x' ? 'x' : Array.isArray(op) ? `p:${op.map(c => c.toString(16)).join(',')}` : `r:${op.pausedReset.map(c => c.toString(16)).join(',')}`).join(';'), log.join('|')].join('\t'));
}
const text = value => [...value].map(c => c.codePointAt(0));
const bodies = [text('2;hello'), text('0002;'), text('2'), text(';empty'), [],
  text('invalid;payload'), [50, 59, 0x1f600, 0xd800, 0xfeff, 0x4e2d],
  [50, 0, 59, 65, 8, 127, 66]];
const starts = [[27, 93], [0x9d]];
const endings = [[7], [0x9c], [27, 92], [0x18], [0x1a], [27, 81]];
for (const [startIndex, start] of starts.entries()) {
  for (const [bodyIndex, body] of bodies.entries()) {
    for (const [endIndex, end] of endings.entries()) {
      const input = [88, ...start, ...body, ...end, 89];
      for (let mode = 0; mode < modes.length; mode++) {
        const name = `intro${startIndex}-body${bodyIndex}-end${endIndex}-mode${mode}`;
        await trace(`${name}-whole`, mode, [input]);
        for (let split = 0; split <= input.length; split++) {
          await trace(`${name}-split${split}`, mode, [input.slice(0, split), input.slice(split)]);
        }
        await trace(`${name}-units`, mode, input.map(c => [c]));
      }
    }
  }
}
for (let mode = 0; mode < modes.length; mode++) {
  await trace(`reset-payload-mode${mode}`, mode, [[27, 93, 50, 59, 65], 'x', [66, 7, 67]]);
  await trace(`reset-identifier-mode${mode}`, mode, [[27, 93, 50], 'x', [65, 7]]);
  await trace(`restart-mode${mode}`, mode, [[27, 93, 50, 59, 65, 0x9d, 50, 59, 66, 7]]);
}
for (const mode of [2, 3, 4]) {
  await trace(`reset-paused-mode${mode}`, mode, [{ pausedReset: [27, 93, 50, 59, 65, 7, 89] }]);
}
const mutationChecks = [];
for (const phase of ['start', 'put', 'end', 'reset']) {
  const parser = new reference.context.EscapeSequenceParser();
  const tokens = [];
  const remove = method => { if (method === phase || phase === 'reset' && method === 'end') tokens.forEach(t => t.dispose()); };
  const handler = { start() { remove('start'); }, put() { remove('put'); }, end() { remove('end'); return false; } };
  tokens.push(parser.registerOscHandler(2, handler), parser.registerOscHandler(2, handler));
  const input = Uint32Array.from([27, 93, 50, 59, 65, 7]);
  assert.throws(() => {
    parser.parse(input, phase === 'reset' ? input.length - 1 : input.length);
    if (phase === 'reset') parser.reset();
  }, { name: 'TypeError' });
  mutationChecks.push({ phase, error: 'TypeError', rustError: 'ParseError::Dispatch(DispatchError::ActiveHandlerRemoved)' });
}
const seed = 0x05c91;
let state = seed;
const random = () => { state ^= state << 13; state ^= state >>> 17; state ^= state << 5; return state >>> 0; };
const chunks = [[], [27], [93], [0x9d], [50, 59], [65, 0x1f600, 0xd800], [0], [7], [0x9c], [27, 92], [0x18], [0x1a], [27, 91, 51, 109]];
for (let index = 0; index < 128; index++) {
  const operations = Array.from({ length: 32 }, () => random() % 16 === 0 ? 'x' : chunks[random() % chunks.length]);
  await trace(`seed${seed}-case${index}`, random() % modes.length, operations);
}
const corpus = `${lines.join('\n')}\n`;
const fixture = 'crates/xterm-core/tests/fixtures/escape-osc-oracle.tsv';
await mkdir('crates/xterm-core/tests/fixtures', { recursive: true });
if (process.argv.includes('--generate')) await writeFile(fixture, corpus);
else assert.equal(await readFile(fixture, 'utf8'), corpus, 'outer OSC oracle drift');
const report = { node: process.version, seed, deterministicTraces: lines.length - 128, randomTraces: 128,
  randomOperationsPerTrace: 32, pendingCount, resetCount, mutationChecks, sources: reference.sources,
  fixtureSha256: createHash('sha256').update(corpus).digest('hex'), status: 'matched' };
await mkdir('results/osc', { recursive: true });
await writeFile('results/osc/escape-oracle.json', `${JSON.stringify(report, null, 2)}\n`);
console.log(JSON.stringify(report, null, 2));
