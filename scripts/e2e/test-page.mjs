// The pending readiness image delays navigation's load event until asynchronous
// WASM compilation and binding publication finish. Chromium limits sync compile
// to 8 MB, which debug Rust distributions can legitimately exceed.
export function bootstrapSource(encodedWasm) {
  return `import initialize from './browser/pkg/xterm_web.js';
import * as rust from './browser/xterm.mjs';
const token = document.currentScript.dataset.rustReady;
const ready = '/rust-ready?token=' + encodeURIComponent(token);
window.__rustE2EReady = (async () => {
  try {
    const bytes = Uint8Array.from(atob(${JSON.stringify(encodedWasm)}), character => character.charCodeAt(0));
    await initialize({ module_or_path: bytes });
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
    const response = await fetch(ready + '&missing=' + encodeURIComponent(window.__rustE2EMissingBindings.join(',')), { method: 'POST' });
    if (!response.ok) throw new Error('Fixture readiness release failed: ' + response.status);
  } catch (error) {
    window.__rustE2EInitializationError = String(error.stack ?? error);
    console.error(error);
    await fetch(ready + '&failed=1', { method: 'POST' });
  }
})();
`;
}
