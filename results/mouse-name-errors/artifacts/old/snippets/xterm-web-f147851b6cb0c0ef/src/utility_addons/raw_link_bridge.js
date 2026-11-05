// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
// Binding objects only. Rust computes URLs and owns provider/pointer decisions.
export function webBrowserLink(text, coordinates, activate, options) {
  const link = {
    text,
    range: {
      start: { x: coordinates[0], y: coordinates[1] },
      end: { x: coordinates[2], y: coordinates[3] }
    },
    activate
  };
  link.leave = options.leave;
  link.hover = (event, uri) => {
    if (options.hover) {
      const { range } = link;
      options.hover(event, uri, range);
    }
  };
  return link;
}
