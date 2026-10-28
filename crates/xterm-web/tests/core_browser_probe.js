// Real isolated DOM nodes, with controlled DPR/media notifications where needed.
export function createProbe(controlled) {
  const frame = document.createElement('iframe');
  document.body.appendChild(frame);
  const win = frame.contentWindow;
  const doc = frame.contentDocument;
  const textarea = doc.createElement('textarea');
  doc.body.appendChild(textarea);
  const calls = [];
  let focused = true;
  let dpr = 1;
  let lastMedia;
  let nextId = 0;
  const ids = new WeakMap();
  const identify = callback => {
    if (typeof callback !== 'function') return String(callback);
    if (!ids.has(callback)) ids.set(callback, ++nextId);
    return ids.get(callback);
  };
  if (controlled) {
    Object.defineProperty(doc, 'hasFocus', { configurable: true, value() { calls.push('hasFocus'); return focused; } });
    Object.defineProperty(win, 'devicePixelRatio', { configurable: true, get() { calls.push('dpr'); return dpr; } });
    Object.defineProperty(win, 'matchMedia', { configurable: true, get() {
      calls.push('get:matchMedia');
      return query => {
        calls.push(`query:${query}`);
        const listeners = new Set();
        lastMedia = {
          addListener(callback) { calls.push(`media:add:${identify(callback)}`); if (typeof callback === 'function') listeners.add(callback); },
          removeListener(callback) { calls.push(`media:remove:${identify(callback)}`); listeners.delete(callback); },
          fire() { for (const callback of [...listeners]) callback.call(lastMedia, new win.Event('change')); },
        };
        return lastMedia;
      };
    } });
    for (const [target, label] of [[win, 'window'], [textarea, 'textarea']]) {
      const add = target.addEventListener;
      const remove = target.removeEventListener;
      target.addEventListener = function(type, callback, options) {
        calls.push(`${label}:add:${type}:${identify(callback)}:${String(options)}`);
        return Reflect.apply(add, this, [type, callback, options]);
      };
      target.removeEventListener = function(type, callback, options) {
        calls.push(`${label}:remove:${type}:${identify(callback)}:${String(options)}`);
        return Reflect.apply(remove, this, [type, callback, options]);
      };
    }
  }
  return {
    get window() { return win; }, get document() { return doc; }, get textarea() { return textarea; },
    setDocFocus(value) { focused = value; }, setDpr(value) { dpr = value; },
    focus() { textarea.dispatchEvent(new win.Event('focus')); },
    blur() { textarea.dispatchEvent(new win.Event('blur')); },
    resize() { win.dispatchEvent(new win.Event('resize')); },
    media() { lastMedia.fire(); },
    adopt(other) { other.document.body.appendChild(textarea); },
    calls() { return calls.slice(); },
    clearCalls() { calls.length = 0; },
    remove() { frame.remove(); },
  };
}
