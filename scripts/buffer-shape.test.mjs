import assert from 'node:assert/strict';
import test from 'node:test';
import { shapeBufferCell, shapeBufferLine, shapeExtendedCell } from '../crates/xterm-wasm-support/src/public_api/buffer_shape.js';

function snapshot(fields, values) {
  const data = new WeakMap();
  class Snapshot { constructor() { this.__wbg_ptr = 42; data.set(this, { ...values }); } }
  for (const field of fields) Object.defineProperty(Snapshot.prototype, field, {
    configurable: true, get() { return data.get(this)[field]; }, set(value) { data.get(this)[field] = value; }
  });
  return new Snapshot();
}
test('line enumeration exposes actual packed state instead of an address or empty object', () => {
  const packed = { _data: new Uint32Array([1, 2, 3]), length: 1 };
  const line = shapeBufferLine(snapshot(['_line'], { _line: packed }));
  assert.deepEqual(Object.keys(line), ['_line']);
  assert.deepEqual(JSON.parse(JSON.stringify(line)), { _line: { _data: { 0: 1, 1: 2, 2: 3 }, length: 1 } });
  line._line = { _data: new Uint32Array([4, 5, 6]), length: 1 };
  assert.equal(line._line._data[0], 4);
});
test('cell enumeration reads live fields in source order, never the internal pointer', () => {
  const fields = ['fg', 'bg', 'extended', 'content', 'combinedData'];
  const initial = { fg: 0, bg: 0, extended: { _ext: 0, _urlId: 0 }, content: 0, combinedData: '' };
  const cell = shapeBufferCell(snapshot(fields, initial));
  assert.deepEqual(Object.keys(cell), fields);
  assert.equal(JSON.stringify(cell), JSON.stringify(initial));
  assert.equal(cell.__wbg_ptr, 42);
  cell.content = 65; cell.fg = 1; cell.combinedData = '\ud800';
  assert.deepEqual({ ...cell }, { ...initial, content: 65, fg: 1, combinedData: '\ud800' });
  assert.equal(Object.hasOwn(cell, 'toJSON'), false);
  assert.equal(Object.getOwnPropertyDescriptor(cell, '__wbg_ptr').writable, true);
});
test('extended enumeration exposes raw fields rather than the computed ext value', () => {
  const attrs = snapshot(['_ext', '_urlId', 'payload'], { _ext: 2, _urlId: 7, payload: undefined });
  Object.defineProperty(Object.getPrototypeOf(attrs), 'ext', { get() { return 5 << 26; } });
  shapeExtendedCell(attrs);
  assert.deepEqual(Object.keys(attrs), ['_ext', '_urlId']);
  assert.equal(JSON.stringify(attrs), '{"_ext":2,"_urlId":7}');
  attrs._ext = 3; attrs._urlId = 11;
  assert.equal(JSON.stringify(attrs), '{"_ext":3,"_urlId":11}');
  assert.equal(attrs.ext, 5 << 26);
});
test('an already present extended payload keeps its value and enumeration position', () => {
  const attrs = snapshot(['_ext', '_urlId', 'payload'], { _ext: 0, _urlId: 0, payload: { application: true } });
  shapeExtendedCell(attrs);
  assert.deepEqual(Object.keys(attrs), ['_ext', '_urlId', 'payload']);
  assert.deepEqual(JSON.parse(JSON.stringify(attrs)), { _ext: 0, _urlId: 0, payload: { application: true } });
});
