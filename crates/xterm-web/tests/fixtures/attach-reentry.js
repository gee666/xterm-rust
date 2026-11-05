// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Shared source-method and Rust-terminal traces. No terminal engine lives here.
async function attachReentryCases(make) {
  const rows = [];
  const scenarios = [
    ['plain', 0], ['nested-once', 1], ['nested-twice', 2], ['nested-five', 5],
    ['binary-nested', 2, 'binary'], ['empty-nested', 1, 'empty'],
    ['throw-inner', 1, 'throw'], ['catch-inner', 1, 'catch'],
    ['dispose-in-getter', 0, 'dispose'], ['close-in-getter', 0, 'close'],
    ['error-in-getter', 0, 'error']
  ];
  for (const [name, depth, mode] of scenarios) {
    const log = [], sentinel = { name: 'original getter error' };
    const listeners = new Map();
    const socket = {
      addEventListener(type, handler) { listeners.set(type, handler); },
      removeEventListener(type, handler) {
        if (listeners.get(type) === handler) listeners.delete(type);
      },
      emit(type, event = {}) { return listeners.get(type)?.(event); }
    };
    const host = make(socket);
    let error = null;
    function event(level) {
      return { get data() {
        log.push('get:' + level);
        if ((mode === 'throw' || mode === 'catch') && level === depth) throw sentinel;
        if (level < depth) {
          try { socket.emit('message', event(level + 1)); }
          catch (thrown) {
            log.push(thrown === sentinel ? 'nested:original' : 'nested:' + String(thrown));
            if (mode !== 'catch') throw thrown;
          }
        }
        if (mode === 'dispose') host.dispose();
        if (mode === 'close' || mode === 'error') socket.emit(mode);
        log.push('return:' + level);
        if (mode === 'binary') return Uint8Array.of(65 + level).buffer;
        if (mode === 'empty' && level === depth) return new ArrayBuffer(0);
        return String.fromCharCode(65 + level);
      } };
    }
    try {
      try { socket.emit('message', event(0)); }
      catch (thrown) { error = thrown === sentinel ? 'original' : String(thrown); }
      const text = await host.flush();
      rows.push({ name, log, error, text, listeners: [...listeners.keys()] });
    } finally { host.dispose(); host.free?.(); }
  }
  return rows;
}
