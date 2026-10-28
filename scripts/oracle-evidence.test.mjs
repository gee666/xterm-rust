import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { test } from 'node:test';
import { createEvidenceRun } from './upstream-baseline-utils.mjs';
import { runScopedOracle } from './oracle-evidence.mjs';

function temporary() {
  mkdirSync(resolve('tmp'), { recursive: true });
  return mkdtempSync(resolve('tmp/oracle-evidence-'));
}
function current(directory) {
  const pointer = JSON.parse(readFileSync(resolve(directory, 'latest.json'), 'utf8'));
  const report = JSON.parse(readFileSync(resolve(directory, pointer.summary), 'utf8'));
  return { pointer, report };
}
for (const phase of ['setup', 'build', 'parse', 'timeout']) {
  test(`${phase} failure replaces prior success and retains its actual cause`, async () => {
    const directory = temporary();
    try {
      await runScopedOracle(directory, { scope: 'negative control only' }, async () => ({ status: 'passed', count: 1 }));
      const previous = current(directory);
      let actualError;
      await assert.rejects(runScopedOracle(directory, { scope: 'negative control only' }, async context => {
        const initial = current(directory);
        assert.equal(initial.pointer.status, 'incomplete');
        assert.notEqual(initial.pointer.runId, previous.pointer.runId);
        context.checkpoint({ phase, argv: ['controlled-fixture', phase] });
        writeFileSync(resolve(context.directory, 'stderr.log'), `retained ${phase} failure\n`);
        try {
          if (phase === 'setup') await import(pathToFileURL(resolve(directory, 'missing-module.mjs')));
          else if (phase === 'parse') JSON.parse('{invalid');
          else if (phase === 'timeout') throw Object.assign(new Error('controlled timeout'), { code: 'ETIMEDOUT', signal: 'SIGTERM' });
          else throw Object.assign(new Error('controlled build exit'), { exitCode: 17 });
        } catch (error) { actualError = error; throw error; }
      }), error => error === actualError);
      const { pointer, report } = current(directory);
      assert.equal(pointer.status, 'failed');
      assert.equal(report.status, 'failed');
      assert.equal(report.phase, phase);
      assert.deepEqual(report.argv, ['controlled-fixture', phase]);
      assert.notEqual(pointer.runId, previous.pointer.runId);
      assert.equal(report.error.name, actualError.name);
      assert.equal(report.error.message, actualError.message);
      assert.equal(report.error.timedOut, phase === 'timeout');
      assert.equal(report.error.exitCode, phase === 'build' ? 17 : null);
      assert.match(readFileSync(resolve(directory, 'runs', pointer.runId, 'stderr.log'), 'utf8'), new RegExp(phase));
    } finally { rmSync(directory, { recursive: true, force: true }); }
  });
}

test('missing completion cannot become a successful no-op', async () => {
  const directory = temporary();
  try {
    await assert.rejects(runScopedOracle(directory, {}, async () => undefined), /completed passing report/);
    assert.equal(current(directory).pointer.status, 'failed');
  } finally { rmSync(directory, { recursive: true, force: true }); }
});

test('older success cannot replace a newer failed attempt', async () => {
  const directory = temporary();
  let release;
  try {
    const wait = new Promise(resolve => { release = resolve; });
    const older = runScopedOracle(directory, {}, async () => { await wait; return { status: 'passed' }; });
    const olderId = current(directory).pointer.runId;
    await assert.rejects(runScopedOracle(directory, {}, async () => { throw new Error('newer failed'); }));
    const newerId = current(directory).pointer.runId;
    assert.notEqual(olderId, newerId);
    release();
    await older;
    assert.equal(current(directory).pointer.runId, newerId);
    assert.equal(current(directory).pointer.status, 'failed');
    assert.equal(JSON.parse(readFileSync(resolve(directory, 'runs', olderId, 'summary.json'), 'utf8')).status, 'passed');
  } finally { rmSync(directory, { recursive: true, force: true }); }
});

test('original and persistence errors are retained together', async () => {
  const directory = temporary();
  const sentinel = new Error('original callback failure');
  try {
    await assert.rejects(runScopedOracle(directory, {}, async context => {
      rmSync(context.directory, { recursive: true });
      throw sentinel;
    }), error => error instanceof AggregateError && error.errors[0] === sentinel && error.errors[1].code === 'ENOENT');
    assert.equal(JSON.parse(readFileSync(resolve(directory, 'latest.json'), 'utf8')).status, 'incomplete');
  } finally { rmSync(directory, { recursive: true, force: true }); }
});

test('a checkpoint cannot promote partial success', async () => {
  const directory = temporary();
  try {
    await assert.rejects(runScopedOracle(directory, {}, async context => {
      context.checkpoint({ status: 'passed', phase: 'only baseline completed' });
      assert.equal(current(directory).pointer.status, 'incomplete');
      throw new Error('native phase did not pass');
    }));
    assert.equal(current(directory).pointer.status, 'failed');
  } finally { rmSync(directory, { recursive: true, force: true }); }
});

test('an interrupted new attempt leaves the current pointer incomplete', async () => {
  const directory = temporary();
  try {
    await runScopedOracle(directory, {}, async () => ({ status: 'passed' }));
    const previous = current(directory).pointer.runId;
    const next = createEvidenceRun(directory, { scope: 'interruption fixture' });
    assert.notEqual(next.id, previous);
    assert.equal(current(directory).pointer.status, 'incomplete');
    assert.equal(current(directory).report.status, 'incomplete');
  } finally { rmSync(directory, { recursive: true, force: true }); }
});
