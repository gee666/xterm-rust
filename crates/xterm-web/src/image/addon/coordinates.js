// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
// Only ECMAScript coercion operators belong here. Buffer reads, captured map
// ownership and image lookup remain Rust. Capture ownKeys before user hooks.
const ownKeys = Reflect.ownKeys;
export function imageRowSum(start, index) {
  return +(start + index);
}
export function imageCellWord(index, words) {
  return index * words;
}
export function imageCellProperty(index) {
  return ownKeys({[index]: true})[0];
}
