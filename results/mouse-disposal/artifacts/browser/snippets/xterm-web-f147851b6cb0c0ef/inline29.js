
export function clipboardRead(provider, selection) {
  return provider === undefined ? navigator.clipboard.readText() : provider.readText(selection);
}
export function clipboardWrite(provider, selection, text) {
  return provider === undefined ? navigator.clipboard.writeText(text) : provider.writeText(selection, text);
}
export function clipboardEncode(codec, value) { return codec.encodeText(value); }
export function clipboardDecode(codec, value) { return codec.decodeText(value); }
export function clipboardString(value) { return `${value}`; }
export function clipboardEncodeBytes(value) { return new TextEncoder().encode(value); }
export function clipboardAppendByte(bin, bytes, index) { return bin + String.fromCharCode(bytes[index]); }
export function clipboardDecodeModern(value) { return new TextDecoder().decode(Uint8Array.fromBase64(value)); }
export function clipboardDecodeBytes(bytes) { return new TextDecoder().decode(bytes); }
export function clipboardAtob(value) { return atob(value); }
export function clipboardFallbackBytes(bin) { return new Uint8Array(bin.length); }
export function clipboardHasByte(bin, index) { return index < bin.length; }
export function clipboardCopyByte(bytes, bin, index) { bytes[index] = bin.charCodeAt(index); }
export function clipboardBtoa(value) { return btoa(value); }
export function clipboardPromise(value) { return value instanceof Promise; }
export function clipboardReadThen(promise, invoke) { return promise.then(data => invoke(data)); }
export function clipboardWriteThen(promise) { return promise.then(() => true); }
