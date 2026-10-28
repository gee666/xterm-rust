import { dirname } from 'node:path';

export function configForRust(originalConfig, origin) {
  return `const loaded = require(${JSON.stringify(originalConfig)});\n`
    + 'const original = loaded.default ?? loaded;\n'
    + 'const path = require("node:path");\n'
    + `module.exports = { ...original, testDir: path.resolve(${JSON.stringify(dirname(originalConfig))}, original.testDir ?? '.'),\n`
    + `use: { ...original.use, baseURL: ${JSON.stringify(origin)} }, webServer: undefined };\n`;
}

export function testCounts(raw) {
  const counts = { discovered: 0, executed: 0, passed: 0, failed: 0, skipped: 0, unrun: 0 };
  function visit(suites) {
    for (const suite of suites) {
      for (const spec of suite.specs ?? []) for (const test of spec.tests ?? []) {
        counts.discovered++;
        const result = test.results?.at(-1);
        if (!result) counts.unrun++;
        else if (result.status === 'skipped') counts.skipped++;
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
