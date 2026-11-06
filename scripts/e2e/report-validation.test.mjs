import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { validateTestReport } from './upstream-config.mjs';

const report = (...tests) => ({ errors: [], suites: [{ specs: [{ tests }] }] });
const passed = () => ({ expectedStatus: 'passed', results: [{ status: 'passed' }] });

test('runtime accepts complete results, original skips and expected failures', () => {
  assert.deepEqual(validateTestReport(report(passed(),
    { expectedStatus: 'skipped', results: [{ status: 'skipped' }] },
    { expectedStatus: 'failed', results: [{ status: 'failed' }] })),
  { discovered: 3, executed: 2, passed: 1, failed: 1, skipped: 1, unrun: 0 });
  assert.equal(validateTestReport(report(passed(), {
    expectedStatus: 'passed', annotations: [{ type: 'fixme' }], results: [{ status: 'skipped' }]
  })).skipped, 1);
});

test('discovery accepts unexecuted cases but does not pretend runtime ran them', () => {
  const raw = report({ expectedStatus: 'passed', results: [] });
  assert.equal(validateTestReport(raw, { list: true }).unrun, 1);
  assert.throws(() => validateTestReport(raw), /unrun/);
});

test('one passing case cannot hide missing results or beforeAll fallout', () => {
  for (const incomplete of [{}, { results: [] }, { expectedStatus: 'passed', results: [{ status: 'skipped' }] }]) {
    assert.throws(() => validateTestReport(report(passed(), incomplete)), /unrun/);
  }
});

test('unknown or malformed results fail even beside a passing case', () => {
  for (const result of [{ status: 'success' }, {}, null, 'passed']) {
    assert.throws(() => validateTestReport(report(passed(), { results: [result] })), /result status/);
  }
  assert.throws(() => validateTestReport(report(passed(), { results: 'passed' })), /results/);
});

test('failed, timed out, interrupted and unexpected passes cannot report success', () => {
  for (const status of ['failed', 'timedOut', 'interrupted']) {
    assert.throws(() => validateTestReport(report(passed(), {
      expectedStatus: 'passed', results: [{ status }]
    })), /Unexpected result/);
  }
  assert.throws(() => validateTestReport(report({ expectedStatus: 'failed', results: [{ status: 'passed' }] })), /Unexpected result/);
});

test('invalid report structures and suite errors fail closed', () => {
  for (const raw of [null, {}, { suites: {} }, { suites: [null] },
    { suites: [{ specs: {} }] }, { suites: [{ specs: null }] }, { suites: [{ suites: null }] },
    { suites: [{ specs: [{ tests: {} }] }] }, { ...report(passed()), errors: null },
    { ...report(passed()), errors: 'not an array' },
    { ...report(passed()), errors: [{ message: 'browser crashed' }] }]) {
    assert.throws(() => validateTestReport(raw));
  }
  assert.throws(() => validateTestReport(report()), /No tests/);
  assert.throws(() => validateTestReport(report({ expectedStatus: 'skipped', results: [{ status: 'skipped' }] })), /No browser assertions/);
});

test('both actual entrypoints validate before assigning a successful status', async () => {
  for (const name of ['e2e-upstream.mjs', 'demo-e2e.mjs']) {
    const source = await readFile(new URL(`../${name}`, import.meta.url), 'utf8');
    assert.match(source, /validateTestReport\(raw, \{ list: options\.list \}\)/);
    assert.match(source, /'scripts\/e2e\/upstream-config\.mjs'/);
    assert.ok(source.indexOf('validateTestReport(raw,') < source.indexOf("report.status = options.list ?"));
  }
});
