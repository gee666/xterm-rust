// Directed source checks paired with Rust edge tests. No upstream files changed.
import assert from 'node:assert/strict';

export function directedChecks({ Emitter, EventUtils, CircularList, DisposableStore }) {
  const cases = [];
  function check(name, callback) {
    try { callback(); cases.push({ name, status: 'passed' }); }
    catch (error) { cases.push({ name, status: 'failed', error: String(error) }); }
  }
  function list(values, capacity = values.length) {
    const list = new CircularList(capacity);
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
  check('E1 late direct registration returns noop without collecting', () => {
    const emitter = new Emitter(); emitter.dispose();
    const handles = []; const result = emitter.event(() => assert.fail('late'), undefined, handles);
    assert.equal(handles.length, 0); result.dispose(); result.dispose();
    emitter.event(null, undefined, Object.freeze([]));
  });
  check('E1 late mapped registration neither collects nor maps', () => {
    const emitter = new Emitter(); let maps = 0;
    const mapped = EventUtils.map(emitter.event, value => { maps++; return value; });
    emitter.dispose(); const handles = [];
    mapped(() => assert.fail('mapped late'), undefined, handles); emitter.fire(1);
    assert.equal(handles.length, 0); assert.equal(maps, 0);
  });
  check('E1 late store registration differs from explicit user collection', () => {
    const emitter = new Emitter(); emitter.dispose(); const store = new DisposableStore();
    const result = emitter.event(() => {}, undefined, store);
    assert.equal(store._disposables.size, 0);
    store.add(result); assert.equal(store._disposables.size, 1); store.clear();
    assert.equal(store._disposables.size, 0);
  });
  check('E3 any over disposed sources collects one aggregate', () => {
    const first = new Emitter(); const second = new Emitter(); first.dispose(); second.dispose();
    const handles = []; const result = EventUtils.any(first.event, second.event)(() => {}, undefined, handles);
    assert.equal(handles.length, 1); assert.equal(handles[0], result); result.dispose(); result.dispose();
  });
  check('E3 empty any and mapped any collect their own aggregates', () => {
    const empty = EventUtils.any(); const handles = [];
    const first = empty(() => {}, undefined, handles);
    const second = EventUtils.map(empty, value => value)(() => {}, undefined, handles);
    assert.equal(handles.length, 2); assert.notEqual(first, second); first.dispose(); second.dispose();
  });
  check('E3 any over mapped disposed events still collects outer aggregate', () => {
    const emitter = new Emitter(); emitter.dispose();
    const any = EventUtils.any(EventUtils.map(emitter.event, value => value)); const handles = [];
    any(() => {}, undefined, handles);
    EventUtils.map(any, value => value)(() => {}, undefined, handles);
    assert.equal(handles.length, 2); handles.forEach(handle => handle.dispose());
  });
  check('L3 sparse splice past logical end does not grow to start', () => {
    const value = list([1,2,3], 12); value.splice(7,0,9);
    assert.equal(value.length, 4); assert.equal(value.get(3), undefined); assert.equal(value.get(7), 9);
  });
  check('L3 deletion count is not clamped to remaining tail', () => {
    const value = list([1,2,3,4,5], 8); value.splice(4,3);
    assert.equal(value.length, 2); assert.equal(value.get(1), 2); assert.equal(value.get(4), 5);
  });
  check('L3 far deletion and insertion preserve callback-time lengths', () => {
    const value = list([1,2,3], 8); const events = [];
    value.onDelete(change => events.push(['delete',change.index,change.amount,value.length]));
    value.onInsert(change => events.push(['insert',change.index,change.amount,value.length]));
    value.splice(7,1,9);
    assert.equal(value.length, 3); assert.equal(value.get(2), 3); assert.equal(value.get(7), 9);
    assert.deepEqual(events, [['delete',7,1,2],['insert',7,1,2]]);
  });
  check('L5 reentrant backing reset leaves cached insertion sparse', () => {
    const value = list([1,2,3,4,5,6,7,8,9,10], 12);
    value.onDelete(() => { value.length = 3; value.maxLength = 13; });
    value.splice(7,1,99);
    assert.equal(value.length, 4); assert.equal(value.get(3), undefined); assert.equal(value.get(7), 99);
  });
  check('L1 fractional and subnormal properties survive ring advances', () => {
    const value = new CircularList(5); const tiny = Number.MIN_VALUE;
    value.set(0.5,11); value.set(-0.5,12); value.set(tiny,13);
    assert.equal(value.get(0.5),11); assert.equal(value.get(-0.5),12); assert.equal(value.get(tiny),13);
    for (let item=1;item<=6;item++) value.push(item);
    assert.equal(value.get(-0.5),11); assert.equal(value.get(0.5),undefined); assert.equal(value.get(tiny),2);
    for (let item=7;item<=10;item++) value.push(item); assert.equal(value.get(tiny),13);
  });
  check('L1 NaN and either infinity share the same property', () => {
    const value = new CircularList(5); value.set(NaN,77);
    for (const index of [NaN,Infinity,-Infinity]) assert.equal(value.get(index),77);
    value.set(-Infinity,88); assert.equal(value.get(NaN),88);
    value.set(NaN,undefined); assert.equal(value.get(Infinity),undefined);
    value.set(-0,9); assert.equal(value.get(0),9);
  });
  check('L2 same capacity and length preserve named slots; rebuild discards', () => {
    const value = new CircularList(5); value.set(0.5,11); value.set(NaN,77); value.set(-1,99);
    value.length=1; value.set(0,3); value.maxLength=5;
    assert.equal(value.get(0.5),11); assert.equal(value.get(Infinity),77);
    value.maxLength=6; assert.equal(value.get(0),3);
    for (const index of [0.5,NaN,-1]) assert.equal(value.get(index),undefined);
  });
  return cases;
}
