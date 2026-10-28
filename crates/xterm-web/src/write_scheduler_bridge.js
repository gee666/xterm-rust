// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
// Host-only scheduling from common/Async.ts and input/WriteBuffer.ts.
export function requestWrite(callback) { return setTimeout(callback, 0); }
export function cancelWrite(token) { if (token !== -1) clearTimeout(token); }
export function reportWriteError(error) { queueMicrotask(() => { throw error; }); }
