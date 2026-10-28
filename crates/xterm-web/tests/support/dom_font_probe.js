// Hooks real owner-document canvases. Terminal logic remains in Rust.
export function installFontProbe(document) {
  const oldOffscreen = globalThis.OffscreenCanvas;
  globalThis.OffscreenCanvas = undefined;
  const original = document.createElement;
  const events = [];
  const contexts = [];
  const probe = { events, contexts, onFont: undefined };
  document.createElement = function (...args) {
    const element = original.apply(this, args);
    if (args[0] === 'canvas') {
      const context = element.getContext('2d');
      const descriptor = Object.getOwnPropertyDescriptor(Object.getPrototypeOf(context), 'font');
      const index = contexts.length;
      contexts.push(context);
      events.push(['canvas', index]);
      Object.defineProperty(context, 'font', {
        configurable: true,
        get() { return descriptor.get.call(this); },
        set(value) {
          events.push(['font', index, value]);
          probe.onFont?.();
          descriptor.set.call(this, value);
        }
      });
    }
    return element;
  };
  probe.restore = () => {
    document.createElement = original;
    globalThis.OffscreenCanvas = oldOffscreen;
  };
  return probe;
}
export function fontValue(probe, key, text) {
  return { [Symbol.toPrimitive](hint) {
    probe.events.push(['coerce', key, hint]);
    return text;
  } };
}
export function hookCss(element, callback) {
  const descriptor = Object.getOwnPropertyDescriptor(Node.prototype, 'textContent');
  Object.defineProperty(element, 'textContent', {
    configurable: true,
    get() { return descriptor.get.call(this); },
    set(value) { callback(value); descriptor.set.call(this, value); }
  });
}
