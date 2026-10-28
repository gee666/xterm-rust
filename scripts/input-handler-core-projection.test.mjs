import { test } from 'node:test';
import assert from 'node:assert/strict';
import { parseRows, project } from './input-handler-core-projection.mjs';
import { assertJsonSafe } from './acceptance-validation.mjs';
const state = '2,1,0,0,0,0,0,0,0,1,0,0,0,0,0,0|1,0,0,0,0,0/0:4194401,0,0,d800,301,0,0;4194304,0,0,,0,0|0,0,0,0,0,0/';
const row = ['reverse-unit', '2', '1', '1', '0', 'W', 'd800,301', state, 'C/D1b,5b,30,6e/R0:0', ''];
test('projection preserves packed cells, lone UTF16, empty alternate buffer and exact replies', () => {
  const capture = project([row]);
  assert.deepEqual(capture.cells[0].normal[0][0].chars, [0xd800, 0x301]);
  assert.deepEqual(capture.cells[0].alternate, []);
  assert.deepEqual(capture.cursor, [{ activeAlternate: false, normal: [1, 0], alternate: [0, 0] }]);
  assert.deepEqual(capture.replies, [[[0x1b, 0x5b, 0x30, 0x6e]]]);
  assert.deepEqual(capture.events, [['C', 'D1b,5b,30,6e', 'R0:0']]);
  assertJsonSafe({ reference: capture, rust: project([row]) });
});
test('TSV terminators do not erase trailing empty callback fields', () => {
  assert.deepEqual(parseRows(row.join('\t') + '\r\n', 'reverse-'), [row]);
  assert.throws(() => parseRows(row.slice(0, -1).join('\t') + '\n'), /Invalid terminal capture/);
});
test('missing numeric fields and malformed UTF16 are rejected rather than normalized', () => {
  const bad = [...row];
  bad[7] = state.replace('4194401,0,0', '4194401,,0');
  assert.throws(() => project([bad]), /Invalid numeric/);
  const malformed = [...row];
  malformed[7] = state.replace('d800,301', 'zzzz,301');
  assert.throws(() => project([malformed]), /Invalid UTF-16/);
});
