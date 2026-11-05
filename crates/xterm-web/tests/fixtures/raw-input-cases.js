// Copyright (c) 2019-2026 The xterm.js authors. MIT license.
// Both runs use the same decorated DOM events and assertions. Source method
// bodies are supplied verbatim, with a separate integrity test.
export function runRawInputCase(terminal, trace, bodies, kind, behavior, level, disabled) {
  const marker = { marker: true };
  const levels = { trace: 0, debug: 1, info: 2, warn: 3, error: 4, off: 5 };
  const makeMethod = (args, body) => new Function('LogLevelEnum', 'LOG_PREFIX', `return function(${args}) {${body}\n}`)(
    { TRACE: 0, DEBUG: 1 }, 'xterm.js: ');
  const expected = execute(false);
  const actual = execute(true);
  const a = JSON.stringify(actual), e = JSON.stringify(expected);
  if (a !== e) throw new Error(`${kind}/${behavior}/${level}/${disabled}\nactual ${a}\nsource ${e}`);
  return actual;

  function execute(rust) {
    const output = rust ? trace : [];
    output.length = 0;
    let sent = 0, nested = false, currentLevel = level;
    const options = { disableStdin: disabled, scrollOnUserInput: true, screenReaderMode: false };
    let receiver;
    const sourceLog = {
      _logLevel: levels[level], _optionsService: { options },
      _evalLazyOptionalParams: makeMethod('optionalParams', bodies[2]),
      _log: makeMethod('type,message,optionalParams', bodies[3]),
      trace: makeMethod('message,...optionalParams', bodies[4]),
      debug: makeMethod('message,...optionalParams', bodies[5])
    };
    const logger = {};
    for (const name of ['trace', 'debug', 'info', 'warn', 'error']) {
      Object.defineProperty(logger, name, { get() {
        output.push(`logger:${name}`);
        if (behavior === 'logger-getter-throw' && name === 'trace') throw marker;
        return (message, ...params) => {
          output.push([name, message, params]);
          if (behavior === 'debug-enables-trace' && name === 'debug') {
            currentLevel = 'trace';
            if (rust) terminal.options = { logLevel: currentLevel };
            else sourceLog._logLevel = levels[currentLevel];
          }
        };
      }});
    }
    options.logger = logger;
    const emit = data => {
      output.push(['data', Object.is(data, payload), typeof data]);
      ++sent;
      if (behavior === 'data-throw') throw marker;
      if (behavior === 'data-nested' && !nested) { nested = true; dispatch('nested'); }
      if (behavior === 'dispose') {
        if (rust) terminal.dispose();
        else disposed = true;
        dispatch('after-dispose');
      }
    };
    let disposed = false;
    receiver = {
      _keyDownSeen: false, _keyPressHandled: false, _unprocessedDeadKey: true,
      optionsService: { rawOptions: options },
      coreService: {
        _optionsService: { rawOptions: options },
        _bufferService: { buffer: { ybase: 1, ydisp: 0 } },
        _onRequestScrollToBottom: { fire() { output.push('scroll'); this.owner._bufferService.buffer.ydisp = 1; } },
        _onUserInput: { fire() { output.push('user'); } },
        _logService: sourceLog,
        _onData: { fire(data) { if (!disposed) emit(data); } },
        triggerDataEvent: makeMethod('data,wasUserInput=false', bodies[1])
      }
    };
    receiver.coreService._onRequestScrollToBottom.owner = receiver.coreService;
    const input = makeMethod('ev', bodies[0]);
    let payload;
    const primitives = { null: null, undefined: undefined, false: false, zero: 0,
      nan: NaN, negativeZero: -0, bigint: 42n, symbol: Symbol('input'),
      array: [1, 2], function: function rawData() {} };
    if (kind === 'object') {
      payload = {
        [Symbol.toPrimitive](hint) {
          output.push(`primitive:${hint}`);
          if (behavior === 'coercion-throw') throw marker;
          if (behavior === 'coercion-nested' && !nested) { nested = true; dispatch('nested'); }
          return 'object-text';
        },
        get split() {
          output.push('split:get');
          if (behavior === 'split-throw') throw marker;
          return function(separator) {
            output.push(['split:call', this === payload, separator]);
            return { get map() {
              output.push('map:get');
              return function(callback) {
                output.push(['map:call', this !== payload]);
                const character = { get charCodeAt() {
                  output.push('charCodeAt:get');
                  return function(index) {
                    output.push(['charCodeAt:call', this === character, index]);
                    return 987;
                  };
                }};
                return [callback(character), callback('XY')];
              };
            }};
          };
        }
      };
    } else payload = primitives[kind];
    let subscription, thrown;
    const trap = event => { thrown = event.error; event.preventDefault(); event.stopImmediatePropagation(); };
    if (rust) {
      terminal.options = { logger, logLevel: level, disableStdin: disabled };
      subscription = terminal.onData(emit);
      window.addEventListener('error', trap, true);
      output.length = 0;
    }
    function dispatch(value) {
      if (!rust && disposed) return;
      let read = 0;
      const event = new InputEvent('input', { inputType: 'insertText', composed: false, bubbles: true });
      Object.defineProperty(event, 'data', { get() {
        output.push(++read === 1 ? 'guard' : 'payload');
        return read === 1 ? 'admit' : value;
      }});
      if (rust) terminal.textarea.dispatchEvent(event);
      else input.call(receiver, event);
    }
    try {
      try { dispatch(payload); } catch (error) { thrown = error; }
      if (thrown !== undefined) output.push(['error', thrown === marker ? 'marker' : thrown?.name ?? typeof thrown]);
      return { output: [...output], sent };
    } finally {
      if (rust) { subscription.dispose(); window.removeEventListener('error', trap, true); }
    }
  }
}
