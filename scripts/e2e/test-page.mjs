// The original suites keep their fixture and assertions. Only their engine changes.
import * as rust from '/xterm.mjs';

const ready = (async () => {
  await rust.initialize();
  const constructors = ['Terminal', 'AttachAddon', 'ClipboardAddon', 'FitAddon', 'ImageAddon',
    'ProgramStatusAddon', 'ProgressAddon', 'SearchAddon', 'SerializeAddon', 'Unicode11Addon',
    'UnicodeGraphemesAddon', 'LigaturesAddon', 'WebLinksAddon', 'WebglAddon'];
  for (const name of constructors) {
    if (typeof rust[name] !== 'function') throw new Error(`Rust browser binding is missing ${name}`);
    window[name] = rust[name];
  }
  if (typeof rust.loadFonts === 'function') window.loadFonts = rust.loadFonts;
  window.__rustE2EBackend = 'xterm-web wasm32';
})();
window.__rustE2EReady = ready;
await ready;
