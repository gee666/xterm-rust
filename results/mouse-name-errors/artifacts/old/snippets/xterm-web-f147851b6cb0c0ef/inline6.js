
export function monitorSetCapture(target, id) { target.setPointerCapture(id); }
export function monitorReleaseCapture(target, id) { target.releasePointerCapture(id); }
export function monitorWindow(value) {
  if (value?.ownerDocument?.defaultView) return value.ownerDocument.defaultView;
  if (value?.view) return value.view;
  return window;
}
