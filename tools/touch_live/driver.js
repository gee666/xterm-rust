// Workloads and observations only. Gesture arithmetic runs in source Gesture or Rust.
const TOUCH_LIVE_CASES = [
  'start-fields', 'move-fields', 'move-key-race', 'end-key-delete', 'item-list-reload',
  'move-create-reentry', 'commit-x-reentry', 'partial-y-history', 'throw-identifier',
  'throw-commit-x', 'init-throw', 'tap-clock-distinct', 'type-live', 'initial-target-live',
  'translation-set-reentry', 'clock-throw', 'end-key-missing', 'numeric-ids', 'initial-depth-reverse',
  'throw-event-type', 'throw-initial-target', 'throw-translation-set'
];
function runTouchScenario(host, name) {
  const log = host.log;
  const labels = host.labels;
  const enc = value => {
    if (typeof value === 'number') return Number.isNaN(value) ? 'NaN' : Object.is(value, -0) ? '-0' : String(value);
    if (typeof value === 'string') return JSON.stringify(value);
    if (value === null) return 'null';
    if (value === undefined) return 'undefined';
    if (typeof value === 'boolean') return String(value);
    return labels.has(value) ? '@' + labels.get(value) : typeof value;
  };
  host.encode = enc;
  let serial = 0, eventSerial = 0;
  let onCreate = null, onSet = null, onGet = null;
  host.installCreate(event => {
    const meta = { id: ++eventSerial, kind: '', values: {}, assigned: new Set(), reads: {} };
    host.meta.set(event, meta);
    const init = event.initEvent;
    Object.defineProperty(event, 'initEvent', { configurable: true, get() {
      log.push('get:E' + meta.id + '.initEvent');
      return function(kind, bubbles, cancelable) {
        init.call(event, kind, bubbles, cancelable); meta.kind = kind;
        log.push('init:E' + meta.id + ':' + kind);
        if (host.initHook) host.initHook(meta);
      };
    }});
    for (const field of ['type','initialTarget','tapCount','pageX','pageY','clientX','clientY','translationX','translationY']) {
      Object.defineProperty(event, field, { configurable: true, get() {
        const count = meta.reads[field] = (meta.reads[field] || 0) + 1;
        const replacement = onGet ? onGet(meta, field, count) : null;
        const value = replacement ? replacement.value : field === 'type' ? meta.kind : meta.values[field];
        log.push('get:E' + meta.id + '.' + field + '#' + count + ':' + enc(value));
        return value;
      }, set(value) {
        meta.values[field] = value; meta.assigned.add(field);
        log.push('set:E' + meta.id + '.' + field + ':' + enc(value));
        if (onSet) onSet(meta, field, value);
      }});
    }
    log.push('create:E' + meta.id);
    if (onCreate) onCreate(meta);
    return event;
  });
  for (const [node, label] of [[host.root,'root'], [host.child,'child']]) {
    for (const kind of ['-xterm-gesturestart','-xterm-gesturechange','-xterm-gesturetap','-xterm-gesturesend','-xterm-gesturecontextmenu']) {
      host.listen(node, kind, event => {
        const meta = host.meta.get(event);
        const fields = [...meta.assigned].map(field => field + '=' + enc(meta.values[field])).join(';');
        log.push('dispatch:' + label + ':E' + meta.id + ':' + meta.kind + ':' + fields);
      });
    }
  }
  function token(label) { const value = {}; labels.set(value, 'error:' + label); return value; }
  function touch(label, overrides = {}, hooks = {}) {
    const data = { identifier: 1, target: host.leaf, pageX: 0, pageY: 0, clientX: 1, clientY: 2, ...overrides };
    const result = {}; labels.set(result, 'touch:' + label);
    for (const key of Object.keys(data)) {
      let reads = 0;
      Object.defineProperty(result, key, { get() {
        reads++;
        const sequence = data[key];
        const value = Array.isArray(sequence) ? sequence[Math.min(reads - 1, sequence.length - 1)] : sequence;
        log.push('touch:' + label + '.' + key + '#' + reads + ':' + enc(value));
        if (hooks[key]) hooks[key](reads);
        return value;
      }});
    }
    return result;
  }
  function list(label, touches) {
    const value = {};
    Object.defineProperty(value, 'length', { get() { log.push('list:' + label + '.length'); return touches.length; } });
    Object.defineProperty(value, 'item', { get() {
      log.push('list:' + label + '.item');
      return function(index) { log.push('item:' + label + ':' + index + ':this=' + (this === value)); return touches[index]; };
    }});
    return value;
  }
  function send(kind, touches, lists) {
    const event = host.input(kind);
    const id = ++serial;
    const standard = list('L' + id, touches);
    for (const key of ['targetTouches','changedTouches']) {
      let reads = 0;
      Object.defineProperty(event, key, { get() {
        reads++; log.push('input:I' + id + '.' + key + '#' + reads);
        return lists ? lists[Math.min(reads - 1, lists.length - 1)] : standard;
      }});
    }
    host.dispatch(event);
    log.push('input:I' + id + ':prevented=' + event.defaultPrevented);
  }
  const start = (id = 1, x = 0, y = 0) => send('touchstart', [touch('s' + serial, { identifier:id, pageX:x, pageY:y })]);
  const move = (x, y = 0, time = host.time) => {
    host.time = time;
    send('touchmove', [touch('m' + serial, { pageX:x, pageY:y, clientX:x+1, clientY:y+2 })]);
  };
  const end = (id = 1, time = host.time) => {
    host.time = time; send('touchend', [touch('e' + serial, { identifier:id })]);
  };
  const flush = time => { host.time = time; log.push('flush:' + enc(time)); host.flush(); log.push('pending:' + host.pending()); };
  const warm = () => { start(); move(10, 0, 1010); move(50, 0, 1020); move(90, 0, 1030); };
  host.recording = true;
  if (name === 'start-fields') {
    send('touchstart', [touch('start', { identifier:[1,2], target:[host.leaf,host.child], pageX:[0,10,20], pageY:[0,11,21] })]);
    move(30,40,1010); end();
  } else if (name === 'move-fields') {
    start();
    send('touchmove', [touch('move', { identifier:[1,1], pageX:[10,20,30], pageY:[11,21,31], clientX:32, clientY:33 })]);
    move(35,36,1010); end();
  } else if (name === 'move-key-race') {
    start(1); start(2,100,100);
    send('touchmove', [touch('key', { identifier:[1,2], pageX:110, pageY:120 })]);
    move(1,2,1010); end(2); end(1);
  } else if (name === 'end-key-delete') {
    start(1); start(2,100,100);
    send('touchend', [touch('delete', { identifier:[1,1,2] })]);
    send('touchmove', [touch('missing', { identifier:2, pageX:110, pageY:120 })]);
    move(3,4,1010);
  } else if (name === 'item-list-reload') {
    start(1); start(2,100,100);
    const a = list('A', [touch('a', { identifier:1, pageX:10 })]);
    const b = list('B', [touch('b', { identifier:2, pageX:120 })]);
    send('touchmove', [], [a,b]);
    end(2);
  } else if (name === 'move-create-reentry') {
    start(); let entered = false;
    onCreate = () => { if (!entered) { entered = true; move(20,10,1010); } };
    move(30,40,1020); onCreate = null; end();
  } else if (name === 'commit-x-reentry') {
    start(); move(10,0,1010); move(20,0,1020);
    send('touchmove', [touch('outer', { pageX:100, pageY:0 }, { pageX(count) {
      if (count === 3) move(30,0,1030);
    }})]);
    end(1,1040); flush(1050); flush(20000);
  } else if (name === 'partial-y-history') {
    warm();
    function nested(depth) {
      send('touchmove', [touch('depth' + depth, { pageX:100+depth, pageY:200+depth }, { pageY(count) {
        if (count === 3 && depth < 4) nested(depth+1);
      }})]);
    }
    nested(0); end(1,1100); flush(1110);
  } else if (name === 'throw-identifier') {
    start(); const error = token('identifier');
    send('touchmove', [touch('throw-id', {}, { identifier(count) { if (count === 2) throw error; } })]);
    move(7,8,1010);
  } else if (name === 'throw-commit-x') {
    warm(); const error = token('commit');
    send('touchmove', [touch('throw-x', { pageX:100 }, { pageX(count) { if (count === 3) throw error; } })]);
    end(1,1040); flush(1050);
  } else if (name === 'init-throw') {
    start(); const error = token('init'); let entered = false;
    host.initHook = () => { if (!entered) { entered = true; throw error; } };
    move(30,40,1010); host.initHook = null; move(1,2,1020);
  } else if (name === 'tap-clock-distinct') {
    host.tapTime = 7000; start(); end();
    host.time = 1500; host.tapTime = 7100; start(); end();
  } else if (name === 'type-live') {
    start(); onGet = (meta, key, count) => key === 'type' && meta.kind === '-xterm-gesturechange'
      ? { value:count === 1 ? 'other' : '-xterm-gesturechange' } : null;
    move(1,2,1010); onGet = null; end();
  } else if (name === 'initial-target-live') {
    onGet = (meta, key, count) => key === 'initialTarget'
      ? { value:count === 2 ? host.outside : host.leaf } : null;
    start(); onGet = null; end();
  } else if (name === 'translation-set-reentry') {
    start(); let entered = false;
    onSet = (meta,key) => { if (key === 'translationX' && !entered) { entered = true; move(20,25,1010); } };
    move(30,40,1020); onSet = null; end();
  } else if (name === 'clock-throw') {
    start(); const error = token('clock'); let entered = false;
    host.nowHook = () => { if (!entered) { entered = true; throw error; } };
    move(30,40,1010); host.nowHook = null; move(1,2,1020);
  } else if (name === 'end-key-missing') {
    start(); send('touchend', [touch('missing-end', { identifier:[1,99] })]);
    move(1,2,1010);
  } else if (name === 'throw-event-type' || name === 'throw-initial-target') {
    start(); const error=token(name);
    onGet=(meta,key,count)=>{if(meta.kind==='-xterm-gesturechange' && key===(name==='throw-event-type'?'type':'initialTarget') && count===2)throw error;return null;};
    move(30,40,1010); onGet=null; move(1,2,1020);
  } else if (name === 'throw-translation-set') {
    start(); const error=token('translation-set');let entered=false;
    onSet=(meta,key)=>{if(key==='translationX'&&!entered){entered=true;throw error;}};
    move(30,40,1010);onSet=null;move(1,2,1020);
  } else if (name === 'initial-depth-reverse') {
    onGet = (meta,key,count) => key === 'initialTarget' ? { value:count === 3 ? host.outside : host.leaf } : null;
    start(); onGet = null; end();
  } else if (name === 'numeric-ids') {
    for (const id of [-0,NaN,Infinity,-Infinity,4294967297,1.5]) {
      start(id); send('touchmove', [touch('numeric', { identifier:id, pageX:10, pageY:11 })]); end(id);
    }
  } else if (name.startsWith('seed:')) {
    let state = Number(name.slice(5)) >>> 0;
    const random = () => { state ^= state<<13; state ^= state>>>17; state ^= state<<5; return state>>>0; };
    start();
    for (let index=0;index<4;index++) {
      const x=(random()%800-400)/4, y=(random()%800-400)/4;
      const nestedX=(random()%800-400)/4, nestedY=(random()%800-400)/4;
      const trigger=1+random()%3;
      send('touchmove', [touch('r'+index, { pageX:x, pageY:y }, { pageX(count) {
        if (count===trigger) move(nestedX,nestedY,1010+index);
      }})]);
    }
    end(1,1100); flush(1116); flush(1132);
  } else { throw new Error('Unknown touch live scenario ' + name); }
  const result = log.slice(); host.recording = false;
  return result;
}
