// Shared original/Rust operations. All engine calls run in the page's realm.
export async function serializationCases() {
  const output = [];
  const write = (term, data) => new Promise(resolve => term.write(data, resolve));
  const capture = fn => { try { return { value: fn() }; } catch (error) { return { error: error.name, message: error.message }; } };
  const snapshot = term => {
    const methods = ['getChars', 'getCode', 'getWidth', 'getFgColorMode', 'getBgColorMode', 'getFgColor', 'getBgColor',
      'isBold', 'isItalic', 'isDim', 'isUnderline', 'isBlink', 'isInverse', 'isInvisible', 'isStrikethrough', 'isOverline',
      'getUnderlineStyle', 'getUnderlineColor', 'getUnderlineColorMode'];
    const buffer = view => ({ type: view.type, x: view.cursorX, y: view.cursorY, base: view.baseY, viewport: view.viewportY,
      lines: Array.from({ length: view.length }, (_, row) => {
        const line = view.getLine(row);
        return { wrapped: line.isWrapped, cells: Array.from({ length: term.cols }, (_, col) => {
          const cell = line.getCell(col); return methods.map(method => cell[method]());
        }) };
      }) });
    return { cols: term.cols, rows: term.rows, modes: { ...term.modes }, active: term.buffer.active.type,
      normal: buffer(term.buffer.normal), alternate: buffer(term.buffer.alternate) };
  };
  async function run(name, body, opened = false) {
    const parent = document.createElement('div'); parent.style.cssText = 'width:640px;height:240px'; document.body.append(parent);
    const term = new Terminal({ cols: 12, rows: 4, scrollback: 20 });
    const addon = new SerializeAddon();
    const warnings = [], warn = console.warn;
    console.warn = (...args) => warnings.push(args.map(String).join(' '));
    try {
      term.loadAddon(addon); if (opened) term.open(parent);
      output.push({ name, result: await body(term, addon, parent), warnings });
    } catch (error) { output.push({ name, thrown: { name: error.name, message: error.message }, warnings }); }
    finally { console.warn = warn; term.dispose(); parent.remove(); }
  }
  const unloaded = new SerializeAddon();
  output.push({ name: 'before-load', result: [capture(() => unloaded.serialize()), capture(() => unloaded.serializeAsHTML())] });
  await run('packed-line-enumeration', async (term) => {
    await write(term, 'A中e\u0301\x1b[4:3;58:2::8:9:10mX');
    const capture = () => {
      const row = term.buffer.normal.getLine(0);
      return JSON.stringify({ data: row._data, combined: row._combined, extended: row._extendedAttrs, length: row.length, wrapped: row._isWrapped });
    };
    const before = capture(); await write(term, '\r\x1b[0mZ'); return [before, capture()];
  });
  await run('buffer-cell-enumeration', async (term) => {
    const cell = term.buffer.normal.getNullCell(), extended = cell.extended;
    const initial = { keys: Object.keys(cell), extendedKeys: Object.keys(extended), json: JSON.stringify(cell) };
    cell.content = (1 << 22) | 65; cell.fg = 1; cell.extended._ext = 2; cell.extended._urlId = 7;
    const mutated = { json: JSON.stringify(cell), chars: cell.getChars(), style: cell.getUnderlineStyle(), ext: extended.ext,
      cloneKeys: Object.keys(extended.clone()), clone: JSON.stringify(extended.clone()) };
    await write(term, 'abc'); const reused = term.buffer.normal.getLine(0).getCell(1, cell);
    return { initial, mutated, same: reused === cell, reused: JSON.stringify(reused) };
  });
  await run('ansi-options-modes-alt', async (term, addon) => {
    await write(term, 'abc\x1b[31mRED\x1b[0m\r\n中e\u0301\r\nnext\r\nlast\r\nscroll');
    const values = [addon.serialize(), addon.serialize({ scrollback: 0 }), addon.serialize({ range: { start: 1, end: 2 } }),
      addon.serialize(null), addon.serialize(false), addon.serialize({ scrollback: '0' })];
    await write(term, '\x1b[?1049hALT\x1b[?1h\x1b[?7l\x1b[?25l\x1b[2;3r');
    values.push(addon.serialize(), addon.serialize({ excludeModes: true }), addon.serialize({ excludeAltBuffer: true }));
    return values;
  });
  await run('ansi-getter-phases', async (term, addon) => {
    await write(term, 'content'); const log = [];
    const range = { get start() { log.push('start'); return 0; }, get end() { log.push('end'); return 0; } };
    const options = { get range() { log.push('range'); return range; },
      get excludeAltBuffer() { log.push('alt'); term.reset(); return false; },
      get excludeModes() { log.push('modes'); return false; } };
    const ansi = addon.serialize(options);
    const sentinel = {}; let same = false;
    try { addon.serialize({ get excludeModes() { throw sentinel; } }); } catch (error) { same = error === sentinel; }
    return { ansi, log, same };
  });
  await run('ansi-warning-reset-restores-current-attrs', async (term, addon) => {
    await write(term, '\x1b[31mABC');
    const previous = console.warn; let count = 0;
    console.warn = (...args) => { previous(...args); count++; term.reset(); };
    try {
      const range = { start: 0, get end() { term.resize(11, 4); return 0; } };
      return { ansi: addon.serialize({ range, excludeAltBuffer: true, excludeModes: true }), count };
    } finally { console.warn = previous; }
  });
  await run('ansi-warning-nested-serialization', async (term, addon) => {
    await write(term, '\x1b[34mABC');
    const previous = console.warn; let nested;
    console.warn = (...args) => {
      previous(...args);
      nested = addon.serialize({ range: { start: 0, end: 0 }, excludeAltBuffer: true, excludeModes: true });
      term.reset();
    };
    try {
      const range = { start: 0, get end() { term.resize(11, 4); return 0; } };
      return { ansi: addon.serialize({ range, excludeAltBuffer: true, excludeModes: true }), nested };
    } finally { console.warn = previous; }
  });
  for (const format of ['ansi', 'html']) {
    await run(`${format}-warning-keeps-replaced-line`, async (term, addon) => {
      await write(term, 'ABCDEFGHIJKL');
      let count = 0;
      const previous = console.warn;
      console.warn = (...args) => {
        previous(...args);
        if (++count === 1) { term.reset(); term.resize(12, 4); }
      };
      try {
        let serialized;
        if (format === 'ansi') {
          const range = { start: 0, get end() { term.resize(6, 4); return 0; } };
          serialized = addon.serialize({ range, excludeAltBuffer: true, excludeModes: true });
        } else {
          Object.defineProperty(term.options, 'fontFamily', { configurable: true,
            get() { term.resize(6, 4); return 'monospace'; } });
          serialized = addon.serializeAsHTML({ range: { startLine: 0, endLine: 0, startCol: 0 } });
        }
        return { serialized, count };
      } finally { console.warn = previous; }
    });
  }
  await run('ansi-warning-exception-identity', async (term, addon) => {
    await write(term, '\x1b[31mABC');
    const sentinel = {}, previous = console.warn; let same = false;
    console.warn = () => { throw sentinel; };
    try {
      addon.serialize({ range: { start: 0, get end() { term.resize(11, 4); return 0; } } });
    } catch (error) { same = error === sentinel; }
    finally { console.warn = previous; }
    return same;
  });
  await run('html-selection-range', async (term, addon) => {
    await write(term, 'abc<&\r\nsecond\r\nthird');
    const values = [addon.serializeAsHTML({ onlySelection: true }), addon.serializeAsHTML()];
    term.select(2, 0, 15);
    values.push(addon.serializeAsHTML({ onlySelection: true }),
      addon.serializeAsHTML({ onlySelection: true, range: { startLine: 1, endLine: 1, startCol: 2 } }),
      addon.serializeAsHTML({ range: { startLine: 0, endLine: 1 } }));
    term.clearSelection(); values.push(addon.serializeAsHTML({ onlySelection: true }));
    return values;
  }, true);
  await run('html-live-palette', async (term, addon) => {
    term.options.theme = { red: '#123456', brightBlue: '#abcdef', foreground: '#102030', background: '#405060' };
    await write(term, '\x1b[31mred\x1b[94mblue\x1b[38;5;128mextended\x1b[4:3;58:2::8:9:10mcurly');
    const values = [addon.serializeAsHTML(), addon.serializeAsHTML({ includeGlobalBackground: true })];
    await write(term, '\x1b]4;1;rgb:22/33/44\x1b\\'); values.push(addon.serializeAsHTML());
    term.options.theme = {}; values.push(addon.serializeAsHTML());
    return values;
  }, true);
  await run('html-nan-row-bounds', async (term, addon) => {
    await write(term, 'must not be traversed');
    const values = [];
    const log = [];
    Object.defineProperty(term.options, 'fontFamily', { configurable: true,
      get() { log.push('fontFamily'); return 'monospace'; } });
    for (const [start, end] of [[NaN, 0], [0, NaN], [NaN, NaN], [NaN, -Infinity], [Infinity, NaN]]) {
      const range = {
        get startCol() { log.push('startCol'); return Symbol('unused column'); },
        get startLine() { log.push('startLine'); return start; },
        get endLine() { log.push('endLine'); return end; }
      };
      log.length = 0;
      values.push({ html: capture(() => addon.serializeAsHTML({ range })), log: [...log] });
    }
    return values;
  });
  await run('html-nan-header-exception-identity', async (term, addon) => {
    const sentinel = {}, log = [];
    Object.defineProperty(term.options, 'fontFamily', { configurable: true,
      get() { log.push('fontFamily'); throw sentinel; } });
    let same = false;
    try { addon.serializeAsHTML({ range: { startLine: NaN, endLine: 0, startCol: 0 } }); }
    catch (error) { same = error === sentinel; }
    return { same, log };
  });
  await run('html-header-reentry-reset', async (term, addon) => {
    await write(term, 'content'); const log = [];
    Object.defineProperty(term.options, 'fontFamily', { configurable: true, get() { log.push('fontFamily'); term.reset(); return 'raw\ud800'; } });
    const options = { get onlySelection() { log.push('onlySelection'); return false; }, get range() { log.push('range'); return undefined; },
      get scrollback() { log.push('scrollback'); return undefined; }, get includeGlobalBackground() { log.push('background'); return true; } };
    return { html: addon.serializeAsHTML(options), log };
  });
  await run('html-nonfinite-start-column', async (term, addon) => {
    await write(term, '\x1b[31mABC\r\n\x1b[32mDEF\r\nGHI');
    const log = [];
    Object.defineProperty(term.options, 'fontFamily', { configurable: true,
      get() { log.push('fontFamily'); return 'monospace'; } });
    return [NaN, Infinity].flatMap(startCol => [0, 1, 2].map(endLine => {
      log.length = 0;
      const range = { get startCol() { log.push('startCol'); return startCol; },
        get startLine() { log.push('startLine'); return 0; },
        get endLine() { log.push('endLine'); return endLine; } };
      return { startCol: String(startCol), endLine,
        html: addon.serializeAsHTML({ range }), log: [...log] };
    }));
  });
  await run('html-nonfinite-column-header-throw', async (term, addon) => {
    await write(term, 'content');
    const sentinel = {}, log = [];
    Object.defineProperty(term.options, 'fontFamily', { configurable: true,
      get() { log.push('fontFamily'); throw sentinel; } });
    return [NaN, Infinity].map(startCol => {
      log.length = 0; let same = false;
      try { addon.serializeAsHTML({ range: { startCol, startLine: 0, endLine: 1 } }); }
      catch (error) { same = error === sentinel; }
      return { startCol: String(startCol), same, log: [...log] };
    });
  });
  await run('html-header-coercion-order', async (term, addon) => {
    await write(term, 'abc'); const log = [];
    const raw = key => ({ [Symbol.toPrimitive](hint) { log.push(`${key}:${hint}`); return key + '\ud800'; } });
    const theme = { get foreground() { log.push('foreground'); return raw('fg'); }, get background() { log.push('background'); return raw('bg'); } };
    Object.defineProperty(term.options, 'theme', { configurable: true, get() { log.push('theme'); return theme; } });
    Object.defineProperty(term.options, 'fontFamily', { configurable: true, get() { log.push('fontFamily'); return raw('family'); } });
    Object.defineProperty(term.options, 'fontSize', { configurable: true, get() { log.push('fontSize'); return raw('size'); } });
    return { html: addon.serializeAsHTML({ includeGlobalBackground: true }), log };
  });
  await run('html-header-resize-warnings', async (term, addon) => {
    await write(term, 'abc');
    Object.defineProperty(term.options, 'fontFamily', { configurable: true, get() { term.resize(6, 4); return 'monospace'; } });
    return addon.serializeAsHTML({ range: { startLine: 0, endLine: 0, startCol: 0 } });
  });
  await run('html-header-alt-view', async (term, addon) => {
    await write(term, 'normal');
    Object.defineProperty(term.options, 'fontFamily', { configurable: true, get() { term.write('\x1b[?1049hALT'); term.resize(10, 4); return 'monospace'; } });
    return addon.serializeAsHTML({ range: { startLine: 0, endLine: 0, startCol: 1 } });
  });
  await run('html-warning-exception-identity', async (term, addon) => {
    await write(term, 'abc');
    Object.defineProperty(term.options, 'fontFamily', { configurable: true, get() { term.resize(6, 4); return 'monospace'; } });
    const sentinel = {}, previous = console.warn; let same = false;
    console.warn = () => { throw sentinel; };
    try { addon.serializeAsHTML({ range: { startLine: 0, endLine: 0, startCol: 0 } }); }
    catch (error) { same = error === sentinel; }
    finally { console.warn = previous; }
    return same;
  });
  await run('html-nested-and-reactivated', async (term, addon) => {
    await write(term, 'original'); const other = new Terminal({ cols: 12, rows: 4, scrollback: 20 });
    await write(other, 'other'); let nested, entered = false;
    Object.defineProperty(term.options, 'fontFamily', { configurable: true, get() {
      if (!entered) { entered = true; nested = addon.serialize({ range: { start: 0, end: 0 } }); addon.activate(other); }
      return 'monospace';
    } });
    try { return [addon.serializeAsHTML(), nested, addon.serialize()]; } finally { other.dispose(); }
  });
  await run('before-after-open-palette', async (term, addon, parent) => {
    term.options.theme = { red: '#123456' }; await write(term, '\x1b[31mred');
    const before = addon.serializeAsHTML(); term.open(parent); return [before, addon.serializeAsHTML()];
  });
  await run('dispose-and-reactivate', async (term, addon) => {
    await write(term, 'before'); addon.dispose(); addon.dispose(); const before = addon.serialize();
    const second = new Terminal({ cols: 12, rows: 4, scrollback: 20 });
    try { addon.activate(second); await write(second, 'second'); const after = addon.serialize(); second.dispose(); return [before, after, addon.serialize()]; }
    finally { second.dispose(); }
  });
  // Retain known unsupported numeric results. The comparison report separates
  // these from the supported binding regressions instead of hiding differences.
  await run('raw-numeric-boundaries', async (term, addon) => {
    await write(term, 'abc');
    const numbers = [-0, -1, 0.5, NaN, Infinity, -Infinity, 2 ** 32, Number.MAX_SAFE_INTEGER];
    return numbers.map(number => ({ number: String(number),
      // Equal infinite start/end never advances in the source row++ loop. Use
      // non-looping bounds here and retain that unrun hang separately.
      equalInfiniteBounds: Number.isFinite(number) || Number.isNaN(number) ? 'not applicable' : 'unrun: source loop does not advance',
      ansi: capture(() => addon.serialize({ range: { start: number, end: Number.isFinite(number) || Number.isNaN(number) ? number : 0 } })),
      html: capture(() => addon.serializeAsHTML({ range: { startLine: number,
        endLine: Number.isFinite(number) || Number.isNaN(number) ? number : NaN, startCol: 0 } })) }));
  });
  let seed = 0x5e21a11;
  const random = () => { seed ^= seed << 13; seed ^= seed >>> 17; seed ^= seed << 5; return seed >>> 0; };
  for (let index = 0; index < 32; index++) {
    const payload = [];
    for (let op = 0; op < 12; op++) payload.push(['a', '中', 'e\u0301', '\r\n', '\x1b[31m', '\x1b[0m', '\x1b[2C', '\x1b[1D', '\x1b[4m'][random() % 9]);
    await run(`seeded-${index}`, async (term, addon) => {
      for (const chunk of payload) await write(term, chunk);
      const ansi = addon.serialize(), html = addon.serializeAsHTML();
      const restored = new Terminal({ cols: 12, rows: 4, scrollback: 20 });
      try { await write(restored, ansi); return { payload, ansi, html, original: snapshot(term), restored: snapshot(restored) }; }
      finally { restored.dispose(); }
    });
  }
  return output;
}
