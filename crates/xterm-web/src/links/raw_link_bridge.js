// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
// Host operations only. Provider ordering and link state stay in Rust.
export function linkProviderToken(provider, dispose) { return { dispose: () => dispose(provider) }; }
export function linkProperty(value, key) { return value[key]; }
export function linkSet(value, key, entry) { value[key] = entry; }
export function linkCall(value, receiver, args) { return Reflect.apply(value, receiver, args); }
export function linkMap(value) { return value?.map(link => link); }
export function linkAdd(a, b) { return a + b; }
export function linkMultiply(a, b) { return a * b; }
export function linkSubtract(a, b) { return a - b; }
export function linkLessEqual(a, b) { return a <= b; }
export function linkLess(a, b) { return a < b; }
export function linkIncrement(value) { return ++value; }
export function linkDescriptors(value, link, pointerGet, pointerSet, underlineGet, underlineSet) {
  Object.defineProperties(value, {
    pointerCursor: { get: pointerGet, set: v => pointerSet(link, v) },
    underline: { get: underlineGet, set: v => underlineSet(link, v) }
  });
}
