// Identical operations execute on original Terminal and Rust-backed Terminal.
const slot = '_customWheelEventHandler';
const falsey = [undefined, null, false, 0, -0, NaN, '', 0n];
const returns = [...falsey, true, 1, 'x', {}, [], new Boolean(false), Symbol('return')];
function open(term) {
  const container = document.createElement('div');
  document.body.append(container);
  term.open(container);
  return {container, screen:container.querySelector('.xterm-screen')};
}
function wheel(screen) {
  const rect = screen.getBoundingClientRect();
  return new WheelEvent('wheel', {bubbles:true, cancelable:true,
    deltaY:1, deltaMode:1, clientX:rect.left+4, clientY:rect.top+4});
}
function catches(callback, expected) {
  try { callback(); return false; } catch (error) { return Object.is(error, expected); }
}
function typeError(callback) {
  try { callback(); return false; } catch (error) { return error instanceof TypeError; }
}

export function directWheelValues(term) {
  const mouse = term._core.mouseStateService;
  const event = {extra:Symbol('event')};
  const seen = [];
  for (const value of falsey) {
    seen.push(mouse.setCustomWheelEventHandler(value) === undefined,
      Object.is(mouse[slot], value), mouse.allowCustomWheelEvent(event));
  }
  for (const result of returns) {
    const handler = function(value) {
      seen.push(this === mouse, value === event, arguments.length);
      return result;
    };
    Object.defineProperty(handler, 'call', {get() { throw new Error('own call read'); }});
    mouse.setCustomWheelEventHandler(handler);
    seen.push(mouse.allowCustomWheelEvent(event));
  }
  for (const error of returns) {
    mouse.setCustomWheelEventHandler(function() { throw error; });
    seen.push(catches(() => mouse.allowCustomWheelEvent(event), error));
  }
  for (const value of [true, 1, 'x', {}, [], Symbol('handler'), 1n]) {
    mouse.setCustomWheelEventHandler(value);
    seen.push(typeError(() => mouse.allowCustomWheelEvent(event)));
  }
  mouse.setCustomWheelEventHandler(undefined);
  term.dispose();
  return seen;
}

export function borrowedWheelMethods(term) {
  const mouse = term._core.mouseStateService;
  const set = mouse.setCustomWheelEventHandler;
  const allow = mouse.allowCustomWheelEvent;
  const event = {};
  const seen = [];
  const receiver = {label:'borrowed'};
  const handler = function(value) { seen.push(this === receiver, value === event); return false; };
  seen.push(set.call(receiver, handler) === undefined, receiver[slot] === handler,
    allow.call(receiver, event));
  const error = {kind:'setter'};
  Object.defineProperty(receiver, slot, {configurable:true,
    get() { seen.push('get'); return undefined; },
    set(value) { seen.push(this === receiver, value === handler); throw error; }});
  seen.push(catches(() => set.call(receiver, handler), error), allow.call(receiver, event));
  for (const value of [null, undefined, 1, 'x', false, Symbol('receiver'), 0n]) {
    seen.push(typeError(() => set.call(value, handler)));
    seen.push(value == null ? typeError(() => allow.call(value, event)) : allow.call(value, event));
  }
  seen.push(typeError(() => set.call(Object.freeze({}), handler)));
  const first = {[slot]:handler};
  // A borrowed method uses inherited handler properties too.
  seen.push(allow.call(Object.create(first), event));
  term.dispose();
  return seen;
}

export function wheelAccessorOrder(term) {
  const mouse = term._core.mouseStateService;
  const event = {};
  const seen = [];
  let reads = 0;
  Object.defineProperty(mouse, slot, {configurable:true,
    get() {
      seen.push(['get', ++reads]);
      if (reads === 1) return true;
      return function(value) { seen.push(['call', this === mouse, value === event]); return false; };
    }});
  seen.push(['result', mouse.allowCustomWheelEvent(event)]);
  const error = {kind:'getter'};
  for (const phase of [1, 2]) {
    reads = 0;
    Object.defineProperty(mouse, slot, {configurable:true,
      get() { if (++reads === phase) throw error; return true; }});
    seen.push([phase, catches(() => mouse.allowCustomWheelEvent(event), error), reads]);
  }
  reads = 0;
  Object.defineProperty(mouse, slot, {configurable:true,
    get() { reads++; return 0; }});
  seen.push(['falsey', mouse.allowCustomWheelEvent(event), reads]);
  reads = 0;
  Object.defineProperty(mouse, slot, {configurable:true,
    get() { return ++reads === 1 ? true : undefined; }});
  seen.push(['changed', typeError(() => mouse.allowCustomWheelEvent(event)), reads]);
  delete mouse[slot];
  term.dispose();
  return seen;
}

export function sharedWheelRouting(term) {
  const mouse = term._core.mouseStateService;
  const seen = [];
  const first = function(event) { seen.push(['first', this === mouse, event.type]); return false; };
  term.attachCustomWheelEventHandler(first);
  seen.push(['storedBeforeOpen', mouse[slot] === first, mouse.allowCustomWheelEvent({type:'direct'})]);
  const {container, screen} = open(term);
  try {
    for (const mode of ['NONE', 'VT200']) {
      mouse.activeProtocol = mode;
      mouse.activeEncoding = 'SGR';
      screen.dispatchEvent(wheel(screen));
      mouse.setCustomWheelEventHandler(function(event) {
        seen.push(['service', mode, this === mouse, event.type]); return false;
      });
      screen.dispatchEvent(wheel(screen));
      term.attachCustomWheelEventHandler(first);
    }
    term.attachCustomWheelEventHandler(undefined);
    seen.push(['cleared', mouse[slot] === undefined, mouse.allowCustomWheelEvent({})]);
    mouse.setCustomWheelEventHandler(first);
    mouse.reset();
    seen.push(['reset', mouse[slot] === first, mouse.allowCustomWheelEvent({type:'reset'})]);
    term.dispose();
    seen.push(['disposed', mouse[slot] === first, mouse.allowCustomWheelEvent({type:'disposed'})]);
    mouse.setCustomWheelEventHandler(undefined);
    return seen;
  } finally { term.dispose(); container.remove(); }
}

export function wheelMethodReentry(term) {
  const mouse = term._core.mouseStateService;
  const event = {};
  const seen = [];
  const replacement = function(value) { seen.push(['inner', this === mouse, value === event]); return false; };
  mouse.setCustomWheelEventHandler(function(value) {
    seen.push(['outer', this === mouse, value === event]);
    this.setCustomWheelEventHandler(replacement);
    seen.push(['nestedResult', this.allowCustomWheelEvent(event)]);
    this.reset();
    return true;
  });
  seen.push(['outerResult', mouse.allowCustomWheelEvent(event)]);
  seen.push(['nextResult', mouse.allowCustomWheelEvent(event)]);
  mouse.setCustomWheelEventHandler(undefined);
  term.dispose();
  return seen;
}

export function wheelMethodOverrides(term) {
  const mouse = term._core.mouseStateService;
  const {container, screen} = open(term);
  const seen = [];
  const error = {kind:'override'};
  const handler = () => false;
  try {
    mouse.setCustomWheelEventHandler = function(value) {
      seen.push(['set', this === mouse, value === handler]); return error;
    };
    seen.push(['publicResult', term.attachCustomWheelEventHandler(handler) === undefined]);
    Object.defineProperty(mouse, 'setCustomWheelEventHandler', {configurable:true,
      get() { throw error; }});
    seen.push(['getterError', catches(() => term.attachCustomWheelEventHandler(handler), error)]);
    const reported = [];
    const trap = event => { reported.push(event.error === error); event.preventDefault(); };
    window.addEventListener('error', trap);
    try {
      for (const mode of ['NONE', 'VT200']) {
        mouse.activeProtocol = mode;
        mouse.allowCustomWheelEvent = function(event) {
          seen.push(['allow', mode, this === mouse, event.type]); return false;
        };
        screen.dispatchEvent(wheel(screen));
        Object.defineProperty(mouse, 'allowCustomWheelEvent', {configurable:true,
          get() { throw error; }});
        screen.dispatchEvent(wheel(screen));
        delete mouse.allowCustomWheelEvent;
      }
    } finally { window.removeEventListener('error', trap); }
    return {seen, reported};
  } finally {
    delete mouse.setCustomWheelEventHandler;
    delete mouse.allowCustomWheelEvent;
    term.dispose(); container.remove();
  }
}

export function wheelMethodPrototype(term) {
  const mouse = term._core.mouseStateService;
  const proto = Object.getPrototypeOf(mouse);
  const methods = ['setCustomWheelEventHandler', 'allowCustomWheelEvent'];
  const descriptors = methods.map(name => {
    const descriptor = Object.getOwnPropertyDescriptor(proto, name);
    return [Object.hasOwn(mouse, name), descriptor.enumerable,
      descriptor.writable, descriptor.configurable, descriptor.value.length];
  });
  const allow = mouse.allowCustomWheelEvent;
  const replacement = () => false;
  proto.allowCustomWheelEvent = replacement;
  const peer = new term.constructor();
  try {
    const other = peer._core.mouseStateService;
    return {descriptors, sameSetter:mouse.setCustomWheelEventHandler === other.setCustomWheelEventHandler,
      sharedOverride:other.allowCustomWheelEvent === replacement,
      otherHandler:Object.hasOwn(other, slot) && other[slot] === undefined,
      originalOnBorrowed:allow.call({}, {})};
  } finally { proto.allowCustomWheelEvent = allow; peer.dispose(); term.dispose(); }
}
