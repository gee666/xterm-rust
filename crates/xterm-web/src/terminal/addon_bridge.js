// Copyright (c) 2019 The xterm.js authors. MIT license.
// Keep the closure's addon/record/array references in JavaScript so ordinary JS
// cycle collection works. Registration and disposal state transitions are Rust.
export function installAddonDispose(addon, entry, entries, route) {
  addon.dispose = () => route(entry, entries);
}
