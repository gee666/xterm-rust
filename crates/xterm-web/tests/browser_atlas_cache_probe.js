// Native canvas observation only. No reference terminal or renderer is loaded.
let originalCreate, canvases, trace, removals, failRemove, failCreate, reenter, createReentry;
export const cacheFailure = {};
export function beginCacheCapture() {
  if (originalCreate) throw new Error('capture already active');
  trace = []; canvases = []; removals = 0; failRemove = false; failCreate = false; reenter = undefined; createReentry = undefined;
  originalCreate = document.createElement;
  document.createElement = function(...args) {
    if (failCreate) { failCreate = false; trace.push('create-failure'); throw cacheFailure; }
    if (createReentry) { const f = createReentry; createReentry = undefined; f(); }
    const canvas = originalCreate.apply(this, args);
    if (args[0] === 'canvas') {
      canvases.push(canvas);
      const remove = canvas.remove;
      canvas.remove = function() {
        removals++; trace.push('remove');
        if (reenter) { const f = reenter; reenter = undefined; f(); }
        if (failRemove) { failRemove = false; throw cacheFailure; }
        return remove.call(this);
      };
    }
    return canvas;
  };
}
export function failCacheRemove() { failRemove = true; }
export function failCacheCreate() { failCreate = true; }
export function reenterCacheRemove(f) { reenter = f; }
export function reenterCacheCreate(f) { createReentry = f; }
export function cacheNote(s) { trace.push(s); }
export function getCacheFailure() { return cacheFailure; }
export function endCacheCapture() {
  document.createElement = originalCreate;
  originalCreate = undefined; reenter = undefined; createReentry = undefined; failRemove = false; failCreate = false;
  const output = JSON.stringify({ trace, removals });
  canvases = undefined;
  return output;
}
export function cacheCanvasDigest(canvas) {
  let hash = 2166136261;
  for (const byte of canvas.getContext('2d').getImageData(0,0,512,512).data) hash = Math.imul(hash ^ byte,16777619) >>> 0;
  return hash;
}
export function sourceCache(name) { return globalThis.sourceCacheScenario(name); }
