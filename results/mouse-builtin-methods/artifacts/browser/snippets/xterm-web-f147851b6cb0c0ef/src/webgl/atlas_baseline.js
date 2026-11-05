// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
// Match the source module's platform snapshot, not the supplied canvas's realm.
const isNode = typeof process !== 'undefined' && 'title' in process &&
  (typeof navigator === 'undefined' || navigator.userAgent.startsWith('Node.js/'));
const userAgent = isNode ? 'node' : navigator.userAgent;
const baseline = userAgent.includes('Firefox') || userAgent.includes('Edge') ? 'bottom' : 'ideographic';
export function atlasTextBaseline() { return baseline; }
