
const clipboardApply = Reflect.apply;
export function clipboardProperty(value, key) { return value[key]; }
export function clipboardInvoke1(fn, receiver, a) { return clipboardApply(fn, receiver, [a]); }
export function clipboardInvoke2(fn, receiver, a, b) { return clipboardApply(fn, receiver, [a,b]); }
export function clipboardSubtract(value, rhs) { return value - rhs; }
export function clipboardPrevent(event) { event.preventDefault(); }
export function clipboardStop(event) { event.stopPropagation(); }
