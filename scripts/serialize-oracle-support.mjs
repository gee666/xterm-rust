// UTF-16 fixture encoding and a structural IBuffer adapter for the JS oracle.
export const units = value => Array.from({ length: value.length }, (_, i) => value.charCodeAt(i).toString(16)).join('.');
const optional = value => value === undefined ? '-' : value;
const attr = cell => [cell.fg >>> 0, cell.bg >>> 0, cell.extended.ext >>> 0, cell.extended.urlId || 0].join('/');
const cellToken = cell => `${cell.content >>> 0}/${attr(cell)}/${units(cell.combinedData || '')}`;
const modeNames = ['applicationCursorKeysMode', 'applicationKeypadMode', 'bracketedPasteMode', 'insertMode',
  'originMode', 'reverseWraparoundMode', 'sendFocusMode', 'wraparoundMode', 'showCursor'];

export class FixtureWriter {
  constructor(defaultPalette) {
    this.defaultPalette = defaultPalette;
    this.palettes = new Map();
    this.rows = [];
    this.cases = [];
  }
  capture(terminal, options = {}, kind, output, error, name) {
    const theme = terminal.options.theme || {};
    const palette = terminal._core._themeService?.colors.ansi.map(color => color.css) || this.defaultPalette;
    const key = JSON.stringify(palette);
    if (!this.palettes.has(key)) this.palettes.set(key, this.palettes.size);
    const paletteId = terminal._core._themeService ? this.palettes.get(key) : '-';
    const modes = terminal.modes;
    const bits = modeNames.reduce((value, field, i) => value | (modes[field] ? 1 << i : 0), 0);
    const selection = terminal.getSelectionPosition();
    const ansi = [options.range?.start?.line ?? options.range?.start,
      options.range?.end?.line ?? options.range?.end, options.scrollback,
      options.excludeModes ? 1 : 0, options.excludeAltBuffer ? 1 : 0].map(optional).join(',');
    const html = [options.scrollback, options.onlySelection ? 1 : 0, options.includeGlobalBackground ? 1 : 0,
      options.range?.startLine, options.range?.endLine, options.range?.startCol].map(optional).join(',');
    const selected = selection ? [selection.start.x, selection.start.y, selection.end.x, selection.end.y].join(',') : '-';
    const index = this.cases.length;
    this.rows.push(['case', index, kind, terminal.cols, terminal.rows,
      terminal.buffer.active.type === 'alternate' ? 1 : 0, attr(terminal._core._inputHandler._curAttrData),
      bits, modes.mouseTrackingMode, `${terminal._core.buffer.scrollTop},${terminal._core.buffer.scrollBottom}`,
      ansi, html, selected, units(terminal.options.fontFamily), terminal.options.fontSize,
      theme.foreground === undefined ? '-' : units(theme.foreground),
      theme.background === undefined ? '-' : units(theme.background), paletteId,
      error ? 'error' : 'ok', units(error ? error.message : output)].join('\t'));
    for (const [id, buffer] of [['N', terminal.buffer.normal], ['A', terminal.buffer.alternate]]) {
      this.rows.push(['buffer', id, buffer.cursorX, buffer.cursorY, buffer.baseY, buffer.length].join('\t'));
      for (let row = 0; row < buffer.length; row++) {
        const line = buffer.getLine(row);
        if (!line) throw new Error(`Missing oracle row ${row}`);
        const cells = Array.from({ length: line.length }, (_, col) => cellToken(line.getCell(col)));
        this.rows.push(['line', id, row, line.isWrapped ? 1 : 0, cells.join('|')].join('\t'));
      }
    }
    this.cases.push({ index, name, kind, status: error ? 'error' : 'ok',
      ...(error ? { errorName: error.name, errorMessage: error.message } : {}) });
  }
  text() {
    const paletteRows = Array.from(this.palettes, ([json, id]) =>
      ['palette', id, JSON.parse(json).map(units).join(',')].join('\t'));
    return [...paletteRows, ...this.rows].join('\n') + '\n';
  }
}

export function fakeBuffer(CellData, rawRows, cols, cursorX = 0, cursorY = 0, baseY = 0, type = 'normal') {
  return {
    length: rawRows.length, cursorX, cursorY, baseY, type,
    getNullCell: () => new CellData(),
    getLine(row) {
      const data = rawRows[row];
      if (!data) return undefined;
      return {
        length: cols, isWrapped: !!data.wrapped,
        getCell(col, target) {
          const cell = data.cells[col];
          if (!cell) return undefined;
          if (!target) return cell;
          target.content = cell.content;
          target.fg = cell.fg;
          target.bg = cell.bg;
          target.extended = cell.extended;
          target.combinedData = cell.combinedData;
          return target;
        }
      };
    }
  };
}

export function directedCases(reference, record) {
  const { CellData, AttributeData, SerializeAddon } = reference;
  let outputs = 0;
  function cell(text = '', width = 1, fg = 0, bg = 0, ext = 0) {
    const value = CellData.fromCharData([fg, text, width, 0]);
    value.bg = bg;
    value.extended.ext = ext;
    return value;
  }
  function run(name, data, { cols = 4, rows = 2, cursor = [0, 0, 0], ansi = {}, html = {}, selection,
    activeAlternate = false, attrs = new AttributeData(), margins = [0, rows - 1], mode = {} } = {}) {
    const normal = fakeBuffer(CellData, data, cols, ...cursor);
    const alternate = fakeBuffer(CellData, [{ cells: Array.from({ length: cols }, () => cell()) }], cols, 0, 0, 0, 'alternate');
    const terminal = {
      cols, rows, buffer: { normal, alternate, active: activeAlternate ? alternate : normal },
      modes: { ...Object.fromEntries(modeNames.map(name => [name, name === 'wraparoundMode' || name === 'showCursor'])), mouseTrackingMode: 'none', ...mode },
      options: { fontFamily: 'monospace', fontSize: 15, theme: {} },
      _core: { buffer: { scrollTop: margins[0], scrollBottom: margins[1] }, _inputHandler: { _curAttrData: attrs } },
      getSelectionPosition: () => selection
    };
    const addon = new SerializeAddon();
    addon.activate(terminal);
    for (const [kind, options, method] of [['ansi', ansi, 'serialize'], ['html', html, 'serializeAsHTML']]) {
      try { record(terminal, options, kind, addon[method](options), undefined, name); }
      catch (error) { record(terminal, options, kind, undefined, error, name); }
      outputs++;
    }
  }
  for (const [mode, count] of [[0x1000000, 16], [0x2000000, 256]]) {
    for (let color = 0; color < count; color++) {
      for (const foreground of [true, false]) {
        const value = mode | color;
        run(`palette:${mode}:${color}:${foreground}`, [{ cells: [cell('a'),
          cell('b', 1, foreground ? value : 0, foreground ? 0 : value),
          cell('c', 1, 0x3010203, 0x3ff0080), cell('d')] }]);
      }
    }
  }
  for (const [field, mask] of [['fg', 0x4000000], ['fg', 0x8000000], ['fg', 0x10000000],
    ['fg', 0x20000000], ['fg', 0x40000000], ['fg', 0x80000000],
    ['bg', 0x4000000], ['bg', 0x8000000], ['bg', 0x40000000]]) {
    const styled = cell('b'); styled[field] = mask;
    run(`flag:${field}:${mask}`, [{ cells: [cell('a'), styled, cell('c'), cell()] }]);
  }
  for (let style = 0; style <= 5; style++) {
    for (const color of [0, 0x2000000, 0x20000ff, 0x3000000, 0x3ffffff]) {
      run(`underline:${style}:${color}`, [{ cells: [cell('a'), cell('b', 1, 0x10000000,
        0x10000000, style * 0x4000000 | color), cell('c'), cell()] }]);
    }
  }
  for (const text of ['', ' ', '&', '<', '>', '&x', 'e\u0301', '😀', '\ud800', '\udc00', '\0']) {
    run(`text:${units(text)}`, [{ cells: [cell(text), cell('x'), cell(), cell()] }]);
  }
  run('colored-null-gap', [{ cells: [cell('', 1, 0, 0x1000002), cell('', 1, 0, 0x1000002),
    cell('', 1, 0, 0x1000002), cell('x', 1, 0, 0x1000002), ...Array.from({ length: 6 }, () => cell())] },
    { cells: Array.from({ length: 10 }, () => cell()) }], { cols: 10, cursor: [4, 0, 0] });
  for (let gaps = 0; gaps < 4; gaps++) {
    for (const wide of [false, true]) {
      const first = Array.from({ length: 4 }, (_, i) => cell(i < 4 - gaps ? 'a' : ''));
      const next = wide ? [cell('中', 2), cell('', 0), cell('z'), cell()] : [cell('z'), cell(), cell(), cell()];
      run(`wrap:${gaps}:${wide}`, [{ cells: first }, { wrapped: true, cells: next }]);
    }
  }
  for (const range of [{ start: 0, end: 1 }, { start: 1, end: 1 }, { start: 0, end: 2 }, { start: 2, end: 2 }, { start: 1, end: 0 }]) {
    run(`range:${range.start}:${range.end}`, [{ cells: [cell('a'), cell(), cell(), cell()] },
      { cells: [cell('b'), cell(), cell(), cell()] }], { ansi: { range }, html: { range: { startLine: range.start, endLine: range.end, startCol: 0 } } });
  }
  for (const scrollback of [0, -1, -2, 99]) {
    run(`scrollback:${scrollback}`, [{ cells: [cell('a'), cell(), cell(), cell()] },
      { cells: [cell('b'), cell(), cell(), cell()] }], { ansi: { scrollback }, html: { scrollback } });
  }
  for (const name of modeNames) {
    run(`mode:${name}`, [{ cells: Array.from({ length: 4 }, () => cell()) }], {
      mode: { [name]: name !== 'wraparoundMode' && name !== 'showCursor' }, margins: [1, 1] });
  }
  return { outputs };
}

export function seededCases(reference, record) {
  const { CellData, AttributeData, SerializeAddon } = reference;
  const seed = 0x5e21a11;
  let state = seed;
  const random = () => {
    state ^= state << 13; state ^= state >>> 17; state ^= state << 5;
    return state >>> 0;
  };
  const strings = ['', ' ', 'a', '&', '<', '>', 'e\u0301', '\ud800', '\udc00', '😀', '中', '&x', '\0'];
  function style(cell, index) {
    const color = index % 5;
    const value = (index >>> 3) % 256;
    cell.fg = color === 1 ? 0x1000000 | value % 16 : color === 2 ? 0x2000000 | value : color === 3 ? 0x3000000 | random() % 0x1000000 : 0;
    cell.bg = color === 4 ? 0x2000000 | value : 0;
    cell.fg = (cell.fg | ((index >>> 6) & 63) * 0x4000000) >>> 0;
    cell.bg = (cell.bg | ((index >>> 12) & 3) * 0x4000000 | ((index >>> 14) & 1) * 0x40000000) >>> 0;
    if (index % 3 === 0) {
      cell.extended.underlineStyle = index % 6;
      cell.extended.underlineColor = index % 2 ? 0x3000000 | random() % 0x1000000 : 0x2000000 | value;
      cell.bg |= 0x10000000;
    }
    return cell;
  }
  for (let iteration = 0; iteration < 1024; iteration++) {
    const cols = 2 + random() % 9;
    const rows = 2 + random() % 3;
    const length = rows + random() % 4;
    const makeRows = size => Array.from({ length: size }, (_, y) => {
      const cells = [];
      for (let x = 0; x < cols; x++) {
        const text = strings[random() % strings.length];
        const wide = text === '中' && x < cols - 1;
        cells.push(style(CellData.fromCharData([0, text, wide ? 2 : 1, 0]), random()));
        if (wide) { cells.push(CellData.fromCharData([0, '', 0, 0])); x++; }
      }
      return { wrapped: y > 0 && random() % 2 === 1, cells };
    });
    const normal = fakeBuffer(CellData, makeRows(length), cols, random() % (cols + 1), random() % rows, length - rows);
    const alternate = fakeBuffer(CellData, makeRows(rows), cols, random() % (cols + 1), random() % rows, 0, 'alternate');
    const active = iteration % 7 === 0 ? alternate : normal;
    const modes = Object.fromEntries(modeNames.map(name => [name, random() % 2 === 1]));
    modes.mouseTrackingMode = ['none', 'x10', 'vt200', 'drag', 'any'][iteration % 5];
    const terminal = {
      cols, rows, buffer: { normal, alternate, active }, modes,
      options: { fontFamily: iteration % 2 ? 'monospace' : 'Test Font, serif', fontSize: 15.5, theme: {} },
      _core: { buffer: { scrollTop: 0, scrollBottom: rows - 1 }, _inputHandler: { _curAttrData: style(new AttributeData(), random()) } },
      getSelectionPosition: () => iteration % 3 ? undefined : ({ start: { x: 1, y: 0 }, end: { x: cols - 1, y: 1 } })
    };
    const addon = new SerializeAddon();
    addon.activate(terminal);
    const ansiOptions = iteration % 4 === 0 ? { range: { start: iteration % 11 === 0 ? length - 1 : 0, end: iteration % 11 === 0 ? 0 : length - 1 } }
      : { scrollback: [undefined, 0, 1, -rows, 999][iteration % 5], excludeModes: iteration % 3 === 0, excludeAltBuffer: iteration % 2 === 0 };
    const htmlOptions = { onlySelection: iteration % 3 === 0, includeGlobalBackground: iteration % 2 === 0,
      ...(iteration % 4 === 0 ? { range: { startLine: 0, endLine: Math.min(1, active.length - 1), startCol: 1 } } : {}) };
    for (const [kind, options, method] of [['ansi', ansiOptions, 'serialize'], ['html', htmlOptions, 'serializeAsHTML']]) {
      try { record(terminal, options, kind, addon[method](options), undefined, `seed:${seed}:${iteration}`); }
      catch (error) { record(terminal, options, kind, undefined, error, `seed:${seed}:${iteration}`); }
    }
  }
  return { seed, iterations: 1024, outputs: 2048 };
}
