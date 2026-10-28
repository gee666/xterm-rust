import test from 'node:test';
import assert from 'node:assert/strict';
import { Script } from 'node:vm';
import { configForRust, testCounts } from './upstream-config.mjs';

test('retains original browser projects, assertions directory and settings', () => {
  const original = { testDir: 'tests', projects: [{ name: 'WebKit', use: { browserName: 'webkit' } }],
    timeout: 10000, use: { deviceScaleFactor: 2 }, webServer: { command: 'npm start', port: 3000 } };
  const module = { exports: {} };
  new Script(configForRust('C:/reference/original.cjs', 'http://127.0.0.1:4567')).runInNewContext({
    require: name => name === 'node:path' ? { resolve: (...parts) => parts.join('/') } : { default: original }, module });
  assert.equal(module.exports.projects, original.projects);
  assert.equal(module.exports.timeout, original.timeout);
  assert.equal(module.exports.testDir, 'C:/reference/tests');
  assert.equal(module.exports.use.deviceScaleFactor, 2);
  assert.equal(module.exports.use.baseURL, 'http://127.0.0.1:4567');
  assert.equal(module.exports.webServer, undefined);
});
test('distinguishes discovered, executed, passed, failed, skipped and unrun', () => {
  const tests = [ {}, { results: [{ status: 'passed' }] }, { results: [{ status: 'failed' }] },
    { results: [{ status: 'skipped' }] }, { results: [{ status: 'timedOut' }] } ];
  assert.deepEqual(testCounts({ suites: [{ suites: [{ specs: [{ tests }] }] }] }),
    { discovered: 5, executed: 3, passed: 1, failed: 2, skipped: 1, unrun: 1 });
});
test('empty output cannot acquire a positive test count', () => {
  assert.deepEqual(testCounts({}), { discovered: 0, executed: 0, passed: 0, failed: 0, skipped: 0, unrun: 0 });
});
