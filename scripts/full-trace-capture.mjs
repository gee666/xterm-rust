import assert from 'node:assert/strict';
import { hex } from './terminal-snapshot.mjs';

import { decodeUnits as units } from './full-trace-codec.mjs';
const U32_MAX = 0xffff_ffff;
function integer(text, min = 0, max = Number.MAX_SAFE_INTEGER) {
  assert.ok(typeof text === 'string' && /^-?(0|[1-9][0-9]*)$/.test(text) && text !== '-0', `Invalid snapshot integer ${text}`);
  const value = Number(text);
  assert.ok(Number.isSafeInteger(value) && value >= min && value <= max, `Snapshot integer out of range ${text}`);
  return value;
}
function numbers(text, count, min = 0, max = Number.MAX_SAFE_INTEGER) {
  const values = text === '' ? [] : text.split(',').map(value => integer(value, min, max));
  if (count !== undefined) assert.equal(values.length, count, 'Invalid integer field count');
  return values;
}
const bit = text => integer(text, 0, 1);
function mapId(value) {
  assert.ok(value === '-' || (value.length === 1 && '04567=ACEHKQRYZ'.includes(value)), `Unknown charset identity ${value}`);
  return value;
}
const mapIds = text => text ? text.split(',').map(mapId) : [];
function attributes(text) {
  const fields = text.split(',');
  assert.equal(fields.length, 4);
  return [integer(fields[0], 0, U32_MAX), integer(fields[1], 0, U32_MAX),
    integer(fields[2], 0, U32_MAX), integer(fields[3])];
}
function buffer(text, savedText) {
  const [coordinates, ...rows] = text.split('/');
  const position = numbers(coordinates, 6);
  const saved = savedText.split('~');
  assert.equal(saved.length, 8);
  if (rows.length > 1) assert.ok(rows.every(row => row !== ''), 'Empty row in populated buffer');
  const lines = rows.filter(row => row !== '').map(row => {
    const colon = row.indexOf(':');
    assert.ok(colon > 0, 'Missing wrapped-line delimiter');
    const cells = row.slice(colon + 1).split(';').map(cell => {
      const fields = cell.split(',');
      assert.ok(fields.length >= 6);
      const content = integer(fields[0], 0, U32_MAX);
      return { content, width: (content >>> 22) & 3, fg: integer(fields[1], 0, U32_MAX), bg: integer(fields[2], 0, U32_MAX),
        units: units(fields.slice(3, -2).join(',')), extended: integer(fields.at(-2), 0, U32_MAX), urlId: integer(fields.at(-1)) };
    });
    return { wrapped: bit(row.slice(0, colon)) === 1, cells };
  });
  const cursor = saved[0].split(',');
  assert.equal(cursor.length, 2);
  return { cursor: position.slice(0, 2), base: position[2], viewport: position[3],
    margins: position.slice(4), lines, savedCursor: [integer(cursor[0]), integer(cursor[1], -Number.MAX_SAFE_INTEGER)],
    savedAttributes: attributes(saved[1]), savedCharset: mapId(saved[2]), savedCharsets: mapIds(saved[3]), savedLevel: integer(saved[4]),
    savedModes: numbers(saved[5], 2, 0, 1), tabs: numbers(saved[6]), capacity: integer(saved[7]) };
}
export function parseTrace(text) {
  const lines = text.split(/\r?\n/);
  if (lines.at(-1) === '') lines.pop();
  assert.ok(lines.length > 0, 'Zero terminal snapshots');
  return lines.map(line => {
    const [state, eventText, ...extra] = line.split('\t');
    assert.equal(extra.length, 0, 'Unexpected trace columns');
    assert.notEqual(eventText, undefined);
    const parts = state.split('|');
    assert.equal(parts.shift(), 'RICH2', 'Unsupported snapshot schema');
    assert.equal(parts.length, 7, 'Snapshot schema changed');
    const headerText = parts[0].split(',');
    assert.equal(headerText.length, 16);
    const header = [integer(headerText[0], 2), integer(headerText[1], 1), bit(headerText[2]),
      ...attributes(headerText.slice(3, 7).join(',')), ...headerText.slice(7).map(bit)];
    assert.equal(header[2], bit(parts[6]));
    const meta = parts[5].split('~');
    assert.equal(meta.length, 14);
    assert.ok(['-', 'block', 'underline', 'bar'].includes(meta[5]), 'Unknown cursor style tag');
    if (meta[6] !== '-') bit(meta[6]);
    for (const index of [7, 8, 9, 10, 12]) bit(meta[index]);
    const events = eventText ? eventText.split('/') : [];
    return { dimensions: header.slice(0, 2), active: header[2], attributes: header.slice(3, 7), modes: header.slice(7),
      normal: buffer(parts[1], parts[3]), alternate: buffer(parts[2], parts[4]),
      charset: mapId(meta[0]), charsets: mapIds(meta[1]), level: integer(meta[2]),
      title: units(meta[3]), icon: units(meta[4]), cursorStyle: meta[5], cursorBlink: meta[6],
      colorSchemeUpdates: meta[7], win32: meta[8], cursorInitialized: meta[9], cursorHidden: meta[10],
      optionCursorStyle: units(meta[11]), optionCursorBlink: meta[12], unicodeVersion: units(meta[13]),
      events, replies: events.filter(event => event.startsWith('D')).map(event => units(event.slice(1))) };
  });
}
export function capture(records) {
  return {
    cells: records.map(record => ({ normal: record.normal.lines, alternate: record.alternate.lines })),
    cursor: records.map(record => ({ normal: [...record.normal.cursor], alternate: [...record.alternate.cursor], active: record.active })),
    margins: records.map(record => ({ normal: [...record.normal.margins], alternate: [...record.alternate.margins] })),
    // Cells are stored once above. These fields retain the remaining buffer state.
    normal: records.map(record => { const { lines, ...metadata } = record.normal; return metadata; }),
    alternate: records.map(record => { const { lines, ...metadata } = record.alternate; return metadata; }),
    events: records.map(record => record.events), replies: records.map(record => record.replies),
    properties: records.map(({ normal, alternate, events, replies, ...properties }) => properties)
  };
}
export function protocol(traces) {
  assert.ok(traces.length > 0, 'Zero trace scenarios');
  const output = [];
  for (const trace of traces) {
    const dimensions = [trace.cols ?? 5, trace.rows ?? 3, trace.scrollback ?? 6];
    assert.ok(dimensions.every(Number.isSafeInteger));
    assert.ok(dimensions[0] >= 2 && dimensions[1] >= 1 && dimensions[2] >= 0);
    output.push(['N', ...dimensions].join('\t'));
    assert.ok(trace.ops.length > 0, 'Zero operations in trace');
    for (const [kind, ...args] of trace.ops) {
      if (kind === 'W') output.push(`W\t${hex(args[0])}`);
      else if (kind === 'B') {
        assert.ok(args[0].every(byte => Number.isInteger(byte) && byte >= 0 && byte <= 255));
        output.push(`B\t${args[0].map(byte => byte.toString(16)).join(',')}`);
      } else if (kind === 'O') {
        const [key, value] = args;
        assert.match(key, /^[A-Za-z][A-Za-z0-9]*$/);
        let type, data;
        if (typeof value === 'boolean') { type = 'b'; data = +value; }
        else if (typeof value === 'number') { type = 'n'; data = Object.is(value, -0) ? '-0' : String(value); }
        else if (typeof value === 'string') { type = 's'; data = hex(value); }
        else {
          assert.ok(value && Object.getPrototypeOf(value) === Object.prototype);
          type = 'm'; data = Object.entries(value).map(([key, flag]) => {
            assert.match(key, /^[A-Za-z][A-Za-z0-9]*$/);
            assert.equal(typeof flag, 'boolean');
            return `${key}=${+flag}`;
          }).join(',');
        }
        output.push(['O', key, type, data].join('\t'));
      } else {
        assert.ok(['R', 'S', 'C', 'Q'].includes(kind), `Unknown trace command ${kind}`);
        output.push([kind, ...args].join('\t'));
      }
    }
  }
  return output.join('\n') + '\n';
}
