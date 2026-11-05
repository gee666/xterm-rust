// Shared cases run against original Terminal and the actual Rust browser ABI.
export function initialMouseModes(term) {
  const core = term._core;
  const mouse = core.mouseStateService;
  const result = [core === term._core, mouse === term._core.mouseStateService,
    mouse.activeProtocol, mouse.activeEncoding, mouse.areMouseEventsActive,
    mouse.isDefaultEncoding, mouse.isPixelEncoding];
  term.dispose();
  return result;
}

export function mouseModeSetters(term) {
  const mouse = term._core.mouseStateService;
  const seen = [];
  const read = () => [mouse.activeProtocol, mouse.activeEncoding,
    mouse.areMouseEventsActive, mouse.isDefaultEncoding, mouse.isPixelEncoding];
  const listener = mouse.onProtocolChange(events => seen.push([events, read()]));
  mouse.activeProtocol = 'ANY';
  mouse.activeProtocol = 'ANY';
  mouse.activeEncoding = 'SGR_PIXELS';
  const pixel = read();
  const errors = [];
  for (const [key, value] of [['activeProtocol', 'missing-protocol'], ['activeEncoding', 'missing-encoding']]) {
    try { mouse[key] = value; } catch (error) { errors.push([error.constructor.name, error.message]); }
  }
  const unchanged = read();
  mouse.reset();
  listener.dispose();
  mouse.activeProtocol = 'VT200';
  const final = read();
  mouse.reset();
  term.dispose();
  return {seen, pixel, errors, unchanged, final};
}

export function mouseModeReentry(term) {
  const mouse = term._core.mouseStateService;
  const seen = [];
  const first = mouse.onProtocolChange(events => {
    seen.push(['first', events, mouse.activeProtocol]);
    if (mouse.activeProtocol === 'ANY') mouse.activeProtocol = 'VT200';
  });
  const second = mouse.onProtocolChange(events => seen.push(['second', events, mouse.activeProtocol]));
  mouse.activeProtocol = 'ANY';
  first.dispose(); second.dispose();
  const sentinel = {thrown: 'protocol listener'};
  const throwing = mouse.onProtocolChange(() => { throw sentinel; });
  let sameError = false;
  try { mouse.activeProtocol = 'X10'; } catch (error) { sameError = error === sentinel; }
  throwing.dispose();
  const committed = mouse.activeProtocol;
  mouse.reset();
  term.dispose();
  return {seen, sameError, committed};
}

export async function parsedMouseModes(term) {
  const mouse = term._core.mouseStateService;
  const write = value => new Promise(resolve => term.write(value, resolve));
  const states = [];
  for (const sequence of ['\x1b[?9h', '\x1b[?1000h', '\x1b[?1002h', '\x1b[?1003h',
    '\x1b[?1006h', '\x1b[?1016h', '\x1b[?1006l', '\x1b[?1016l', '\x1b[?1003l']) {
    await write(sequence);
    states.push([mouse.activeProtocol, mouse.activeEncoding]);
  }
  await write('\x1b[?1003h\x1b[?1006h');
  term.reset();
  const reset = [mouse === term._core.mouseStateService, mouse.activeProtocol, mouse.activeEncoding];
  term.dispose();
  return {states, reset, afterDispose: [mouse.activeProtocol, mouse.activeEncoding]};
}

export async function mouseDomRouting(term) {
  const container = document.createElement('div');
  document.body.append(container);
  try {
    term.open(container);
    await new Promise(resolve => requestAnimationFrame(resolve));
    const screen = container.querySelector('.xterm-screen');
    const mouse = term._core.mouseStateService;
    const output = [];
    const data = term.onData(value => output.push(['data', value]));
    const binary = term.onBinary(value => output.push(['binary', value]));
    const rect = screen.getBoundingClientRect();
    const event = type => new MouseEvent(type, {bubbles:true, button:0,
      buttons:type==='mousedown'?1:0,
      clientX:rect.left+rect.width/term.cols/2, clientY:rect.top+rect.height/term.rows/2});
    mouse.activeProtocol = 'VT200';
    mouse.activeEncoding = 'SGR';
    screen.dispatchEvent(event('mousedown'));
    document.dispatchEvent(event('mouseup'));
    const activeOutput = output.slice();
    mouse.reset();
    screen.dispatchEvent(event('mousedown'));
    document.dispatchEvent(event('mouseup'));
    data.dispose(); binary.dispose();
    return {activeOutput, afterReset:output};
  } finally { term.dispose(); container.remove(); }
}
