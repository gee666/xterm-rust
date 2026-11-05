
export function joinerGet(v,k) { return v[k]; }
export function joinerSet(v,k,x) { 'use strict'; v[k]=x; }
export function joinerLess(a,b) { return a<b; }
export function joinerLessEqual(a,b) { return a<=b; }
export function joinerMinStart(a,b) { return Math.min(a[0],b[0]); }
export function joinerMaxEnd(a,b) { return Math.max(a[1],b[1]); }
export function joinerSubtractOne(a) { return a-1; }
export function joinerInsert(v,i,x) { v.splice(i,0,x); }
export function joinerRemove(v,i) { v.splice(i,1); }
export function joinerPush(v,x) { v.push(x); }
export function joinerPushFrom(target,source,index) { target.push(source[index]); }
export function joinerReport(e) { console.error(e); }
export function joinerCall(record,text) { return record.handler(text); }
export function joinerRecord(id,handler) { return {id,handler}; }
