// Only JS object lifetime and the addon's original activation contract live here.
// State traversal, attribute transitions and ANSI/HTML generation run in Rust.
import { SerializeBackend, Terminal as RustTerminal } from './pkg/xterm_web.js';

export class SerializeAddon {
  #backend = new SerializeBackend();
  _terminal;
  activate(terminal) {
    this._terminal = terminal;
    if (terminal instanceof RustTerminal) this.#backend.activate(terminal);
    else this.#backend.clearHost();
  }
  serialize(options) {
    if (!this._terminal) throw new Error('Cannot use addon until it has been loaded');
    return this.#backend.serialize(options);
  }
  serializeAsHTML(options) {
    if (!this._terminal) throw new Error('Cannot use addon until it has been loaded');
    return this.#backend.serializeAsHTML(options);
  }
  dispose() {
    // Source disposal retains the terminal and leaves both methods usable.
  }
}
