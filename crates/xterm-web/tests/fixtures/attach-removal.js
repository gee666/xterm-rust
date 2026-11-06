// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Shared unchanged-source/actual-WASM traces. No terminal implementation here.
async function attachRemovalCases(create) {
  const rows = [];
  for (const entry of ['dispose', 'close', 'error']) {
    for (const failEvent of ['message', 'close', 'error']) {
      for (const phase of ['getter', 'call']) {
        for (const bidirectional of [false, true]) {
          const name = [entry, failEvent, phase, bidirectional].join('/');
          const trace = [], listeners = [], sent = [];
          const marker = { name };
          let failing = true, reads = 0;
          const socket = {
            readyState: 1,
            addEventListener(kind, callback) { listeners.push({ kind, callback, active: true }); },
            send(value) { sent.push(value); }
          };
          Object.defineProperty(socket, 'removeEventListener', {
            configurable: true,
            get() {
              reads++;
              trace.push('get');
              // Source resolves the getter on each removal, not at registration.
              const current = listeners.find(item => item.active && item.kind === failEvent);
              if (failing && phase === 'getter' && current &&
                  reads === ['message', 'close', 'error'].indexOf(failEvent) + 1) throw marker;
              return function(kind, callback) {
                const item = listeners.find(item => item.kind === kind && item.callback === callback);
                if (this !== socket || arguments.length !== 2 || !item) throw Error('listener tuple/receiver');
                trace.push(kind);
                if (failing && kind === failEvent && phase === 'call') throw marker;
                item.active = false;
              };
            }
          });
          const api = create(socket, bidirectional);
          try {
            api.activate();
            let caught;
            try {
              if (entry === 'dispose') api.dispose();
              else listeners.find(item => item.kind === entry).callback({});
            } catch (error) { caught = error; }
            if (caught !== marker) throw Error('lost thrown identity ' + name);
            const activeAfterFailure = listeners.filter(item => item.active).map(item => item.kind);
            // Any retained callback must still be callable after failed removal.
            listeners.find(item => item.kind === 'message').callback({ data: 'A' });
            api.input('x');
            const text = await api.flush();
            failing = false;
            api.dispose();
            const activeAfterSuccess = listeners.filter(item => item.active).map(item => item.kind);
            // Replace the method after success. All three handles call it again.
            Object.defineProperty(socket, 'removeEventListener', { value(kind, callback) {
              if (this !== socket || arguments.length !== 2 ||
                  !listeners.some(item => item.kind === kind && item.callback === callback)) throw Error('repeat tuple');
              trace.push('replacement:' + kind);
              // Calling during removal must not see a dropped Rust Closure.
              if (kind === 'message') callback({ data: 'B' });
            } });
            api.dispose();
            const repeatedText = await api.flush();
            rows.push({ name, trace, activeAfterFailure, activeAfterSuccess, sent, text, repeatedText });
          } finally {
            failing = false;
            // Cleanup never invokes the deliberately throwing override.
            Object.defineProperty(socket, 'removeEventListener', { value() {}, configurable: true });
            api.dispose();
            api.free?.();
          }
        }
      }
    }
  }
  return rows;
}

async function attachLiteralArguments(create) {
  const iterator = Array.prototype[Symbol.iterator];
  const marker = Symbol('unexpected spread');
  let count = 0;
  const socket = { readyState: 1, send() {}, addEventListener() {} };
  Object.defineProperty(socket, 'removeEventListener', { configurable: true,
    get() {
      Array.prototype[Symbol.iterator] = () => { throw marker; };
      return function(kind, handler) {
        Array.prototype[Symbol.iterator] = iterator;
        if (this !== socket || arguments.length !== 2 || typeof handler !== 'function') throw Error('literal arguments');
        count++;
      };
    }
  });
  const api = create(socket, false);
  try {
    api.activate();
    api.dispose();
    if (count !== 3) throw Error('literal argument calls');
    return count;
  } finally {
    Array.prototype[Symbol.iterator] = iterator;
    Object.defineProperty(socket, 'removeEventListener', { value() {}, configurable: true });
    api.dispose(); api.free?.();
  }
}

async function attachRawRemovalErrors(create) {
  let count = 0;
  for (const marker of [undefined, null, false, 17, 'removal', Symbol('removal'), {}]) {
    for (const phase of ['getter', 'call']) {
      for (const entry of ['dispose', 'close', 'error']) {
        const listeners = [];
        const socket = { readyState: 1, send() {},
          addEventListener(kind, callback) { listeners.push({ kind, callback }); }
        };
        Object.defineProperty(socket, 'removeEventListener', { configurable: true,
          get() { if (phase === 'getter') throw marker; return () => { throw marker; }; }
        });
        const api = create(socket, false);
        try {
          api.activate();
          let thrown = false, caught;
          try {
            if (entry === 'dispose') api.dispose();
            else listeners.find(item => item.kind === entry).callback({});
          } catch (error) { thrown = true; caught = error; }
          if (!thrown || caught !== marker) throw Error('raw removal identity ' + phase + '/' + entry);
          count++;
        } finally {
          Object.defineProperty(socket, 'removeEventListener', { value() {}, configurable: true });
          api.dispose(); api.free?.();
        }
      }
    }
  }
  return count;
}

async function attachRetainedRegistration(create) {
  const marker = Symbol('registration');
  let callback, caught;
  const socket = { readyState: 1, send() {},
    addEventListener(kind, handler) { callback = handler; throw marker; },
    removeEventListener() {}
  };
  const api = create(socket, false);
  try {
    try { api.activate(); } catch (error) { caught = error; }
    if (caught !== marker) throw Error('registration error identity');
    callback({ data: 'retained' });
    const text = await api.flush();
    if (text !== 'retained') throw Error('retained registration handler failed: ' + text);
    return { text, identity: true };
  } finally { api.dispose(); api.free?.(); }
}
