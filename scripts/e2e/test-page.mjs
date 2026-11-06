// The pending readiness image delays navigation's load event until asynchronous
// WASM compilation and binding publication finish. Fetching the verified package
// avoids parsing a base64 copy and iterating every byte on the browser thread.
export function bootstrapSource() {
  return `import initialize from './browser/pkg/xterm_web.js';
import * as rust from './browser/xterm.mjs';
const token = document.currentScript.dataset.rustReady;
const ready = '/rust-ready?token=' + encodeURIComponent(token);
window.__rustE2EReady = (async () => {
  try {
    const response = await fetch('/pkg/xterm_web_bg.wasm');
    if (!response.ok) throw new Error('Rust WASM fetch failed: ' + response.status);
    await initialize({ module_or_path: response });
    if (typeof rust.Terminal !== 'function') throw new Error('Rust Terminal is not exported');
    const constructors = ['Terminal', 'AttachAddon', 'ClipboardAddon', 'FitAddon', 'ImageAddon',
      'ProgramStatusAddon', 'ProgressAddon', 'SearchAddon', 'SerializeAddon', 'Unicode11Addon',
      'UnicodeGraphemesAddon', 'LigaturesAddon', 'WebFontsAddon', 'WebLinksAddon', 'WebglAddon'];
    window.__rustE2EMissingBindings = [];
    for (const name of constructors) {
      if (typeof rust[name] === 'function') window[name] = rust[name];
      else window.__rustE2EMissingBindings.push(name);
    }
    if (typeof rust.loadFonts === 'function') window.loadFonts = rust.loadFonts;
    window.__rustE2EBackend = 'xterm-web wasm32';
    const notification = await fetch(ready + '&missing=' + encodeURIComponent(window.__rustE2EMissingBindings.join(',')), { method: 'POST' });
    if (!notification.ok) throw new Error('Fixture readiness release failed: ' + notification.status);
  } catch (error) {
    window.__rustE2EInitializationError = String(error.stack ?? error);
    console.error(error);
    await fetch(ready + '&failed=1', { method: 'POST' });
  }
})();
`;
}
