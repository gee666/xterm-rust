
export function mouseGet(v, k) { return v[k]; }
export function mouseSet(v, k, x) { v[k] = x; }
export function mouseAnd(v, n) { return v & n; }
export function mouseOr(n, v) { return n | v; }
export function mouseAdd(v, n) { return v + n; }
export function mouseGreater(v, n) { return v > n; }
export function mouseCharCode(v) { return v & 65535; }
export function mouseText(v) { return `${v}`; }
