// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import test from 'node:test';
import { canonicalTsvRecords } from './tsv-records.mjs';

const lf = 'p:41,0d,0a\tH:p0|P:41\np:42\tP:42\n';
const crlf = lf.replace(/\n/g, '\r\n');
const sha = bytes => createHash('sha256').update(bytes).digest('hex');

test('LF and Windows CRLF records compare identically', () => {
  assert.equal(canonicalTsvRecords(lf), lf);
  assert.equal(canonicalTsvRecords(crlf), lf);
});
test('raw bytes remain distinct provenance from canonical records', () => {
  assert.notEqual(sha(Buffer.from(lf)), sha(Buffer.from(crlf)));
  assert.equal(sha(canonicalTsvRecords(lf)), sha(canonicalTsvRecords(crlf)));
});
test('actual payload token changes are never hidden', () => {
  assert.notEqual(canonicalTsvRecords(crlf.replace('0d,0a', '0d,0b')), lf);
  assert.notEqual(canonicalTsvRecords(`${crlf}\r\n`), lf);
});
test('standalone CR and escaped newline payload text are preserved', () => {
  const encoded = 'literal\\r\\n\t41\r42\n';
  assert.equal(canonicalTsvRecords(encoded), encoded);
});
test('BOM and unsupported input are not silently repaired', () => {
  assert.notEqual(canonicalTsvRecords(`\ufeff${crlf}`), lf);
  assert.throws(() => canonicalTsvRecords(null), TypeError);
});
