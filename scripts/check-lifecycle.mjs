// Copyright (c) 2024-2026 The xterm.js authors. All rights reserved.
// Licensed under the MIT license.
// Node 24 strips TypeScript types; the reference source is not rewritten.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { toDisposable, DisposableStore, MutableDisposable, combinedDisposable }
  from '../source_repo/src/common/Lifecycle.ts';

const log = [];
const record = name => toDisposable(() => log.push(name));
const store = new DisposableStore();
const a = record('a');
store.add(a);
store.add(a);
store.add(record('b'));
store.clear();
store.add(a);
store.dispose();
store.dispose();
store.add(record('late'));
const mutable = new MutableDisposable();
mutable.value = a;
mutable.value = a;
mutable.value = record('replacement');
mutable.clear();
mutable.value = record('final');
mutable.dispose();
mutable.value = record('ignored');
const group = combinedDisposable(a, record('group'), a);
group.dispose();
group.dispose();
// Retain the nested-clear counterexample found during independent review.
const nestedStore = new DisposableStore();
let entered = false;
const late = record('nested-b');
nestedStore.add(toDisposable(() => {
  log.push('nested-a');
  if (!entered) {
    entered = true;
    nestedStore.clear();
    nestedStore.add(late);
  }
}));
nestedStore.clear();
nestedStore.clear();
const expected = readFileSync(new URL('../crates/xterm-core/tests/fixtures/lifecycle-trace.txt', import.meta.url), 'utf8');
assert.equal(log.join('\n') + '\n', expected);
console.log(JSON.stringify({
  oracle: fileURLToPath(new URL('../source_repo/src/common/Lifecycle.ts', import.meta.url)),
  fixture: 'crates/xterm-core/tests/fixtures/lifecycle-trace.txt',
  callbacks: log.length,
  status: 'passed',
  node: process.version
}));
