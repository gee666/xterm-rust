// Only dynamic-this forwarding and JS property operations live here. Ordinary
// reads box primitive receivers; strict writes must retain engine errors.
export function readWheelHandler(receiver) { return receiver._customWheelEventHandler; }
export function writeWheelHandler(receiver, handler) { receiver._customWheelEventHandler = handler; }
function invokeWheel(event) { return this._customWheelEventHandler(event); }
export function invokeWheelHandler(receiver, event) {
  return Reflect.apply(invokeWheel, receiver, [event]);
}
const installed = new WeakSet();
export function installMouseWheelMethods(receiver, set, allow) {
  const prototype = Object.getPrototypeOf(receiver);
  if (installed.has(prototype)) return;
  Object.defineProperties(prototype, {
    setCustomWheelEventHandler: {
      configurable: true, writable: true,
      value: function(handler) { return set(this, handler); }
    },
    allowCustomWheelEvent: {
      configurable: true, writable: true,
      value: function(event) { return allow(this, event); }
    }
  });
  installed.add(prototype);
}
