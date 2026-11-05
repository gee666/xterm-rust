
export function xtermMouseNumber(v) { return +v; }
export function xtermMouseMultiply(v, n) { return v * n; }
export function xtermMouseDivide(v, n) { return v / n; }
export function xtermMouseLess(v, n) { return v < n; }
export function xtermMouseBit(v, n) { return (v & n) !== 0; }
export function xtermMouseProperty(v, k) { return v[k]; }
export function xtermMouseWindow() { return window; }
export function xtermMousePadding(v) { return parseInt(v, 10); }
export function xtermMouseSubtract(a, b) { return a - b; }
