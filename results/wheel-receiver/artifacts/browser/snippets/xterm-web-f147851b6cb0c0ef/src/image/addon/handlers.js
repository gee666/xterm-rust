// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
// Only binding identity is cached here. Membership, ordering, bytes and protocol
// mutations are read from Rust. A retained old record must not alias a replacement.
const apply = Reflect.apply;
function keys(reader) {
  const iterator = reader.entries();
  return (function* () {
    try {
      let key;
      while ((key = iterator.next()) !== undefined) yield key;
    } finally { iterator.free(); }
  })();
}
function keyView(reader) {
  return {
    get size() { return reader.size; },
    has(key) { return reader.has(key); },
    keys() { return keys(reader); }
  };
}
function readMap(reader, read) {
  const map = keyView(reader);
  Object.assign(map, {
    get: read,
    *values() { for (const key of this.keys()) yield this.get(key); },
    *entries() { for (const key of this.keys()) yield [key, this.get(key)]; },
    forEach(callback, thisArg) {
      if (typeof callback !== 'function') throw new TypeError('Image inspection callback is not a function');
      for (const [key, value] of this.entries()) apply(callback, thisArg, [value, key, this]);
    },
    [Symbol.iterator]() { return this.entries(); }
  });
  return map;
}
function records(reader, wrap) {
  const handles = new Map();
  return readMap(reader, key => {
    for (const cached of handles.keys()) if (!reader.has(cached)) handles.delete(cached);
    const current = reader.get(key);
    if (current === undefined) return undefined;
    const cached = handles.get(key), retained = cached?.handle.deref();
    if (retained && cached.identity === current.identity) {
      current.free();
      return retained;
    }
    const value = wrap(current);
    handles.set(key, {identity: current.identity, handle: new WeakRef(value)});
    return value;
  });
}
export function imageHandlerInspection(reader) {
  return records(reader, handler => {
    const view = { reset() { return handler.reset(); } };
    if (handler.kind === 'kitty') Object.assign(view, handler.kitty());
    return view;
  });
}
export function kittyHandlerInspection(images, pending, placements) {
  return {
    images: records(images, image => {
      const record = {};
      for (const key of ['id', 'data', 'width', 'height', 'format', 'compression']) {
        Object.defineProperty(record, key, {enumerable: true, get() { return image[key]; }});
      }
      return record;
    }),
    // This view intentionally offers membership only. It does not invent fake
    // mutable command/decoder values for the source-private pending Map.
    pendingTransmissions: keyView(pending),
    _kittyIdToStorageId: readMap(placements, key => placements.get(key))
  };
}
