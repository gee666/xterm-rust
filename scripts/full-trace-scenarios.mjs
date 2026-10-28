// Explicit source-bound input matrices. No expected Rust outputs are copied here.
const write = text => ['W', text];
const option = (key, value) => ['O', key, value];
function partitions(text) {
  return [
    { name: 'whole', ops: [write(text)] },
    { name: 'utf16-units', ops: text.split('').map(write) },
    { name: 'utf8-bytes', ops: [...new TextEncoder().encode(text)].map(byte => ['B', [byte]]) }
  ];
}
function printing() {
  const traces = [];
  for (const cols of [2, 3, 5]) for (const wrap of [false, true]) for (const insert of [false, true]) {
    for (const column of [cols - 1, cols]) for (const text of ['x', '界', '🙂', 'e\u0301', '\u0301', 'a\u0301\u0302']) {
      for (const partition of partitions(text)) {
        traces.push({ id: `edge-${cols}-${wrap}-${insert}-${column}-${text}-${partition.name}`, cols, rows: 3,
          ops: [option('screenReaderMode', true), write('A'.repeat(cols) + '\r\n' + 'B'.repeat(cols)),
            write(`\x1b[?7${wrap ? 'h' : 'l'}\x1b[4${insert ? 'h' : 'l'}\x1b[2;${column}H`),
            ...partition.ops, write('\x1b[6n')] });
      }
    }
  }
  return traces;
}
function softHyphens() {
  const traces = [];
  for (const cols of [2, 3, 5]) for (const atMargin of [false, true]) {
    for (const text of ['\u00ad', 'a\u00adb', 'e\u0301\u00ad界', '界\u00ad\u0301', '\u00ad\u00ad', 'a\u00ad\u0301', '🙂\u00adx']) {
      for (const partition of partitions(text)) {
        traces.push({ id: `hyphen-${cols}-${atMargin}-${text}-${partition.name}`, cols, rows: 3,
          ops: [option('screenReaderMode', true), write(atMargin ? 'x'.repeat(cols) : 'x'),
            ...partition.ops, write('\x1b[2b'), write('\x1b[6n')] });
      }
    }
  }
  return traces;
}
function positioned({ alternate, region, row, column, convertEol = false }) {
  const ops = [option('screenReaderMode', true), option('convertEol', convertEol)];
  if (alternate) ops.push(write('\x1b[?1049h'));
  ops.push(write('\x1b[31;44mAAAA\r\nBBBB\r\nCCCC\r\nDDDD'),
    write(region ? '\x1b[2;3r' : '\x1b[1;4r'), write(`\x1b[${row};${Math.min(column, 4)}H`));
  if (column === 5) ops.push(write('Z'));
  return ops;
}
function lineFeeds() {
  const traces = [];
  for (const alternate of [false, true]) for (const region of [false, true]) {
    for (const row of [1, 2, 3, 4]) for (const column of [1, 4, 5]) {
      for (const convertEol of [false, true]) for (const control of ['\n', '\v', '\f']) {
        traces.push({ id: `lf-${alternate}-${region}-${row}-${column}-${convertEol}-${control.charCodeAt(0)}`,
          cols: 4, rows: 4, ops: [...positioned({ alternate, region, row, column, convertEol }), write(control), write('\x1b[6n')] });
      }
    }
  }
  return traces;
}
function indices() {
  const traces = [];
  for (const alternate of [false, true]) for (const region of [false, true]) {
    for (const row of [1, 2, 3, 4]) for (const column of [1, 4, 5]) {
      for (const control of ['\x1bD', '\x1bE', '\x1bM', '\u0084', '\u0085']) {
        traces.push({ id: `index-${alternate}-${region}-${row}-${column}-${[...control].map(c => c.charCodeAt(0)).join('-')}`,
          cols: 4, rows: 4, ops: [...positioned({ alternate, region, row, column }), write(control), write('\x1b[6n')] });
      }
    }
  }
  return traces;
}
function bellAndReturn() {
  const traces = [];
  for (const cols of [2, 4, 5]) for (const prefix of ['', 'a', '界', 'e\u0301', '🙂']) {
    for (const pendingWrap of [false, true]) for (const control of ['\x07', '\r', '\x07\x07', '\r\r', '\x07\r']) {
      const initial = pendingWrap ? 'x'.repeat(cols) : 'x';
      traces.push({ id: `bell-cr-${cols}-${prefix}-${pendingWrap}-${[...control].map(c => c.charCodeAt(0)).join('-')}`,
        cols, rows: 3, ops: [option('screenReaderMode', true), write(initial + prefix),
          write(control), write('\x1b[2b'), write('\x1b[6n')] });
    }
  }
  return traces;
}
export function fullTraceScenarios(caseId) {
  switch (caseId) {
    case 'vt-23': return printing();
    case 'vt-24': return softHyphens();
    case 'vt-44': return lineFeeds();
    case 'vt-45': return indices();
    case 'vt-46': return bellAndReturn();
    default: throw new Error(`No full-trace scenario for ${caseId}`);
  }
}
