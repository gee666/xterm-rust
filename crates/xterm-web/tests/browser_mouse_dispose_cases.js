// Shared source and actual Rust Terminal cases. No JavaScript terminal fallback.
const state = mouse => [mouse.activeProtocol, mouse.activeEncoding,
  mouse.areMouseEventsActive, mouse.isDefaultEncoding, mouse.isPixelEncoding];
const event = () => ({button:0, action:1, col:2, row:3, x:4, y:5,
  ctrl:false, alt:false, shift:false});
const failure = callback => {
  try { callback(); return null; }
  catch (error) { return error instanceof TypeError ? 'TypeError' : String(error); }
};

export function directMouseDisposal(term) {
  const mouse = term._core.mouseStateService;
  const seen = [];
  const subscribe = mouse.onProtocolChange;
  const handle = subscribe(value => seen.push(value));
  mouse.activeProtocol = 'ANY';
  mouse.activeEncoding = 'SGR';
  const before = state(mouse);
  const returns = [mouse.dispose() === undefined, mouse.dispose() === undefined];
  const after = state(mouse);
  const collected = [];
  const late = subscribe(value => seen.push(['late', value]), null, collected);
  mouse.activeProtocol = 'X10';
  mouse.activeEncoding = 'SGR_PIXELS';
  const input = event();
  const callable = [mouse.restrictMouseEvent(input), mouse.encodeMouseEvent(input), state(mouse)];
  mouse.reset();
  const reset = state(mouse);
  handle.dispose(); late.dispose();
  term.dispose();
  mouse.activeProtocol = 'VT200';
  const retained = [mouse === term._core.mouseStateService,
    subscribe === mouse.onProtocolChange, state(mouse), mouse.restrictMouseEvent(event())];
  return {before, returns, after, seen, collected:collected.length, callable, reset, retained};
}

export function singleListenerDisposal(term) {
  const mouse = term._core.mouseStateService;
  const seen = [];
  mouse.activeEncoding = 'SGR';
  const handle = mouse.onProtocolChange(value => {
    seen.push(['enter', value, state(mouse)]);
    mouse.dispose();
    mouse.reset();
    seen.push(['leave', state(mouse)]);
  });
  const error = failure(() => { mouse.activeProtocol = 'ANY'; });
  mouse.activeProtocol = 'X10';
  handle.dispose();
  const after = state(mouse);
  term.dispose();
  return {seen, error, after};
}

export function multipleListenerDisposal(term) {
  const mouse = term._core.mouseStateService;
  const seen = [];
  mouse.activeProtocol = 'ANY';
  mouse.activeEncoding = 'SGR';
  const first = mouse.onProtocolChange(value => {
    seen.push(['first', value, state(mouse)]);
    mouse.dispose();
  });
  const second = mouse.onProtocolChange(value => seen.push(['second', value]));
  // Reset commits NONE before dispatch, but cannot reset the encoding if the
  // active listener array is cleared before its second entry is visited.
  const error = failure(() => mouse.reset());
  const interrupted = state(mouse);
  mouse.reset();
  const after = state(mouse);
  first.dispose(); second.dispose();
  term.dispose();
  return {seen, error, interrupted, after};
}

function mutateBeforeDisposal(term, add) {
  const mouse = term._core.mouseStateService;
  const seen = [];
  let second;
  let added;
  const first = mouse.onProtocolChange(value => {
    seen.push(['first', value]);
    if (add) added = mouse.onProtocolChange(value => seen.push(['added', value]));
    else second.dispose();
    mouse.dispose();
  });
  second = mouse.onProtocolChange(value => seen.push(['second', value]));
  const error = failure(() => { mouse.activeProtocol = 'DRAG'; });
  mouse.activeProtocol = 'ANY';
  first.dispose(); second.dispose(); added?.dispose();
  const after = state(mouse);
  term.dispose();
  return {seen, error, after};
}

export function removedListenerDisposal(term) { return mutateBeforeDisposal(term, false); }
export function addedListenerDisposal(term) { return mutateBeforeDisposal(term, true); }

export function throwingListenerDisposal(term) {
  const mouse = term._core.mouseStateService;
  const sentinel = {source:'dispose then throw'};
  const seen = [];
  const first = mouse.onProtocolChange(value => {
    seen.push(value);
    mouse.dispose();
    throw sentinel;
  });
  const second = mouse.onProtocolChange(value => seen.push(['second', value]));
  let same = false;
  try { mouse.activeProtocol = 'X10'; } catch (error) { same = error === sentinel; }
  const committed = state(mouse);
  mouse.reset();
  first.dispose(); second.dispose();
  term.dispose();
  return {same, seen, committed, after:state(mouse)};
}

export async function parsedModesAfterDisposal(term) {
  const mouse = term._core.mouseStateService;
  const seen = [];
  const handle = mouse.onProtocolChange(value => seen.push(value));
  const write = value => new Promise(resolve => term.write(value, resolve));
  mouse.dispose();
  const states = [];
  for (const sequence of ['\x1b[?9h', '\x1b[?1000h', '\x1b[?1002h', '\x1b[?1003h',
    '\x1b[?1006h', '\x1b[?1016h', '\x1b[?1003l']) {
    await write(sequence);
    states.push(state(mouse));
  }
  term.reset();
  const after = state(mouse);
  handle.dispose();
  term.dispose();
  return {seen, states, after};
}

export async function domRoutingAfterDisposal(term) {
  const container = document.createElement('div');
  document.body.append(container);
  try {
    term.open(container);
    await new Promise(resolve => requestAnimationFrame(resolve));
    const mouse = term._core.mouseStateService;
    const screen = container.querySelector('.xterm-screen');
    const rect = screen.getBoundingClientRect();
    const output = [];
    const seen = [];
    const data = term.onData(value => output.push(value));
    const protocol = mouse.onProtocolChange(value => seen.push(value));
    let wheels = 0;
    term.attachCustomWheelEventHandler(() => { wheels++; return false; });
    const click = () => {
      const options = {bubbles:true, button:0,
        clientX:rect.left+rect.width/term.cols/2, clientY:rect.top+rect.height/term.rows/2};
      screen.dispatchEvent(new MouseEvent('mousedown', {...options, buttons:1}));
      document.dispatchEvent(new MouseEvent('mouseup', {...options, buttons:0}));
    };
    mouse.activeProtocol = 'VT200';
    mouse.activeEncoding = 'SGR';
    mouse.dispose();
    click();
    const active = output.slice();
    screen.dispatchEvent(new WheelEvent('wheel', {bubbles:true, deltaY:1, deltaMode:1}));
    mouse.activeProtocol = 'NONE';
    click();
    const inactive = output.slice();
    mouse.activeProtocol = 'VT200';
    click();
    data.dispose(); protocol.dispose();
    return {active, inactive, resumed:output, seen, wheels};
  } finally { term.dispose(); container.remove(); }
}
