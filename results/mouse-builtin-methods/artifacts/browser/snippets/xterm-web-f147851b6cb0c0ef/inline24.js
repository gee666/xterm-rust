
export function validateMarkerOffset(value) {
  if (value === undefined) value = 0;
  if (value === Infinity || isNaN(value) || value % 1 !== 0) {
    throw new Error('This API only accepts integers');
  }
  return value;
}
export function markerPosition(base, offset) { return base + offset; }
// Look up dispose when invoked, not during registration. Preserve its receiver.
export function disposeMarkerResource(value) { value.dispose(); }
export function markerWeakRef(value) { return new WeakRef(value); }
export function markerDeref(value) { return value.deref(); }
export function markerList(read) {
  const target = [];
  const sync = () => {
    const values = read();
    target.length = values.length;
    for (let i = 0; i < values.length; i++) target[i] = values[i];
  };
  const readonly = () => { throw new TypeError('Marker list is a read-only view'); };
  return new Proxy(target, {
    get(target, key, receiver) { sync(); return Reflect.get(target, key, receiver); },
    has(target, key) { sync(); return Reflect.has(target, key); },
    ownKeys(target) { sync(); return Reflect.ownKeys(target); },
    getOwnPropertyDescriptor(target, key) { sync(); return Reflect.getOwnPropertyDescriptor(target, key); },
    set: readonly, deleteProperty: readonly, defineProperty: readonly
  });
}
