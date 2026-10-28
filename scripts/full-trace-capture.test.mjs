import assert from 'node:assert/strict';
import { test } from 'node:test';
import { parseTrace, capture, protocol } from './full-trace-capture.mjs';
import { decodeUnits, fromUnits } from './full-trace-source.mjs';
import { fullTraceScenarios } from './full-trace-scenarios.mjs';
import { assertJsonSafe } from './acceptance-validation.mjs';

function snapshot() {
  const header = [2, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0].join(',');
  const buffer = '0,0,0,0,0,0/0:4194304,0,0,,0,0;4194304,0,0,61,301,0,0';
  const saved = ['0,0', '0,0,0,0', '-', '', 0, '0,1', '0', 2].join('~');
  const meta = ['-', '', 0, 'd800', '', '-', '-', 0, 0, 1, 0, '62,6c,6f,63,6b', 0, '36'].join('~');
  return ['RICH2', header, buffer, buffer, saved, saved, meta, 0].join('|');
}

test('all packed cells, UTF16 units, saved state, events and replies survive parsing', () => {
  const records = parseTrace(`${snapshot()}\tD1b,5b,30,6e/A61/TAB2\n`);
  assert.equal(records.length, 1);
  assert.equal(records[0].normal.lines[0].cells[0].width, 1);
  assert.deepEqual(records[0].normal.lines[0].cells[0].units, []);
  assert.deepEqual(records[0].normal.lines[0].cells[1].units, [0x61, 0x301]);
  assert.deepEqual(records[0].normal.savedCursor, [0, 0]);
  assert.deepEqual(records[0].normal.savedModes, [0, 1]);
  assert.deepEqual(records[0].normal.tabs, [0]);
  assert.deepEqual(records[0].title, [0xd800]);
  assert.deepEqual(records[0].events, ['D1b,5b,30,6e', 'A61', 'TAB2']);
  assert.deepEqual(records[0].replies, [[0x1b, 0x5b, 0x30, 0x6e]]);
  const domain = capture(records);
  for (const key of ['cells', 'cursor', 'margins', 'normal', 'alternate', 'events', 'replies']) assert.ok(key in domain);
  assert.equal(domain.normal[0].capacity, 2);
  assert.equal('lines' in domain.normal[0], false, 'Do not duplicate complete cell arrays');
  assert.deepEqual(domain.cells[0].normal, records[0].normal.lines);
  assert.deepEqual(domain.properties[0].title, [0xd800]);
  assert.deepEqual(domain.properties[0].unicodeVersion, [0x36]);
  assertJsonSafe(domain);
});

test('missing schema fields, extra columns and zero captures fail', () => {
  assert.throws(() => parseTrace(''), /Zero terminal snapshots/);
  assert.throws(() => parseTrace(`${snapshot()}\textra\tcolumn\n`), /Unexpected trace columns/);
  assert.throws(() => parseTrace('0,0|empty\t\n'), /snapshot schema/i);
  assert.throws(() => parseTrace(`${snapshot().replace('2,1,0', 'NaN,1,0')}\t\n`), /Invalid snapshot integer/);
});

test('malformed UTF16, numeric cells, unsafe integers and flags cannot normalize to valid captures', () => {
  const changes = [
    ['61,301', '61z,301'], ['61,301', '10000,301'], ['61,301', '61,,301'],
    ['4194304,0,0,,0,0', 'NaN,0,0,,0,0'],
    ['4194304,0,0,,0,0', '4194304,4294967296,0,,0,0'],
    ['0,0,0,0,0,0/', '9007199254740993,0,0,0,0,0/'],
    ['0,0,0,0,0,0/', '0.5,0,0,0,0,0/'],
    ['0,0,0,0,0,0/', '0,,0,0,0,0/'],
    ['/0:', '/2:'], ['RICH2|2,1,0', 'RICH2|2,1,-0'],
    ['~36|0', '~36z|0'], ['~36|0', '~10000|0']
  ];
  for (const [oldText, replacement] of changes) {
    assert.ok(snapshot().includes(oldText));
    assert.throws(() => parseTrace(`${snapshot().replace(oldText, replacement)}\t\n`), undefined, replacement);
  }
  assert.throws(() => parseTrace(`${snapshot()}\tD61z\n`), /Invalid UTF-16/);
});

test('a changed cell or saved tab remains a capture difference', () => {
  const original = capture(parseTrace(`${snapshot()}\t\n`));
  const changedCell = capture(parseTrace(`${snapshot().replace('4194304,0,0,,0,0', '4194304,1,0,,0,0')}\t\n`));
  const changedTabs = capture(parseTrace(`${snapshot().replace('~0~2|', '~0,8~2|')}\t\n`));
  assert.notDeepEqual(changedCell, original);
  assert.notDeepEqual(changedTabs, original);
});

test('protocol preserves empty values, raw surrogates and typed option bags', () => {
  const text = protocol([{ cols: 2, rows: 1, scrollback: 0,
    ops: [['W', 'a\ud800\0'], ['B', [0, 255]], ['O', 'termName', ''],
      ['O', 'blinkIntervalDuration', -0], ['O', 'windowOptions', { pushTitle: true, popTitle: false }]] }]);
  assert.ok(text.includes('W\t61,d800,0\n'));
  assert.ok(text.includes('B\t0,ff\n'));
  assert.ok(text.includes('O\ttermName\ts\t\n'));
  assert.ok(text.includes('O\tblinkIntervalDuration\tn\t-0\n'));
  assert.ok(text.includes('O\twindowOptions\tm\tpushTitle=1,popTitle=0\n'));
  assert.equal(fromUnits(decodeUnits('61,d800,0')), 'a\ud800\0');
  for (const invalid of ['10000', 'gg', '-1', '1,', undefined]) assert.throws(() => decodeUnits(invalid), /Invalid UTF-16/);
  assert.throws(() => protocol([]), /Zero trace scenarios/);
  assert.throws(() => protocol([{ ops: [] }]), /Zero operations/);
  assert.throws(() => protocol([{ cols: 1, ops: [['W', 'x']] }]));
  assert.throws(() => protocol([{ ops: [['B', [256]]] }]));
  assert.throws(() => protocol([{ ops: [['unknown']] }]), /Unknown trace command/);
});

test('each declared User matrix has exact nonzero directed scenario count', () => {
  for (const [id, count] of [['vt-23', 432], ['vt-24', 126], ['vt-44', 288], ['vt-45', 240], ['vt-46', 150]]) {
    const traces = fullTraceScenarios(id);
    assert.equal(traces.length, count, id);
    assert.equal(new Set(traces.map(trace => trace.id)).size, count, id);
    assert.equal(protocol(traces).split('\n').filter(line => line.startsWith('N\t')).length, count);
  }
  assert.throws(() => fullTraceScenarios('vt-99'), /No full-trace scenario/);
});
