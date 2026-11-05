
export function rowGet(v,k) { return v[k]; }
export function rowShift(v) { return v.shift(); }
export function rowLess(a,b) { return a<b; }
export function rowLessEqual(a,b) { return a<=b; }
export function rowAdd(a,b) { return a+b; }
export function rowIncrement(a) { return ++a; }
export function rowSubtract(a,b) { return a-b; }
export function rowSubtractOne(a) { return a-1; }
export function rowMultiply(a,b) { return a*b; }
export function rowAddress(a) { return a*3; }
export function rowMin(a,b) { return Math.min(a,b); }
