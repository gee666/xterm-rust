// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
// This is an inspection adapter, not a second image store. All membership and
// iteration decisions come from Rust; weak handles do not retain evicted images.
const apply = Reflect.apply;
export function imageStorageInspection(reader) {
  const handles = new Map();
  const images = {
    get size() { return reader.size; },
    has(key) { return reader.has(key); },
    get(key) {
      for (const cached of handles.keys()) if (!reader.has(cached)) handles.delete(cached);
      if (!reader.has(key)) return undefined;
      let value = handles.get(key)?.deref();
      if (!value) {
        value = reader.get(key);
        handles.set(key, new WeakRef(value));
      }
      return value;
    },
    *keys() {
      const iterator = reader.entries();
      try {
        let key;
        while ((key = iterator.next()) !== undefined) yield key;
      } finally { iterator.free(); }
    },
    *values() { for (const key of this.keys()) yield this.get(key); },
    *entries() { for (const key of this.keys()) yield [key, this.get(key)]; },
    forEach(callback, thisArg) {
      // Native Map validates the callback even when the map is empty.
      if (typeof callback !== 'function') throw new TypeError('Image inspection callback is not a function');
      for (const [key, value] of this.entries()) apply(callback, thisArg, [value, key, this]);
    },
    [Symbol.iterator]() { return this.entries(); }
  };
  return { _images: images };
}
