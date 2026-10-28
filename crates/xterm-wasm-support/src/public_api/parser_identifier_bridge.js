// JS boundary operations only. Rust owns identifier control flow and packing.
export function identifierProperty(value, key) { return value[key]; }
export function identifierCodeAt(value, index) { return value.charCodeAt(index); }
export function identifierLess(left, right) { return left < right; }
export function identifierGreater(left, right) { return left > right; }
export function identifierWord(value) { return value >>> 0; }
export function identifierClearPrefix(value) { value.prefix = undefined; }
