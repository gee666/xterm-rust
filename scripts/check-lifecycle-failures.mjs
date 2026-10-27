// Compare callback throws with the unchanged lifecycle reference, not a JS model.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { toDisposable, dispose, combinedDisposable, DisposableStore, MutableDisposable }
  from '../source_repo/src/common/Lifecycle.ts';

const log = [];
const record = name => toDisposable(() => log.push(name));
const failOnce = name => {
  let fail = true;
  return toDisposable(() => {
    log.push(name);
    if (fail) {
      fail = false;
      throw new Error('injected lifecycle failure');
    }
  });
};
let thrown = 0;
const expectFailure = callback => {
  assert.throws(callback, /injected lifecycle failure/);
  thrown++;
};

const failure = failOnce('helper-fail');
const tail = record('helper-tail');
expectFailure(() => dispose([failure, tail]));
assert.deepEqual(log, ['helper-fail']);
dispose([failure, tail]);

const clear = new DisposableStore();
clear.add(record('clear-before'));
clear.add(failOnce('clear-fail'));
clear.add(record('clear-after'));
expectFailure(() => clear.clear());
assert.equal(clear.isDisposed, false);
clear.clear();
clear.clear();

const closed = new DisposableStore();
closed.add(failOnce('dispose-fail'));
closed.add(record('dispose-skipped'));
expectFailure(() => closed.dispose());
assert.equal(closed.isDisposed, true);
closed.dispose();
closed.add(record('dispose-late'));

const replace = new MutableDisposable();
const old = failOnce('replace-old');
const replacement = record('replace-new');
replace.value = old;
expectFailure(() => { replace.value = replacement; });
assert.equal(replace.value, old);
replace.value = replacement;
replace.clear();

const mutable = new MutableDisposable();
mutable.value = failOnce('mutable-fail');
expectFailure(() => mutable.dispose());
assert.equal(mutable.value, undefined);
mutable.value = record('mutable-ignored');
mutable.dispose();
mutable.dispose();

const combined = combinedDisposable(failOnce('combined-fail'), record('combined-tail'));
expectFailure(() => combined.dispose());
combined.dispose();
const fixture = new URL('../crates/xterm-core/tests/fixtures/lifecycle-failures.txt', import.meta.url);
assert.equal(log.join('\n') + '\n', readFileSync(fixture, 'utf8'));
console.log(JSON.stringify({ scope: 'Lifecycle callback failures only', callbacks: log.length,
  injectedThrows: thrown, status: 'passed', node: process.version }));
