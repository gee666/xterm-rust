// Directed source checks paired with Rust edge tests. No upstream files changed.
import assert from 'node:assert/strict';

export function directedChecks({ Emitter, EventUtils, CircularList }) {
  const cases = [];
  function check(name, callback) {
    try { callback(); cases.push({ name, status: 'passed' }); }
    catch (error) { cases.push({ name, status: 'failed', error: String(error) }); }
  }
  function list(values) {
    const list = new CircularList(values.length);
    for (const value of values) list.push(value);
    return list;
  }
  check('E1 cached registration identity and duplicate handles', () => {
    const emitter = new Emitter();
    assert.equal(emitter.event, emitter.event);
    let calls = 0;
    const callback = () => calls++;
    const first = emitter.event(callback);
    emitter.event(callback);
    emitter.fire(1); assert.equal(calls, 2);
    first.dispose(); first.dispose(); emitter.fire(1); assert.equal(calls, 3);
  });
  check('E2 callback exceptions retain original identity through map/any/forward', () => {
    const source = new Emitter();
    const target = new Emitter();
    const original = { identity: 7 };
    target.event(() => { throw original; });
    target.event(() => assert.fail('listener after failure'));
    EventUtils.forward(EventUtils.map(EventUtils.any(source.event), value => value * 2), target);
    let caught;
    try { source.fire(1); } catch (error) { caught = error; }
    assert.equal(caught, original);
  });
  check('E3 any ignores dropped handle but combined explicit disposal is idempotent', () => {
    const source = new Emitter();
    let count = 0;
    EventUtils.any(source.event)(() => count++);
    const combined = EventUtils.any(source.event, source.event)(() => count++);
    source.fire(1); assert.equal(count, 3);
    combined.dispose(); combined.dispose(); source.fire(1); assert.equal(count, 4);
  });
  check('E3 initial failure leaves no registration', () => {
    const source = new Emitter(); let count = 0;
    const error = new Error('initial');
    assert.throws(() => EventUtils.runAndSubscribe(source.event, () => { count++; throw error; }), value => value === error);
    source.fire(1); assert.equal(count, 1);
  });
  check('L1 source maximum capacity is sparse and high-index writes preserve holes', () => {
    const value = new CircularList(0xffffffff);
    assert.equal(value.length, 0);
    value.push(5); value.set(0xfffffffe, 7);
    assert.equal(value.get(0), 5); assert.equal(value.get(1), undefined);
    assert.equal(value.get(0xfffffffe), 7);
    value.maxLength = 2;
    assert.equal(value.length, 1); assert.equal(value.get(1), undefined);
  });
  check('L2 growth after shrink repeats cyclic values and retains holes', () => {
    const value = list([1,2,3,4]); value.maxLength = 2; value.maxLength = 5;
    assert.deepEqual(Array.from({ length: 5 }, (_, index) => value.get(index)), [1,2,1,2,undefined]);
    value.set(1, undefined); value.maxLength = 2; value.maxLength = 5;
    assert.deepEqual(Array.from({ length: 5 }, (_, index) => value.get(index)), [1,undefined,1,undefined,undefined]);
  });
  check('L5 recycle reads after reentrant length mutation', () => {
    const value = list([1,2]); value.onTrim(() => value.length = 0);
    assert.equal(value.recycle(), 1); assert.equal(value.length, 0);
  });
  check('L5 push permits reentrant capacity mutation', () => {
    const value = list([1,2]); value.onTrim(() => value.maxLength = 1); value.push(3);
    assert.equal(value.length, 2); assert.equal(value.maxLength, 1);
    assert.equal(value.get(0), 2); assert.equal(value.get(1), 2);
  });
  check('L5 insert observer trim changes outer adjustment', () => {
    const value = list([1,2,3]); value.length = 2;
    value.onInsert(() => value.trimStart(1)); value.splice(1,0,3);
    assert.equal(value.length, 2); assert.equal(value.get(0), 3); assert.equal(value.get(1), 2);
  });
  check('L5 delete observer length reset precedes insert', () => {
    const value = list([1,2,3]); value.onDelete(() => value.length = 0); value.splice(1,1,4);
    assert.equal(value.length, 1); assert.equal(value.get(0), 1); assert.equal(value.get(1), 4);
  });
  check('L5 overflow observer can shorten before next trim', () => {
    const value = list([1,2,3]); let calls = 0;
    value.onTrim(amount => { assert.equal(amount, 1); calls++; value.length = 2; });
    value.shiftElements(0,1,5);
    assert.equal(calls, 1); assert.equal(value.length, 2);
    assert.equal(value.get(0), 2); assert.equal(value.get(1), 1);
  });
  check('L5 disposed arrays retain exact partial delete/insert/trim state', () => {
    for (const family of ['delete','insert','trim']) {
      const value = list([1,2,3]);
      const emitter = family === 'delete' ? value.onDeleteEmitter : family === 'insert' ? value.onInsertEmitter : value.onTrimEmitter;
      emitter.event(() => emitter.dispose()); emitter.event(() => assert.fail('listener after disposed array'));
      assert.throws(() => family === 'trim' ? value.push(4) : value.splice(1,1,4), error => error.name === 'TypeError');
      assert.equal(value.length, family === 'trim' ? 3 : 2);
      assert.equal(value.get(0), family === 'trim' ? 2 : 1);
      assert.equal(value.get(1), family === 'delete' ? 3 : family === 'insert' ? 4 : 3);
    }
  });
  return cases;
}
