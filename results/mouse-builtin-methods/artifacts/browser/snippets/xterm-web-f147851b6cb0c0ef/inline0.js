
export function scrollWeakWindow(window) { return new WeakRef(window); }
export function scrollChainEntry(window, iframeElement) { return { window, iframeElement }; }
