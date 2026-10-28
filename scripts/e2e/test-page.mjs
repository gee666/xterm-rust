// A classic deferred script must finish initialization before original page.goto
// calls return. Top-level await does not delay the browser load event.
export function bootstrapSource(encodedWasm) {
  return `import { initSync } from './browser/pkg/xterm_web.js';
import * as rust from './browser/xterm.mjs';
const bytes = Uint8Array.from(atob(${JSON.stringify(encodedWasm)}), character => character.charCodeAt(0));
initSync({ module: bytes });
const constructors = ['Terminal', 'AttachAddon', 'ClipboardAddon', 'FitAddon', 'ImageAddon',
  'ProgramStatusAddon', 'ProgressAddon', 'SearchAddon', 'SerializeAddon', 'Unicode11Addon',
  'UnicodeGraphemesAddon', 'LigaturesAddon', 'WebLinksAddon', 'WebglAddon'];
for (const name of constructors) {
  if (typeof rust[name] !== 'function') throw new Error('Rust browser binding is missing ' + name);
  window[name] = rust[name];
}
if (typeof rust.loadFonts === 'function') window.loadFonts = rust.loadFonts;
window.__rustE2EBackend = 'xterm-web wasm32';
`;
}
