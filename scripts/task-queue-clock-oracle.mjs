// Copyright (c) 2022 The xterm.js authors. All rights reserved. MIT license.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { stripTypeScriptTypes } from 'node:module';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const root = resolve(import.meta.dirname, '..');
const sourceRoot = process.env.XT_SOURCE_ROOT ?? resolve(root, 'source_repo');
const out = resolve(root, 'results/task-queue-clock');
await mkdir(out, { recursive: true });
const report = { status: 'running', node: process.version, cases: 0, browsers: [] };
const persist = () => writeFile(resolve(out, 'oracle.json'), JSON.stringify(report, null, 2) + '\n');
await persist();
try {
const source = await readFile(resolve(sourceRoot, 'src/common/TaskQueue.ts'), 'utf8');
const js = stripTypeScriptTypes(source.replace(/^import .*;\r?$/gm, '').replace(/^export /gm, ''), { mode: 'transform' });
const cases = [];
for (const mode of ['idle', 'priority']) {
  const start = mode === 'idle' ? 1 : 3;
  for (const action of ['replace', 'clear', 'flush', 'replace-error']) {
    for (const phase of ['start', 'end', 'second-start']) {
      cases.push({ mode, action, trigger: phase === 'start' ? start : phase === 'end' ? start + 1 : mode === 'idle' ? 3 : 6,
        initial: phase === 'second-start' ? ['prefix', 'old'] : ['old'] });
    }
  }
}
// Priority's deadline construction and initial remaining-time read precede the
// loop guard. Clearing there must complete normally, not report a missing task.
for (const trigger of [1, 2]) {
  for (const action of ['replace', 'clear', 'flush', 'replace-error']) {
    cases.push({ mode: 'priority', action, trigger, initial: ['old'] });
  }
}

// Serialized into real browser realms below as well as executed in Node. Only
// clocks/timer hosts are controlled; the queue implementation is unchanged TS.
function runCases(queueSource, inputs) {
  return inputs.map(input => {
    const trace = [], callbacks = [], sentinel = {};
    let queue, clock = 0, next = 0;
    const request = callback => { const id = ++next; callbacks.push(callback); trace.push(`request:${id}`); return id; };
    const cancel = id => trace.push(`cancel:${id}`);
    const host = { setTimeout: request, clearTimeout: cancel, requestIdleCallback: request, cancelIdleCallback: cancel,
      performance: { now() {
        trace.push(`now:${++clock}`);
        if (clock === input.trigger) {
          if (input.action === 'flush') queue.flush();
          else {
            queue.clear();
            if (input.action !== 'clear') queue.enqueue(() => {
              trace.push('task:new');
              if (input.action === 'replace-error') throw sentinel;
              return false;
            });
          }
        }
        return 0;
      } } };
    const classes = Function('host', `const globalThis=host; const {performance,setTimeout,clearTimeout,requestIdleCallback,cancelIdleCallback}=host;\n${queueSource}\nreturn {IdleTaskQueue,PriorityTaskQueue};`)(host);
    queue = new classes[input.mode === 'idle' ? 'IdleTaskQueue' : 'PriorityTaskQueue']({ warn() { throw Error('unexpected deadline warning'); } });
    for (const name of input.initial) queue.enqueue(() => { trace.push(`task:${name}`); return false; });
    let error = null;
    try { callbacks[0]({ timeRemaining: () => { trace.push('deadline'); return 100; } }); }
    catch (caught) {
      if (caught === sentinel) error = 'callback';
      else if (caught instanceof TypeError) error = 'missing-task';
      else throw caught;
    }
    return { trace, error, remaining: Math.max(0, queue._tasks.length - queue._i), pending: queue._idleCallback !== undefined };
  });
}
const expected = runCases(js, cases);
const fixture = { sourceSha256: createHash('sha256').update(source).digest('hex'), cases: cases.map((input, i) => ({ ...input, expected: expected[i] })) };
const fixturePath = resolve(root, 'crates/xterm-core/tests/fixtures/task-queue-clock.tsv');
const fixtureText = '# source SHA256 ' + fixture.sourceSha256 + '\n' + fixture.cases.map(c => [
  c.mode, c.action, c.trigger, c.initial.join(','), c.expected.trace.join('|'),
  c.expected.error ?? 'none', c.expected.remaining, c.expected.pending
].join('\t')).join('\n') + '\n';
if (process.argv.includes('--write')) {
  await mkdir(resolve(root, 'crates/xterm-core/tests/fixtures'), { recursive: true });
  await writeFile(fixturePath, fixtureText);
} else assert.equal(await readFile(fixturePath, 'utf8'), fixtureText);
report.sourceSha256 = fixture.sourceSha256;
report.cases = cases.length;
if (process.argv.includes('--browsers')) {
  const playwright = await import(pathToFileURL(process.env.PLAYWRIGHT_MODULE ?? resolve(sourceRoot, 'node_modules/playwright/index.mjs')));
  for (const name of ['chromium', 'firefox', 'webkit']) {
    const browser = await playwright[name].launch({ headless: true, timeout: 30000 });
    try {
      const page = await browser.newPage();
      page.setDefaultTimeout(30000);
      const actual = await page.evaluate(({ code, queueSource, inputs }) => Function(`return (${code})`)()(queueSource, inputs),
        { code: runCases.toString(), queueSource: js, inputs: cases });
      assert.deepEqual(actual, expected);
      report.browsers.push({ name, version: browser.version(), passed: cases.length, failed: 0 });
    } finally { await browser.close(); }
  }
}
report.status = 'passed';
console.log(JSON.stringify(report));
} catch (error) {
  report.status = 'failed';
  report.error = error.stack ?? String(error);
  process.exitCode = 1;
  console.error(report.error);
} finally {
  await persist();
}
