// Shared source/Rust operations. Playwright serializes this function into each
// page, so its helpers must remain local. No terminal logic is implemented here.
export async function htmlRowDifferenceCases() {
  const output = [];
  const write = (term, data) => new Promise(resolve => term.write(data, resolve));
  const snapshot = term => {
    const buffer = term.buffer.normal;
    return { cols: term.cols, rows: term.rows, x: buffer.cursorX, y: buffer.cursorY,
      base: buffer.baseY, active: term.buffer.active.type,
      lines: Array.from({ length: buffer.length }, (_, row) => {
        const line = buffer.getLine(row);
        return { wrapped: line.isWrapped, cells: Array.from({ length: term.cols }, (_, col) => {
          const cell = line.getCell(col);
          return [cell.getChars(), cell.getWidth(), cell.getFgColorMode(), cell.getFgColor(),
            cell.getBgColorMode(), cell.getBgColor(), cell.isBold(), cell.isUnderline()];
        }) };
      }) };
  };
  async function run(name, body) {
    const term = new Terminal({ cols: 12, rows: 4, scrollback: 20 });
    const addon = new SerializeAddon();
    try { term.loadAddon(addon); await write(term, '\x1b[31mABC\r\nSECOND');
      output.push({ name, result: await body(term, addon) });
    } finally { term.dispose(); }
  }
  function invoke(addon, log, start, end, sentinel) {
    const range = { get startCol() { log.push('startCol'); return {
      [Symbol.toPrimitive](hint) { log.push(`column:${hint}`); throw sentinel; }
    }; }, get startLine() { log.push('startLine'); return start; },
    get endLine() { log.push('endLine'); return end; } };
    const options = { get onlySelection() { log.push('onlySelection'); return false; },
      get range() { log.push('range'); return range; },
      get includeGlobalBackground() { log.push('header'); throw sentinel; } };
    try { return { value: addon.serializeAsHTML(options) }; }
    catch (error) { return { same: error === sentinel, name: error?.name, message: error?.message }; }
  }
  await run('html-raw-row-subtraction-throws', async (term, addon) => {
    const values = [];
    for (const side of ['end', 'start']) for (const phase of ['getter', 'call', 'valueOf', 'toString']) {
      const log = [], sentinel = {};
      const operand = label => {
        if (phase === 'valueOf' || phase === 'toString') return {
          valueOf() { log.push(`${label}:valueOf`); if (side === label && phase === 'valueOf') throw sentinel;
            return phase === 'toString' ? {} : 0; },
          toString() { log.push(`${label}:toString`); if (side === label) throw sentinel; return '0'; }
        };
        return { get [Symbol.toPrimitive]() {
          log.push(`${label}:getPrimitive`); if (side === label && phase === 'getter') throw sentinel;
          return hint => { log.push(`${label}:${hint}`); if (side === label) throw sentinel; return 0; };
        } };
      };
      values.push({ side, phase, result: invoke(addon, log, operand('start'), operand('end'), sentinel), log });
    }
    return { values, state: snapshot(term) };
  });
  await run('html-raw-row-subtraction-native-errors', async (term, addon) => {
    const values = [];
    for (const [label, make] of [
      ['symbol-end', () => [0, Symbol('end')]], ['symbol-start', () => [Symbol('start'), 0]],
      ['mixed-bigint-end', () => [0, 0n]], ['mixed-bigint-start', () => [0n, 0]],
      ['nonprimitive-end', () => [0, { [Symbol.toPrimitive]() { return {}; } }]],
      ['nonprimitive-start', () => [{ [Symbol.toPrimitive]() { return {}; } }, 0]],
      ['boxed-symbol-end', () => [0, Object(Symbol('end'))]],
      ['boxed-bigint-start', () => [Object(0n), 0]]
    ]) {
      const log = [], sentinel = {}, [start, end] = make();
      values.push({ label, result: invoke(addon, log, start, end, sentinel), log });
    }
    return { values, state: snapshot(term) };
  });
  await run('html-raw-row-subtraction-reentry', async (term, addon) => {
    const values = [];
    for (const action of ['reset', 'resize', 'nested', 'replace-start']) {
      term.reset(); term.resize(12, 4); await write(term, '\x1b[34mORIGINAL\r\nSECOND');
      const log = [], sentinel = {}; let nested;
      const start = { [Symbol.toPrimitive](hint) { log.push(`start:${hint}`); throw sentinel; } };
      const end = { [Symbol.toPrimitive](hint) {
        log.push(`end:${hint}`);
        if (action === 'reset') term.reset();
        if (action === 'resize') term.resize(6, 2);
        if (action === 'nested') nested = addon.serializeAsHTML({ range: { startLine: 0, endLine: 0, startCol: 0 } });
        if (action === 'replace-start') start[Symbol.toPrimitive] = hint => { log.push(`changed:${hint}`); throw sentinel; };
        return 0;
      } };
      values.push({ action, result: invoke(addon, log, start, end, sentinel), log, nested, state: snapshot(term) });
    }
    return values;
  });
  return output;
}
