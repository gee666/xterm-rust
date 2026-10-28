// Minimal browser ABI bootstrap. Terminal state and addon management are Rust.
import initialize, * as wasm from './pkg/xterm_web.js';

export * from './pkg/xterm_web.js';
export { initialize };

let runtime;
function sharedRuntime() { return runtime ??= new wasm.BrowserRuntime(); }

// The ephemeron keeps an addon's retained Terminal usable, while allowing an
// otherwise unreachable addon._terminal cycle to be collected entirely by JS.
const addonOwners = new WeakMap();

export class Terminal extends wasm.Terminal {
  constructor(options = {}) {
    super(options);
    addonOwners.set(this, wasm.Terminal.prototype._takeAddonOwner.call(this));
    this._setRuntime(sharedRuntime());
  }
  static get strings() { return sharedRuntime().strings; }
  loadAddon(addon) { return this._loadAddon(this, addon); }
}
