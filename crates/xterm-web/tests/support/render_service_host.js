// Test-only bounded clocks/observers. No terminal or renderer behavior is implemented here.
export function withRenderHarness(callback) {
  let next = 1;
  const frames = new Map(), timers = new Map(), globalTimers = new Map(), idle = new Map();
  const observers = [];
  const owner = {
    devicePixelRatio: 1,
    requestAnimationFrame(fn) { const id = next++; frames.set(id, fn); return id; },
    cancelAnimationFrame(id) { frames.delete(id); },
    setTimeout(fn, delay = 0) { const id = next++; timers.set(id, { fn, delay }); return id; },
    clearTimeout(id) { timers.delete(id); },
    IntersectionObserver: class {
      constructor(fn, options) { this.fn = fn; this.options = options; this.disconnected = false; observers.push(this); }
      observe(target) { this.target = target; }
      disconnect() { this.disconnected = true; }
    }
  };
  const names = ['setTimeout', 'clearTimeout', 'requestIdleCallback', 'cancelIdleCallback'];
  const original = new Map(names.map(name => [name, Object.getOwnPropertyDescriptor(globalThis, name)]));
  const install = (name, value) => Object.defineProperty(globalThis, name, { configurable: true, writable: true, value });
  install('setTimeout', fn => { const id = next++; globalTimers.set(id, fn); return id; });
  install('clearTimeout', id => globalTimers.delete(id));
  install('requestIdleCallback', fn => { const id = next++; idle.set(id, fn); return id; });
  install('cancelIdleCallback', id => idle.delete(id));
  const host = {
    window: owner,
    frame() { const jobs = [...frames.values()]; frames.clear(); for (const fn of jobs) fn(42.5); },
    visibility(value) { this.entries([{ isIntersecting: value }]); },
    ratio(value) { this.entries([{ intersectionRatio: value }]); },
    entries(entries) { const observer = observers.findLast(item => !item.disconnected); if (!observer) throw new Error('No active observer'); observer.fn(entries); },
    runIdle() {
      const jobs = [...idle.values(), ...globalTimers.values()]; idle.clear(); globalTimers.clear();
      if (jobs.length > 16) throw new Error('Unexpected unbounded idle work');
      for (const fn of jobs) fn({ timeRemaining: () => 10000 });
    },
    safety() {
      const entry = [...timers].find(([, timer]) => timer.delay === 1000);
      if (!entry) throw new Error('No synchronized-output safety timeout');
      timers.delete(entry[0]); entry[1].fn();
    },
    pending() { return { frames: frames.size, timers: timers.size, idle: idle.size + globalTimers.size, observers: observers.filter(item => !item.disconnected).length }; }
  };
  try { return callback(host); }
  finally { for (const [name, descriptor] of original) { if (descriptor) Object.defineProperty(globalThis, name, descriptor); else delete globalThis[name]; } }
}
