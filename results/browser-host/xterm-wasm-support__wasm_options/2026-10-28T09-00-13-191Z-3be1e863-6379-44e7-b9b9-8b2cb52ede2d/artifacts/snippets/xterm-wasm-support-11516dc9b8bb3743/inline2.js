
export function optionData(target, key, value) { Object.defineProperty(target,key,{value,writable:true,enumerable:true,configurable:true}); }
export function optionNumber(value) { return +value; }
export function optionLess(value, rhs) { return value < rhs; }
export function optionLessEqual(value, rhs) { return value <= rhs; }
export function optionMultiply(value, rhs) { return value * rhs; }
export function optionString(value) { return `${value}`; }
export function optionConcat(value) { return '' + value; }
export function optionAdd(left, right) { return left + right; }
export function optionTruthy(value) { return !!value; }
export function optionProperty(value, key) { return value[key]; }
export function optionSetProperty(value,key,data) { value[key]=data; }
export function optionDeleteProperty(value,key) { return delete value[key]; }
export function optionKeys(value) { return Object.keys(value); }
export function optionError(message, type) { return type ? new TypeError(message) : new Error(message); }
