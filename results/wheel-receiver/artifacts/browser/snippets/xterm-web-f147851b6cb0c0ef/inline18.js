export function touchField(target, key) { return target[key]; }
export function missingTouchRecord(timestamp) { let data; return timestamp ? data.initialTimeStamp : data.initialTarget; }
export function touchCall1(callback, receiver, argument) { return Reflect.apply(callback, receiver, [argument]); }