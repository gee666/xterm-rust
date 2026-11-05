// Shared directed program for original and Rust-backed browser Terminals.
return (async () => {
  const write = value => new Promise(resolve => terminal.write(value, resolve));
  const core = terminal._core;
  const service = core.coreService;
  const buffers = core._bufferService;
  const initialModes = service.decPrivateModes;
  const initialBuffer = buffers.buffer;
  const trace = [];
  const readModes = modes => ({
    applicationCursorKeys: modes.applicationCursorKeys,
    applicationKeypad: modes.applicationKeypad,
    bracketedPasteMode: modes.bracketedPasteMode,
    colorSchemeUpdates: modes.colorSchemeUpdates,
    cursorStyle: modes.cursorStyle ?? null,
    cursorBlink: modes.cursorBlink ?? null,
    origin: modes.origin,
    reverseWraparound: modes.reverseWraparound,
    sendFocus: modes.sendFocus,
    synchronizedOutput: modes.synchronizedOutput,
    win32InputMode: modes.win32InputMode,
    wraparound: modes.wraparound
  });
  const readBuffer = buffer => ({ x: buffer.x, y: buffer.y,
    ybase: buffer.ybase, ydisp: buffer.ydisp,
    scrollTop: buffer.scrollTop, scrollBottom: buffer.scrollBottom });
  const capture = step => trace.push({ step,
    stableCore: core === terminal._core,
    stableService: service === terminal._core.coreService,
    stableBufferService: buffers === terminal._core._bufferService,
    sameModes: initialModes === service.decPrivateModes,
    sameBuffer: initialBuffer === buffers.buffer,
    stableCurrentModes: service.decPrivateModes === service.decPrivateModes,
    stableCurrentBuffer: buffers.buffer === buffers.buffer,
    cols: buffers.cols, rows: buffers.rows,
    modes: readModes(service.decPrivateModes),
    capturedModes: readModes(initialModes),
    buffer: readBuffer(buffers.buffer), capturedBuffer: readBuffer(initialBuffer)
  });
  capture('initial');
  for (const style of [1, 2, 3, 4, 5, 6, 0]) {
    await write(`\x1b[${style} q`);
    capture(`style${style}`);
  }
  await write('\x1b[?1h\x1b=\x1b[?2004h\x1b[?6h\x1b[?45h\x1b[?1004h\x1b[?9001h\x1b[?7l\x1b[6 q');
  capture('modes');
  await write('\x1b[?6l\x1b[2;4r\x1b[3;5H');
  capture('region');
  await write('\x1b[?1049h');
  capture('alternate');
  await write('\x1b[2;3r\x1b[2;2H');
  capture('alternate-region');
  await write('\x1b[?1049l');
  capture('normal');
  terminal.resize(12, 6);
  capture('resize');
  terminal.reset();
  capture('reset');
  await write('\x1b[3 q\x1b[2;5r\x1b[2;3H');
  capture('after-reset');
  return trace;
})();
