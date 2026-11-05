
export function byteLength(value) { return value.length; }
function copyBytes(value) {
  const length = value.length;
  const result = new Uint8Array(length);
  for (let i = 0; i < length; ++i) result[i] = value[i];
  return result;
}
export function allBytes(value) { return copyBytes(value); }
export function rangeBytes(value, start, end) { return copyBytes(value.subarray(start, end)); }
export function debugBytes(value) {
  return Array.prototype.map.call(value, e => String.fromCharCode(e)).join('');
}
export function completeWrite(callback) { callback(); }
