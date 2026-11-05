
export function noneDisposable() { return { dispose() {} }; }
export function collectDisposable(container, result) {
  if (container) {
    if (Array.isArray(container)) container.push(result);
    else container.add(result);
  }
}
export function sourceDisposable(callback) { return { dispose() { callback(); } }; }
export function callbackDisposable(callback) { return {dispose:callback}; }
