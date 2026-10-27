// Mocha metadata only. Original test bodies and reported file fields are unchanged.
// Imported test modules otherwise inherit the importing test's Mocha file label.
const fs = require('node:fs');
const path = require('node:path');
const Suite = require('../source_repo/node_modules/mocha/lib/suite.js');
const addTest = Suite.prototype.addTest;
const tests = [];

Suite.prototype.addTest = function recordRegistration(test) {
  const holder = {};
  Error.captureStackTrace(holder, recordRegistration);
  const frames = holder.stack.split('\n').slice(1);
  const match = frames.map(frame => frame.match(/(?:at |\()([^()]+\.test\.(?:js|ts)):\d+:\d+\)?$/))
    .find(Boolean);
  tests.push({ test, registrationFile: match ? match[1].trim() : null });
  return addTest.call(this, test);
};

process.on('exit', () => {
  const records = tests.map(({ test, registrationFile }) => ({
    title: test.fullTitle(), reportedFile: test.file, registrationFile,
    pending: test.isPending(), state: test.state ?? null
  }));
  const file = process.env.XTERM_BASELINE_REGISTRATION_FILE;
  if (!file || !path.isAbsolute(file)) throw new Error('An absolute per-attempt registration report path is required');
  fs.writeFileSync(file, JSON.stringify(records, null, 2) + '\n');
});
