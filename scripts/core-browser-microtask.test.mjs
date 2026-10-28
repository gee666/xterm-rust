// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Ownership protocol only. These tests do not execute the Rust browser service.
import assert from 'node:assert/strict';
import test from 'node:test';
import { queueFocusCache } from '../crates/xterm-web/src/core_browser_microtask.js';
const checkpoint = () => new Promise(resolve => setImmediate(resolve));

test('native queue delivers one normal cache invalidation', async () => {
  const calls = [];
  queueFocusCache(cancelled => calls.push(cancelled));
  assert.deepEqual(calls, []);
  await checkpoint();
  assert.deepEqual(calls, [false]);
});

test('failed-before-enqueue releases capture without changing the cache', async () => {
  const original = globalThis.queueMicrotask;
  const marker = {};
  const calls = [];
  globalThis.queueMicrotask = () => { throw marker; };
  try {
    assert.throws(() => queueFocusCache(cancelled => calls.push(cancelled)), error => error === marker);
    assert.deepEqual(calls, []);
    await checkpoint();
    assert.deepEqual(calls, [true]);
  } finally { globalThis.queueMicrotask = original; }
});

test('accepted-then-thrown enqueue preserves its already-queued invalidation', async () => {
  const original = globalThis.queueMicrotask;
  const marker = {};
  const calls = [];
  globalThis.queueMicrotask = callback => { original(callback); throw marker; };
  try {
    assert.throws(() => queueFocusCache(cancelled => calls.push(cancelled)), error => error === marker);
    assert.deepEqual(calls, []);
    await checkpoint();
    assert.deepEqual(calls, [false]);
  } finally { globalThis.queueMicrotask = original; }
});
