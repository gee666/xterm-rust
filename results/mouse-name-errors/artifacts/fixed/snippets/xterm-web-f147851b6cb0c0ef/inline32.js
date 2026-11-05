
export function scrollGet(value, key) { return value[key]; }
export function scrollSet(value, key, next) { 'use strict'; value[key] = next; }
export function scrollMethod0(value, key) { return value[key](); }
export function scrollMethod1(value, key, a) { return value[key](a); }
export function scrollMethod2(value, key, a, b) { return value[key](a,b); }
export function scrollSubtract(a,b) { return a-b; }
export function scrollMultiply(a,b) { return a*b; }
export function scrollDivide(a,b) { return a/b; }
export function scrollNegate(value) { return -value; }
export function scrollClock() { return Date.now(); }
