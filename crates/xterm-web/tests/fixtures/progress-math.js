// Shared source/Rust public-setter cases. No terminal engine is implemented here.
async function progressMathCases(create) {
  const rows = [];
  const originalMath = globalThis.Math;
  function assert(ok, message) { if (!ok) throw Error(message); }
  async function check(name, body) {
    const { addon, close } = create();
    try { await body(addon); rows.push({ name, passed: true }); }
    finally { globalThis.Math = originalMath; close(); }
  }
  for (const [name, value, expected] of [
    ['negative', -3, 0], ['fractional', 12.5, 12.5], ['overflow', Infinity, 100],
    ['negative-zero', -0, 0], ['nan', NaN, NaN], ['numeric-string', '23', 23]
  ]) await check(name, addon => {
    addon.progress = { state: 1, value };
    assert(Object.is(addon.progress.value, expected), name + ' numeric clamp');
  });
  await check('lookup-call-order-receivers', addon => {
    const calls = [], math = {};
    const maxResult = {};
    Object.defineProperties(math, {
      min: { get() { calls.push('get:min'); return function(a, b) {
        calls.push('call:min'); assert(this === math && a === maxResult && b === 100, 'min arguments/receiver'); return 91;
      }; } },
      max: { get() { calls.push('get:max'); return function(a, b) {
        calls.push('call:max'); assert(this === math && a === 17 && b === 0, 'max arguments/receiver'); return maxResult;
      }; } }
    });
    globalThis.Math = math;
    addon.progress = { get state() { calls.push('state'); return 1; }, get value() { calls.push('value'); return 17; } };
    assert(calls.join(',') === 'state,state,state,get:min,get:max,value,call:max,call:min', 'observable evaluation order ' + calls);
    assert(addon.progress.value === 91, 'host min result');
  });
  await check('math-global-replacement-preserves-captured-min', addon => {
    const calls = [], second = { max(a, b) {
      calls.push('second:max'); assert(this === second && a === 7 && b === 0, 'second max receiver'); return 13;
    } };
    const first = { get min() {
      calls.push('first:min'); globalThis.Math = second;
      return function(a, b) {
        calls.push('first:call'); assert(this === first && a === 13 && b === 100, 'captured min receiver'); return 21;
      };
    } };
    globalThis.Math = first;
    addon.progress = { state: 1, value: 7 };
    assert(calls.join(',') === 'first:min,second:max,first:call' && addon.progress.value === 21, 'global replacement order');
  });
  for (const phase of ['get:min', 'get:max', 'value', 'call:max', 'call:min']) {
    await check('throw-' + phase, addon => {
      addon.progress = { state: 1, value: 40 };
      const sentinel = { phase }, calls = []; let events = 0;
      addon.onChange(() => events++);
      const step = name => { calls.push(name); if (name === phase) throw sentinel; };
      globalThis.Math = {
        get min() { step('get:min'); return function() { step('call:min'); return 88; }; },
        get max() { step('get:max'); return function() { step('call:max'); return 22; }; }
      };
      let caught;
      try { addon.progress = { state: 2, get value() { step('value'); return 8; } }; }
      catch (error) { caught = error; }
      assert(caught === sentinel, phase + ' error identity');
      assert(addon.progress.state === 2 && addon.progress.value === 40 && events === 0, phase + ' partial assignment');
      const phases = ['get:min', 'get:max', 'value', 'call:max', 'call:min'];
      assert(calls.join(',') === phases.slice(0, phases.indexOf(phase) + 1).join(','), phase + ' short circuit');
    });
  }
  for (const [name, value] of [['object', {}], ['symbol', Symbol('host')], ['undefined', undefined],
    ['bigint', 27n], ['negative-number', -17], ['large-number', 200], ['negative-zero', -0]]) {
    await check('raw-return-' + name, addon => {
      let event;
      addon.onChange(payload => { event = payload; });
      globalThis.Math = { max() { return 0; }, min() { return value; } };
      addon.progress = { state: 2, value: 10 };
      assert(Object.is(addon.progress.value, value), name + ' getter raw identity');
      assert(event.state === 2 && Object.is(event.value, value), name + ' event raw identity');
      event.value = 'mutated';
      assert(Object.is(addon.progress.value, value), name + ' payload mutation changed stored value');
    });
  }
  for (const phase of ['get:min', 'get:max', 'value', 'call:max', 'call:min']) {
    await check('reentry-' + phase, addon => {
      const events = []; let inner = false;
      addon.onChange(payload => events.push([payload.state, payload.value]));
      const step = name => {
        if (name === phase && !inner) {
          inner = true;
          globalThis.Math = originalMath;
          try { addon.progress = { state: 4, value: 11 }; }
          finally { globalThis.Math = math; }
        }
      };
      const math = {
        get min() { step('get:min'); return function(a, b) { step('call:min'); return originalMath.min(a, b); }; },
        get max() { step('get:max'); return function(a, b) { step('call:max'); return originalMath.max(a, b); }; }
      };
      globalThis.Math = math;
      addon.progress = { state: 1, get value() { step('value'); return 25; } };
      assert(inner && addon.progress.state === 4 && addon.progress.value === 25, phase + ' final reentrant state');
      assert(JSON.stringify(events) === '[[4,11],[4,25]]', phase + ' reentrant events ' + JSON.stringify(events));
    });
  }
  for (const key of ['min', 'max']) await check('noncallable-' + key, addon => {
    const calls = []; let caught;
    globalThis.Math = {
      min: key === 'min' ? null : function() { calls.push('min'); },
      max: key === 'max' ? null : function() { calls.push('max'); }
    };
    try { addon.progress = { state: 3, get value() { calls.push('value'); return 1; } }; }
    catch (error) { caught = error; }
    assert(caught instanceof TypeError, key + ' must throw TypeError');
    assert(calls.join(',') === (key === 'min' ? 'value,max' : 'value'), key + ' noncallable phase');
    assert(addon.progress.state === 3 && addon.progress.value === 0, key + ' partial assignment');
  });
  await check('throw-after-raw-return-keeps-raw-previous', addon => {
    const raw = {}, sentinel = {};
    globalThis.Math = { min() { return raw; }, max() { return 0; } };
    addon.progress = { state: 1, value: 1 };
    globalThis.Math = { get min() { throw sentinel; } };
    let caught;
    try { addon.progress = { state: 3, value: 1 }; } catch (error) { caught = error; }
    assert(caught === sentinel && addon.progress.state === 3 && addon.progress.value === raw, 'raw previous lost on failure');
  });
  await check('dispose-during-math-call', addon => {
    let events = 0;
    addon.onChange(() => events++);
    globalThis.Math = { min() { addon.dispose(); return 77; }, max() { return 0; } };
    addon.progress = { state: 2, value: 9 };
    assert(events === 0 && addon.progress.state === 2 && addon.progress.value === 77, 'dispose call phases');
  });
  await check('invalid-state-skips-math', addon => {
    let touched = false;
    globalThis.Math = { get min() { touched = true; throw Error('invalid state reads Math'); } };
    addon.progress = { state: -1, value: 0 };
    assert(!touched && addon.progress.state === 0, 'invalid state touched Math');
  });
  return rows;
}
