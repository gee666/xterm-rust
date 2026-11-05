// Copyright (c) 2022-2026 The xterm.js authors. All rights reserved. MIT license.
// Binding objects only. Rust owns OSC discovery, policy and pointer decisions.
export function oscBrowserLink(text, coordinates, handler, fallback) {
  const range = {
    start: { x: coordinates[0], y: coordinates[1] },
    end: { x: coordinates[2], y: coordinates[3] }
  };
  return {
    text,
    range,
    activate: (event, text) => handler ? handler.activate(event, text, range) : fallback(event, text),
    hover: (event, text) => handler?.hover?.(event, text, range),
    leave: (event, text) => handler?.leave?.(event, text, range)
  };
}
