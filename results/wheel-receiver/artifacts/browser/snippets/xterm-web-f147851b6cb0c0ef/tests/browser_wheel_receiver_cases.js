// Run these operations unchanged against source Terminal and Rust WASM.
function open(term) {
  const container = document.createElement('div');
  document.body.append(container);
  term.open(container);
  return {container, screen: container.querySelector('.xterm-screen')};
}
function wheel(screen) {
  const rect = screen.getBoundingClientRect();
  const event = new WheelEvent('wheel', {bubbles:true, cancelable:true,
    deltaY:1, deltaMode:WheelEvent.DOM_DELTA_LINE,
    clientX:rect.left + 4, clientY:rect.top + 4});
  event.extra = {identity:'wheel'};
  return event;
}

export function liveWheelReceiver(term) {
  const {container, screen} = open(term);
  const mouse = term._core.mouseStateService;
  const reads = [];
  let event;
  const handler = function(value) {
    reads.push([this === mouse, value === event, value.extra === event.extra,
      this.activeProtocol, this.activeEncoding, this.areMouseEventsActive,
      this.isDefaultEncoding, this.isPixelEncoding, this.extra ?? null]);
    this.extra = 'retained';
    this.reset();
    return false;
  };
  Object.defineProperty(handler, 'call', {get() { throw new Error('must not read call'); }});
  try {
    term.attachCustomWheelEventHandler(handler);
    // The active protocol takes the tracked-wheel path; NONE takes the fallback.
    mouse.activeProtocol = 'VT200';
    mouse.activeEncoding = 'SGR';
    event = wheel(screen);
    screen.dispatchEvent(event);
    const firstPrevented = event.defaultPrevented;
    event = wheel(screen);
    screen.dispatchEvent(event);
    return {reads, firstPrevented, secondPrevented:event.defaultPrevented,
      retained:mouse.extra, final:[mouse.activeProtocol, mouse.activeEncoding]};
  } finally { term.dispose(); container.remove(); }
}

export function wheelReentryAndReplacement(term) {
  const {container, screen} = open(term);
  const mouse = term._core.mouseStateService;
  const seen = [];
  let outer;
  const replacement = function(event) {
    seen.push(['inner', this === mouse, event !== outer, event.extra.identity,
      this.activeProtocol, this.extra]);
    return false;
  };
  const handler = function(event) {
    seen.push(['outer', this === mouse, event === outer, this.activeProtocol]);
    this.extra = 'from-outer';
    term.attachCustomWheelEventHandler(replacement);
    this.activeProtocol = 'VT200';
    screen.dispatchEvent(wheel(screen));
    seen.push(['resume', this === mouse, this.activeProtocol, this.extra]);
    return false;
  };
  try {
    term.attachCustomWheelEventHandler(handler);
    outer = wheel(screen);
    screen.dispatchEvent(outer);
    screen.dispatchEvent(wheel(screen));
    return seen;
  } finally { term.dispose(); container.remove(); }
}

export function wheelResultsAndThrows(term) {
  const {container, screen} = open(term);
  const mouse = term._core.mouseStateService;
  const results = [false, undefined, null, 0, -0, NaN, '', Symbol('result'),
    0n, {}, [], new Boolean(false)];
  const errors = [undefined, null, false, 0, NaN, Symbol('throw'), 0n, {}];
  const seen = [];
  let expected;
  const reported = [];
  const trap = event => {
    reported.push(Object.is(event.error, expected));
    event.preventDefault();
  };
  window.addEventListener('error', trap);
  try {
    // Line units avoid a partial pixel accumulator masking an allowed event.
    // Only literal false vetoes; every other return must produce one report.
    const output = [];
    const listener = term.onData(value => output.push(value));
    mouse.activeProtocol = 'VT200';
    mouse.activeEncoding = 'SGR';
    for (const value of results) {
      const before = output.length;
      term.attachCustomWheelEventHandler(function() {
        seen.push(this === mouse);
        return value;
      });
      screen.dispatchEvent(wheel(screen));
      const count = output.length - before;
      if (count !== (value === false ? 0 : 1)) throw new Error('wrong strict-false wheel veto');
      seen.push(count);
    }
    listener.dispose();
    for (const value of errors) {
      expected = value;
      term.attachCustomWheelEventHandler(function(event) {
        this.extra = event.extra;
        throw value;
      });
      const event = wheel(screen);
      screen.dispatchEvent(event);
      seen.push(mouse.extra === event.extra);
    }
    return {seen, reported};
  } finally {
    window.removeEventListener('error', trap);
    term.dispose(); container.remove();
  }
}
