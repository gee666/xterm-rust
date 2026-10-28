// Demo ownership only. Addon behavior and Terminal.loadAddon execute in Rust.
export const addonTypes = Object.freeze({
  attach: 'AttachAddon', clipboard: 'ClipboardAddon', fit: 'FitAddon', image: 'ImageAddon',
  ligatures: 'LigaturesAddon', 'program-status': 'ProgramStatusAddon', progress: 'ProgressAddon',
  search: 'SearchAddon', serialize: 'SerializeAddon', 'unicode-graphemes': 'UnicodeGraphemesAddon',
  unicode11: 'Unicode11Addon', 'web-fonts': 'WebFontsAddon', 'web-links': 'WebLinksAddon', webgl: 'WebglAddon'
});

export function requiredMethod(object, name) {
  if (typeof object?.[name] !== 'function') throw new Error(`Rust binding ${name} is not implemented`);
  return object[name].bind(object);
}

export class DemoAddons {
  constructor(bindings, terminal, log, update) {
    this.bindings = bindings;
    this.terminal = terminal;
    this.log = log;
    this.update = update;
    this.loaded = new Map();
  }
  names() { return [...this.loaded.keys()]; }
  get(name) {
    const entry = this.loaded.get(name);
    if (!entry) throw new Error(`Load the ${name} addon first`);
    return entry.addon;
  }
  load(name, socketUrl) {
    if (this.loaded.has(name)) throw new Error(`${name} is already loaded; use reload`);
    const Constructor = this.bindings[addonTypes[name]];
    if (typeof Constructor !== 'function') throw new Error(`Rust ${addonTypes[name] ?? name} is not exported`);
    let socket;
    let addon;
    if (name === 'attach') {
      if (!socketUrl) throw new Error('Attach needs a WebSocket URL');
      socket = new WebSocket(socketUrl);
      addon = new Constructor(socket, {});
    } else if (name === 'clipboard') {
      addon = new Constructor(undefined, undefined);
    } else {
      addon = new Constructor();
    }
    try { requiredMethod(this.terminal, 'loadAddon')(addon); }
    catch (error) {
      // The demo owns a socket it creates, not the attach addon. Close it on a
      // failed activation instead of leaking a connection with no owner.
      if (socket) socket.close();
      throw error;
    }
    const entry = { addon, socket, handles: [] };
    this.loaded.set(name, entry);
    for (const event of ['onDidChangeResults', 'onChange', 'onContextLoss']) {
      if (typeof addon[event] === 'function') {
        entry.handles.push(addon[event](value => this.log(`addon:${name}:${event}`, value)));
      }
    }
    this.log('addon:load', name);
    this.update();
    return addon;
  }
  ensure(name) { return this.loaded.has(name) ? this.get(name) : this.load(name); }
  unload(name) {
    const entry = this.loaded.get(name);
    if (!entry) throw new Error(`${name} is not loaded`);
    requiredMethod(entry.addon, 'dispose')();
    for (const handle of entry.handles) requiredMethod(handle, 'dispose')();
    if (entry.socket) entry.socket.close();
    this.loaded.delete(name);
    this.log('addon:unload', name);
    this.update();
  }
  reload(name, socketUrl) {
    if (this.loaded.has(name)) this.unload(name);
    return this.load(name, socketUrl);
  }
  dispose() {
    const failures = [];
    for (const name of this.names()) {
      try { this.unload(name); } catch (error) { failures.push(error); }
    }
    if (failures.length) throw new AggregateError(failures, 'Demo addon teardown failed');
  }
}
