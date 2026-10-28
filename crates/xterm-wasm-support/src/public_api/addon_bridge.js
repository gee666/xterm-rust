// Copyright (c) 2019 The xterm.js authors. MIT license.
// Keep closure references in JavaScript so normal JS cycle collection works.
// Rust controls registration, branching, callback order and captured indices.
export function installAddonDispose(addon, entry, entries, route) {
  addon.dispose = () => route(entry, entries);
}
// These calls retain JS receiver, getter and non-function TypeError semantics.
export function emptyAddonEntries() { return []; }
export function weakAddonOwner(entries) { return new WeakRef(entries); }
export function readAddonOwner(owner) { return owner.deref(); }
export function pushAddonEntry(entries, entry) { entries.push(entry); }
export function addonDisposeProperty(instance) { return instance.dispose; }
export function addonRecord(instance, dispose) { return { instance, dispose, isDisposed: false }; }
export function activateAddon(instance, terminal) { instance.activate(terminal); }
export function disposeAddon(instance) { instance.dispose(); }
export function applyAddonDispose(dispose, instance) { dispose.apply(instance); }
export function removeAddon(entries, index) { entries.splice(index, 1); }
