// Same program on the unchanged original and real Rust browser Terminal.
return (async () => {
  const write = value => new Promise(resolve => terminal.write(value, resolve));
  const core = terminal._core;
  const input = core._inputHandler;
  const parser = input._parser;
  const dirty = input._dirtyRowTracker;
  const initial = input._curAttrData;
  const trace = [];
  const attr = value => ({ fg: value.fg, bg: value.bg, inverse: value.isInverse(),
    fgMode: value.getFgColorMode(), bgMode: value.getBgColorMode(),
    fgColor: value.getFgColor(), bgColor: value.getBgColor() });
  const row = value => ({ length: value.length, wrapped: value.isWrapped,
    data: Array.from(value._data), extended: Object.fromEntries(
      Object.entries(value._extendedAttrs).map(([key, value]) =>
        [key, { ext: value.ext, urlId: value.urlId }])) });
  const snapshot = step => trace.push({ step, stableInput: core._inputHandler === input,
    stableParser: input._parser === parser, stableDirty: input._dirtyRowTracker === dirty,
    stableCurrentAttrs: input._curAttrData === input.getAttrData(),
    initialAttrs: initial === input._curAttrData, attributes: attr(input._curAttrData),
    captured: attr(initial), aliasBuffer: core.buffer === core._bufferService.buffer,
    first: row(core.buffer.lines.get(0)) });
  const equal = (actual, expected, label) => {
    if (JSON.stringify(actual) !== JSON.stringify(expected)) {
      throw Error(label + ': ' + JSON.stringify(actual) + ' != ' + JSON.stringify(expected));
    }
  };
  const accessors = ['_renderService', '_inputHandler', '_themeService', '_coreBrowserService'];
  equal(accessors.map(key => core[key] != null), [true, true, true, true], 'real service access');
  equal([parser.constructor.name, initial.constructor.name],
    ['EscapeSequenceParser', '_AttributeData'], 'source class names');
  equal(core.buffer.lines.get(0)._data instanceof Uint32Array, true, 'packed type');
  equal(core.buffer.lines.get(0)._extendedAttrs instanceof Object, true, 'extended map');
  snapshot('default');
  await write('\x1b[38;2;18;52;86;48;5;17;7mA');
  snapshot('sgr');
  equal(initial === input._curAttrData, true, 'SGR changes the same object');
  equal(initial.getFgColor(), 0x123456, 'live attributes');
  const capturedRow = core.buffer.lines.get(0);
  await write('B');
  equal(Array.from(capturedRow._data), Array.from(core.buffer.lines.get(0)._data), 'retained row');
  snapshot('retained-row');
  const lf = [];
  const listener = terminal.onLineFeed(() => lf.push([core.buffer.x, core.buffer.y]));
  equal(input.lineFeed(), true, 'LF operation return');
  listener.dispose();
  trace.push({ step: 'linefeed', events: lf, x: core.buffer.x, y: core.buffer.y });
  dirty.clearRange();
  trace.push({ step: 'dirty-clear', range: [dirty.start, dirty.end] });
  dirty.markDirty(3);
  trace.push({ step: 'dirty-row', range: [dirty.start, dirty.end] });
  dirty.markRangeDirty(2, 0);
  trace.push({ step: 'dirty-reversed', range: [dirty.start, dirty.end] });
  dirty.clearRange();
  dirty.markAllDirty();
  trace.push({ step: 'dirty-all', range: [dirty.start, dirty.end] });
  const theme = core._themeService;
  const colors = theme.colors;
  terminal.options.theme = { foreground: '#123456', background: '#234567', red: '#345678' };
  trace.push({ step: 'theme', stable: theme === core._themeService,
    stableColors: colors === theme.colors, foreground: [colors.foreground.css, colors.foreground.rgba],
    background: [colors.background.css, colors.background.rgba],
    red: [colors.ansi[1].css, colors.ansi[1].rgba], ansiLength: colors.ansi.length });
  const browser = core._coreBrowserService;
  trace.push({ step: 'browser', stable: browser === core._coreBrowserService,
    window: browser.window === window, document: browser.mainDocument === document,
    dpr: browser.dpr, focused: browser.isFocused });

  // Read callback payloads at the callback, rather than retaining source parser
  // scratch arrays that are intentionally reused between parser chunks.
  const calls = [];
  const handles = [];
  const stream = (name, first, last) => ({
    [first](params) {
      calls.push([name, first, params ? params.toArray() : null]);
      if (params) equal(Array.from(params.params).slice(0, params.length),
        [1, 3], 'native IParams view');
    },
    put(data, start, end) { calls.push([name, 'put', Array.from(data.subarray(start, end))]); },
    [last](success) { calls.push([name, last, success]); return name === 'new' ? Promise.resolve(true) : true; }
  });
  handles.push(parser.registerDcsHandler({ final: 'z' }, stream('old', 'hook', 'unhook')));
  handles.push(parser.registerDcsHandler({ final: 'z' }, stream('new', 'hook', 'unhook')));
  await write('\x1bP1:2;3z');
  await write('a\u{1f600}');
  await write('\x1b\\');
  for (const handle of handles) handle.dispose();
  trace.push({ step: 'dcs-lifo-async', calls: calls.splice(0) });
  const osc = parser.registerOscHandler(7779, stream('osc', 'start', 'end'));
  await write('\x1b]7779;');
  await write('xy');
  await write('\x07');
  osc.dispose();
  trace.push({ step: 'osc', calls: calls.splice(0) });
  const id = { prefix: '?', final: 'Z' };
  const apc = parser.registerApcHandler(id, stream('apc', 'start', 'end'));
  equal(id.prefix, undefined, 'APC clears prefix');
  await write('\x1b_Z');
  await write('ab');
  await write('\x1b\\');
  apc.dispose();
  trace.push({ step: 'apc', calls: calls.splice(0) });
  const csi = parser.registerCsiHandler({ intermediates: '"', final: 'z' }, params => {
    calls.push(['csi', params.toArray(), params.hasSubParams(0), Array.from(params.getSubParams(0))]);
    return true;
  });
  await write('\x1b[4:5;6"z');
  csi.dispose();
  trace.push({ step: 'csi', calls: calls.splice(0) });
  const beforeReset = attr(initial);
  terminal.reset();
  equal(attr(initial), beforeReset, 'reset preserves the replaced object');
  equal(initial === input._curAttrData, false, 'reset replaces attribute identity');
  snapshot('reset');
  const resetCalls = [];
  const reset = input.onRequestReset(() => resetCalls.push([core.buffer.x, core.buffer.y]));
  await write('x\x1bc');
  reset.dispose();
  trace.push({ step: 'request-reset', calls: resetCalls });
  await write('y\x1bc');
  equal(resetCalls.length, 1, 'reset disposal removes listener');
  await write('\x1b[?1049hZ');
  snapshot('alternate');
  await write('\x1b[?1049l');
  snapshot('normal');
  return trace;
})();
