// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Recompute receipt attribution independently instead of trusting suite totals.
import assert from 'node:assert/strict';
const groups = { tests: 'discovered', passes: 'passed', failures: 'failed', pending: 'pending' };
function sourceForOutput(file) {
  assert(typeof file === 'string' && file.length, 'Missing baseline registration file');
  const portable = file.replaceAll('\\', '/');
  const marker = '/source_repo/';
  const index = portable.lastIndexOf(marker);
  const relative = index >= 0 ? portable.slice(index + marker.length) : portable.replace(/^source_repo\//, '');
  assert(!relative.split('/').includes('..'), 'Invalid baseline generated path');
  const prefix = 'out-esbuild/';
  if (relative.startsWith(prefix)) return `src/${relative.slice(prefix.length).replace(/\.js$/, '.ts')}`;
  const addon = /^(addons\/[^/]+)\/out-esbuild\/(.+)\.js$/.exec(relative);
  if (addon) return `${addon[1]}/src/${addon[2]}.ts`;
  assert(/^(src|addons\/[^/]+\/src)\/.*\.test\.ts$/.test(relative), `Unexpected baseline generated path: ${relative}`);
  return relative;
}
function key(file, title) {
  assert(typeof title === 'string', 'Invalid baseline case title');
  return `${sourceForOutput(file)}\0${title}`;
}
const increment = (map, name) => map.set(name, (map.get(name) ?? 0) + 1);
function multiset(rows) {
  const counts = new Map();
  for (const row of rows) increment(counts, key(row.file, row.fullTitle));
  return [...counts].sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0);
}

export function validateCaseCounts(summary, raw, registrations) {
  assert(Array.isArray(registrations), 'Baseline registrations must be an array');
  for (const [group, field] of Object.entries(groups)) {
    assert(Array.isArray(raw[group]), `Missing raw baseline cases: ${group}`);
    assert.equal(raw[group].length, summary.totals[field], `Baseline raw array count mismatch: ${group}`);
  }
  assert.equal(registrations.length, raw.tests.length, 'Baseline registration count mismatch');
  const sourceByCase = new Map();
  for (const entry of registrations) {
    assert(typeof entry.title === 'string', 'Invalid registration title');
    const name = key(entry.reportedFile, entry.title);
    const source = sourceForOutput(entry.registrationFile);
    assert(!sourceByCase.has(name) || sourceByCase.get(name) === source, `Ambiguous baseline registration: ${entry.title}`);
    sourceByCase.set(name, source);
  }
  const discovered = multiset(raw.tests);
  assert.deepEqual(discovered, multiset([...raw.passes, ...raw.failures, ...raw.pending]), 'Baseline raw case outcomes differ from discovery');
  const registered = registrations.map(row => ({ file: row.reportedFile, fullTitle: row.title }));
  assert.deepEqual(discovered, multiset(registered), 'Baseline registration cases differ from discovery');
  const counts = new Map(summary.suites.map(s => [s.source, { discovered: 0, passed: 0, failed: 0, pending: 0 }]));
  for (const [group, field] of Object.entries(groups)) for (const test of raw[group]) {
    const source = sourceByCase.get(key(test.file, test.fullTitle));
    assert(source && counts.has(source), `Baseline case has no source attribution: ${test.fullTitle}`);
    counts.get(source)[field]++;
  }
  for (const suite of summary.suites) for (const field of Object.values(groups)) {
    assert.equal(suite[field], counts.get(suite.source)[field], `Baseline case attribution mismatch: ${suite.source}.${field}`);
  }
}
