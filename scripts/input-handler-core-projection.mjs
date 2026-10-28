// Decode the lossless terminal-snapshot protocol without normalizing cell data.
function integers(text, count) {
  const fields = text.split(',');
  if (fields.some(value => !/^-?\d+$/.test(value))) throw new Error('Invalid numeric snapshot syntax');
  const values = fields.map(Number);
  if (values.length !== count || values.some(value => !Number.isSafeInteger(value))) throw new Error('Invalid numeric snapshot fields');
  return values;
}
function units(text) {
  if (!text) return [];
  return text.split(',').map(value => {
    if (!/^[a-f0-9]{1,4}$/.test(value)) throw new Error('Invalid UTF-16 snapshot unit');
    return Number.parseInt(value, 16);
  });
}
function buffer(text) {
  const [position, ...encodedLines] = text.split('/');
  const lines = encodedLines.length === 1 && encodedLines[0] === '' ? [] : encodedLines;
  const [x, y, base, display, top, bottom] = integers(position, 6);
  const wrapped = [];
  const cells = lines.map(line => {
    const colon = line.indexOf(':');
    if (colon < 0 || !/^[01]$/.test(line.slice(0, colon))) throw new Error('Invalid wrapped row');
    wrapped.push(line[0] === '1');
    return line.slice(colon + 1).split(';').map(cell => {
      const fields = cell.split(',');
      if (fields.length < 6) throw new Error('Invalid packed cell');
      const [content, foreground, background] = integers(fields.slice(0, 3).join(','), 3);
      const [extended, url] = integers(fields.slice(-2).join(','), 2);
      const numbers = [content, foreground, background, extended, url];
      if (numbers.some(value => !Number.isSafeInteger(value))) throw new Error('Invalid packed cell numbers');
      return { content, foreground, background, chars: units(fields.slice(3, -2).join(',')), extended, url };
    });
  });
  return { x, y, base, display, top, bottom, wrapped, cells };
}
export function parseRows(text, prefix) {
  return text.replaceAll('\r\n', '\n').split('\n').filter(Boolean).map(line => {
    const fields = line.split('\t');
    if (fields.length !== 10) throw new Error('Invalid terminal capture record');
    return fields;
  }).filter(fields => !prefix || fields[0].startsWith(prefix));
}
export function project(rows) {
  const result = { cells: [], cursor: [], margins: [], normal: [], alternate: [], events: [], replies: [],
    properties: { traceIds: [], inputs: [], attributesAndModes: [], callbackStates: [] } };
  for (const fields of rows) {
    const state = fields[7].split('|');
    if (state.length !== 3) throw new Error('Invalid terminal state projection');
    const header = integers(state[0], 16);
    const normal = buffer(state[1]);
    const alternate = buffer(state[2]);
    result.cells.push({ normal: normal.cells, alternate: alternate.cells });
    result.cursor.push({ activeAlternate: header[2] === 1, normal: [normal.x, normal.y], alternate: [alternate.x, alternate.y] });
    result.margins.push({ normal: [normal.top, normal.bottom], alternate: [alternate.top, alternate.bottom] });
    result.normal.push({ base: normal.base, display: normal.display, wrapped: normal.wrapped });
    result.alternate.push({ base: alternate.base, display: alternate.display, wrapped: alternate.wrapped });
    const events = fields[8] ? fields[8].split('/') : [];
    result.events.push(events);
    result.replies.push(events.filter(event => event.startsWith('D')).map(event => units(event.slice(1))));
    result.properties.traceIds.push(fields[0]);
    result.properties.inputs.push({ cols: Number(fields[1]), rows: Number(fields[2]), scrollback: Number(fields[3]),
      screenReaderMode: fields[4] === '1', kind: fields[5], data: units(fields[6]) });
    result.properties.attributesAndModes.push(header);
    result.properties.callbackStates.push(fields[9] ? fields[9].split('~').map(value => {
      const colon = value.indexOf(':');
      if (colon < 1) throw new Error('Invalid callback state');
      return { event: value.slice(0, colon), packedState: value.slice(colon + 1) };
    }) : []);
  }
  return result;
}
