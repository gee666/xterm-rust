import { dirname } from 'node:path';

export function configForRust(originalConfig, origin) {
  return `const loaded = require(${JSON.stringify(originalConfig)});\n`
    + 'const original = loaded.default ?? loaded;\n'
    + 'const path = require("node:path");\n'
    + `module.exports = { ...original, testDir: path.resolve(${JSON.stringify(dirname(originalConfig))}, original.testDir ?? '.'),\n`
    + `use: { ...original.use, baseURL: ${JSON.stringify(origin)} }, webServer: undefined };\n`;
}

// An exit code alone cannot prove that every discovered case has a valid result.
// Count first in callers so rejected reports still retain their partial accounting.
export function validateTestReport(raw, { list = false } = {}) {
  const statuses = ['passed', 'failed', 'timedOut', 'skipped', 'interrupted'];
  const array = (value, name) => {
    if (!Array.isArray(value)) throw new Error(`Invalid Playwright ${name}: expected an array`);
    return value;
  };
  if (!raw || typeof raw !== 'object') throw new Error('Invalid Playwright report');
  const optionalArray = (value, name) => array(value === undefined ? [] : value, name);
  if (optionalArray(raw.errors, 'errors').length) throw new Error('Playwright reported suite errors');
  function visit(suites) {
    for (const suite of array(suites, 'suites')) {
      if (!suite || typeof suite !== 'object') throw new Error('Invalid Playwright suite');
      for (const spec of optionalArray(suite.specs, 'specs')) {
        for (const test of array(spec?.tests, 'tests')) {
          if (!test || typeof test !== 'object') throw new Error('Invalid Playwright test');
          const results = optionalArray(test.results, 'results');
          for (const result of results) {
            if (!result || !statuses.includes(result.status)) throw new Error('Invalid Playwright result status');
          }
          const result = results.at(-1);
          if (list || !result) continue;
          const declaredSkip = test.expectedStatus === 'skipped'
            || test.annotations?.some(annotation => ['skip', 'fixme'].includes(annotation.type));
          if (result.status === 'skipped' && declaredSkip) continue;
          if (result.status !== 'skipped' && result.status !== (test.expectedStatus ?? 'passed')) {
            throw new Error(`Unexpected result ${result.status}, expected ${test.expectedStatus ?? 'passed'}`);
          }
        }
      }
      visit(suite.suites === undefined ? [] : suite.suites);
    }
  }
  visit(raw.suites);
  const counts = testCounts(raw);
  if (!counts.discovered) throw new Error('No tests discovered');
  if (!list && counts.unrun) throw new Error(`${counts.unrun} discovered tests are unrun`);
  if (!list && !counts.executed) throw new Error('No browser assertions executed');
  return counts;
}

export function testCounts(raw) {
  const counts = { discovered: 0, executed: 0, passed: 0, failed: 0, skipped: 0, unrun: 0 };
  function visit(suites) {
    for (const suite of suites) {
      for (const spec of suite.specs ?? []) for (const test of spec.tests ?? []) {
        counts.discovered++;
        const result = test.results?.at(-1);
        if (!result) counts.unrun++;
        else if (result.status === 'skipped') {
          const declaredSkip = test.expectedStatus === 'skipped' || test.annotations?.some(annotation => ['skip', 'fixme'].includes(annotation.type));
          if (declaredSkip) counts.skipped++; else counts.unrun++;
        }
        else {
          counts.executed++;
          if (result.status === 'passed') counts.passed++; else counts.failed++;
        }
      }
      visit(suite.suites ?? []);
    }
  }
  visit(raw.suites ?? []);
  return counts;
}
