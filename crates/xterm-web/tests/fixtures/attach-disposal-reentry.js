// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Shared recording socket; both producers run their own actual addon logic.
async function attachDisposalReentryCases(make) {
  const rows = [];
  for (const trigger of ['dispose', 'close', 'error']) {
    for (const appendAt of ['message', 'close', 'error']) {
      for (const count of [0, 1, 3]) {
        for (const bidirectional of [false, true]) {
          const name = `${trigger}/${appendAt}/${count}/${bidirectional}`;
          const listeners = [], log = [], sent = [];
          let remaining = count, armed = false, host;
          const socket = {
            readyState: 1,
            addEventListener(type, handler) {
              listeners.push({ type, handler });
              log.push('add:' + type);
            },
            removeEventListener(type, handler) {
              log.push('remove:' + type);
              const index = listeners.findIndex(item => item.type === type && item.handler === handler);
              if (index !== -1) listeners.splice(index, 1);
              if (armed && type === appendAt && remaining > 0) {
                --remaining;
                host.activate();
              }
            },
            send(value) { sent.push(typeof value === 'string' ? value : [...value]); },
            emit(type, event = {}) {
              for (const item of listeners.slice()) {
                // EventTarget skips listeners removed during this dispatch.
                if (item.type === type && listeners.includes(item)) item.handler(event);
              }
            }
          };
          host = make(socket, bidirectional);
          try {
            host.activate();
            armed = true;
            if (trigger === 'dispose') host.dispose();
            else socket.emit(trigger);
            armed = false;
            const activeAfterDispose = listeners.map(item => item.type);
            socket.emit('message', { data: 'unexpected' });
            host.input('unexpected');
            const text = await host.flush();
            rows.push({ name, remaining, log: log.slice(), activeAfterDispose, sent, text });
          } finally {
            armed = false;
            host.dispose();
            host.free?.();
          }
        }
      }
    }
  }
  return rows;
}
