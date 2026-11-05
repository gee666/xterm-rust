// Deterministic host shared by unchanged source and Rust. No terminal decisions
// are implemented here. Timer callbacks are driven explicitly, without sleeps.
export function runCases(factory) {
  const results = [];
  const realClock = Date.now;
  const cases = [
    ['unfocused', false, e => { e.op('restart'); e.op('pause'); e.op('dispose'); }],
    ['initial-start-interval', true, e => { e.fire('start'); e.fire('frame'); e.fire('interval'); e.fire('frame'); }],
    ['restart-defers-positive', true, e => { e.now = 1100; e.op('restart'); e.fire('frame'); e.now = 1200; e.fire('start'); e.now = 1700; e.fire('start', 1); e.fire('frame', 1); }],
    ['restart-expired-start', true, e => { e.op('restart'); e.now = 1800; e.fire('start'); }],
    ['restart-zero-clock', true, e => { e.now = 0; e.op('restart'); e.fire('start'); }],
    ['interval-negative-restart', true, e => { e.fire('start'); e.op('restart'); e.now = 1900; e.fire('interval'); }],
    ['interval-zero-restart', true, e => { e.fire('start'); e.op('restart'); e.now = 1600; e.fire('interval'); }],
    ['coalesced-frame', true, e => { e.op('restart'); e.op('restart'); e.fire('frame'); e.op('restart'); }],
    ['pause-resume', true, e => { e.fire('start'); e.op('pause'); e.op('resume'); e.fire('frame', 1); e.op('dispose'); e.op('dispose'); }],
    ['idle-does-not-auto-resume', true, e => { e.fire('idle'); e.op('restart'); e.op('restart'); }],
    ['idle-then-resume', true, e => { e.fire('start'); e.fire('idle'); e.op('resume'); }],
    ['queued-after-dispose', true, e => { e.op('dispose'); e.fire('start'); e.fire('interval'); e.fire('idle'); }],
    ['queued-idle-twice', true, e => { e.fire('idle'); e.fire('idle'); }],
    ['zero-handles', true, e => { e.op('restart'); e.op('pause'); e.op('resume'); }, { handle: 0 }],
    ['string-handles', true, e => { e.fire('start'); e.op('pause'); }, { handle: 'timer' }],
    ['object-handles', true, e => { e.fire('start'); e.op('dispose'); }, { handle: { token: 1 } }],
    ['throw-clearInterval', true, e => { e.fire('start'); e.fail = 'clearInterval'; e.op('pause'); e.fail = null; e.op('dispose'); }],
    ['throw-clearTimeout', true, e => { e.fail = 'clearTimeout'; e.op('pause'); e.fail = null; e.op('dispose'); }],
    ['throw-cancelFrame', true, e => { e.op('restart'); e.fail = 'cancelAnimationFrame'; e.op('dispose'); e.fail = null; e.op('dispose'); }],
    ['throw-requestFrame', true, e => { e.fail = 'requestAnimationFrame'; e.op('restart'); e.fail = null; e.op('restart'); }],
    ['throw-setInterval', true, e => { e.fail = 'setInterval'; e.fire('start'); e.fail = null; e.fire('start'); }],
    ['throw-setTimeout', false, e => { e.fail = 'setTimeout'; e.op('resume'); e.fail = null; e.op('resume'); }],
    ['throw-render-retains-frame', true, e => { e.op('restart'); e.fail = 'render'; e.fire('frame'); e.op('restart'); e.fail = null; e.fire('frame'); }],
    ['render-reentry-resume', true, e => { e.renderHook = () => e.op('resume'); e.op('restart'); e.fire('frame'); }],
    ['clear-reentry-resume', true, e => { e.fire('start'); e.callHook = name => { if (name === 'clearInterval') e.op('resume'); }; e.op('pause'); }],
    ['window-getter-reentry', true, e => { e.op('restart'); e.windowHook = () => e.op('pause'); e.op('dispose'); }],
    ['method-getter-reentry', true, e => { e.op('restart'); e.methodHook = name => { if (name === 'clearTimeout') e.op('pause'); }; e.op('dispose'); }],
    ['clock-reentry', true, e => { e.clockHook = () => e.op('pause'); e.op('restart'); }],
    ['clock-object-coercion', true, e => { e.now = { valueOf() { e.log.push(['clock-coerce']); return 1000; } }; e.op('restart'); e.now = 1200; e.fire('start'); }],
    ['clock-bigint-error', true, e => { e.now = 1000n; e.op('restart'); e.now = 1200; e.fire('start'); }],
    ['clock-nan', true, e => { e.now = NaN; e.op('restart'); e.fire('start'); }],
    ['clock-infinity', true, e => { e.now = Infinity; e.op('restart'); e.now = 1200; e.fire('start'); }],
    ['clock-throws', true, e => { e.failClock = true; e.op('restart'); e.failClock = false; e.op('restart'); }],
    ['synchronous-start-after-resume', false, e => { e.sync = 'setTimeout'; e.op('resume'); }],
    ['synchronous-frame-reentry', true, e => { e.sync = 'requestAnimationFrame'; e.renderHook = () => e.op('restart'); e.op('restart'); }],
    ['idle-render-error', true, e => { e.fail = 'render'; e.fire('idle'); e.fail = null; e.op('restart'); }],
    ['window-getter-error', true, e => { e.failWindow = true; e.op('pause'); e.failWindow = false; e.op('dispose'); }],
    ['method-getter-error', true, e => { e.failGet = 'clearTimeout'; e.op('dispose'); e.failGet = null; e.op('dispose'); }],
  ];
  try {
    for (const [name, focused, drive, options = {}] of cases) {
      const e = { log: [], now: 1000, fail: null, jobs: [], next: 0 };
      const failure = { deliberate: name };
      let manager;
      const snapshot = () => [manager.visible(), manager.paused()];
      const invoke = (label, callback) => {
        try { callback(); e.log.push([label, 'ok', ...snapshot()]); }
        catch (error) { e.log.push([label, error === failure ? 'same-error' : error.name, ...snapshot()]); }
      };
      e.op = op => invoke(op, () => manager[op]());
      e.fire = (kind, offset = 0) => {
        const job = e.jobs.filter(job => job.kind === kind)[offset];
        if (!job) throw new Error(`${name} missing ${kind} ${offset}`);
        invoke(`fire-${kind}-${job.id}`, () => job.callback());
      };
      const takeHook = key => { const hook = e[key]; e[key] = null; return hook; };
      const window = {};
      for (const method of ['setTimeout', 'setInterval', 'requestAnimationFrame', 'clearTimeout', 'clearInterval', 'cancelAnimationFrame']) {
        Object.defineProperty(window, method, { get() {
          e.log.push(['get', method]);
          takeHook('methodHook')?.(method);
          if (e.failGet === method) throw failure;
          return function(callback, delay) {
            e.log.push([method, this === window, method.startsWith('clear') || method.startsWith('cancel') ? callback : delay ?? null]);
            takeHook('callHook')?.(method);
            if (e.fail === method) throw failure;
            if (method === 'setTimeout' || method === 'setInterval' || method === 'requestAnimationFrame') {
              const id = ++e.next;
              e.jobs.push({ id, kind: method === 'setInterval' ? 'interval' : method === 'requestAnimationFrame' ? 'frame' : delay === 300000 ? 'idle' : 'start', callback });
              if (e.sync === method) { e.sync = null; callback(); }
              return Object.hasOwn(options, 'handle') ? options.handle : id;
            }
          };
        }});
      }
      const service = {
        get isFocused() { e.log.push(['focused']); return focused; },
        get window() { e.log.push(['window']); takeHook('windowHook')?.(); if (e.failWindow) throw failure; return window; }
      };
      Date.now = () => { e.log.push(['clock']); takeHook('clockHook')?.(); if (e.failClock) throw failure; return e.now; };
      const render = () => { e.log.push(['render', ...snapshot()]); takeHook('renderHook')?.(); if (e.fail === 'render') throw failure; };
      try {
        manager = factory(service, render);
        e.log.push(['constructed', ...snapshot()]);
        drive(e);
        // Snapshot before ownership cleanup, which has no JavaScript counterpart.
        results.push({ name, trace: structuredClone(e.log) });
      } finally {
        e.fail = null; e.failGet = null; e.failWindow = false;
        e.callHook = e.windowHook = e.methodHook = e.renderHook = null;
        manager?.dispose(); manager?.drop?.();
      }
    }
  } finally { Date.now = realClock; }
  return results;
}

export async function nativeDriver(factory) {
  const iframe = document.createElement('iframe');
  document.body.appendChild(iframe);
  const realm = iframe.contentWindow;
  let manager, deadline;
  const frames = [];
  try {
    await new Promise((resolve, reject) => {
      deadline = setTimeout(() => reject(Error('native cursor blink delivery deadline')), 5000);
      manager = factory({ isFocused: true, window: realm }, () => {
        frames.push([manager.visible(), manager.paused()]);
        if (frames.length === 1) queueMicrotask(() => manager.restart());
        else resolve();
      });
    });
    return frames;
  } finally {
    clearTimeout(deadline);
    manager?.pause(); manager?.dispose(); manager?.drop?.();
    iframe.remove();
  }
}

export function dropDriver(factory) {
  const calls = [], callbacks = [];
  const window = {
    setTimeout(callback) { callbacks.push(callback); return callbacks.length; },
    clearTimeout(id) { calls.push(['clearTimeout', id]); },
    setInterval() { throw Error('expired callback scheduled interval'); },
    requestAnimationFrame() { throw Error('expired callback scheduled frame'); }
  };
  const manager = factory({ isFocused: true, window }, () => { throw Error('expired callback rendered'); });
  manager.drop();
  for (const callback of callbacks) callback();
  return calls;
}

export function sourceFactory(service, render) {
  const manager = new globalThis.BlinkReference.CursorBlinkStateManager(render, service);
  return {
    visible: () => manager.isCursorVisible, paused: () => manager.isPaused,
    restart: () => manager.restartBlinkAnimation(), pause: () => manager.pause(),
    resume: () => manager.resume(), dispose: () => manager.dispose()
  };
}
