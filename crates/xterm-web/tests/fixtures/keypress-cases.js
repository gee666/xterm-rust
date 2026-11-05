// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
// Source functions are supplied verbatim and checked against the reference.
export function runKeypressCase(terminal, bodies, kind, route, modifier, behavior) {
  const marker = behavior === 'throw-undefined' ? undefined : { marker: true };
  const expected = execute(false);
  const actual = execute(true);
  const a = JSON.stringify(actual), e = JSON.stringify(expected);
  if (a !== e) throw new Error(`${kind}/${route}/${modifier}/${behavior}\nactual ${a}\nsource ${e}`);
  return actual;

  function execute(rust) {
    const trace = [];
    let nested = false, activeEvent;
    const method = new Function('ev', bodies[0]);
    const third = new Function('browser,ev', bodies[1]);
    const input = new Function('ev', bodies[2]);
    function thrown(error) {
      trace.push(['error', Object.is(error, marker) ? 'marker' : error?.name ?? typeof error]);
    }
    function data(text) {
      trace.push(['data', text]);
      if (behavior === 'data-error') throw marker;
    }
    function key(event) {
      trace.push(['key', event.key, event.domEvent === activeEvent]);
      if (behavior === 'key-error') throw marker;
      if (behavior === 'key-nested' && !nested) { nested = true; dispatchInput('nested'); }
    }
    const receiver = {
      _keyDownHandled: false, _keyPressHandled: false, _keyDownSeen: false, _unprocessedDeadKey: false,
      browser: {
        isMac: ['Macintosh', 'MacIntel', 'MacPPC', 'Mac68K'].includes(navigator.platform),
        isWindows: ['Windows', 'Win16', 'Win32', 'WinCE'].includes(navigator.platform)
      },
      options: { macOptionIsMeta: false }, optionsService: { rawOptions: { screenReaderMode: false } },
      _onKey: { fire: key }, _showCursor() {}, coreService: { triggerDataEvent: data },
      _isThirdLevelShift: third
    };
    function dispatchInput(text) {
      const event = new InputEvent('input', { bubbles: true, inputType: 'insertText', data: text });
      for (const [name, value] of [['data', text], ['inputType', 'insertText'], ['composed', false]]) {
        Object.defineProperty(event, name, { get() { trace.push(`input:${name}`); return value; } });
      }
      if (rust) terminal.textarea.dispatchEvent(event);
      else { try { input.call(receiver, event); } catch (error) { thrown(error); } }
    }
    function value() {
      switch (kind) {
        case 'undefined': return undefined;
        case 'null': return null;
        case 'false': return false;
        case 'true': return true;
        case 'zero': return 0;
        case 'negative-zero': return -0;
        case 'nan': return NaN;
        case 'empty': return '';
        case 'string-zero': return '0';
        case 'string-number': return '65';
        case 'string-surrogate-number': return '55296';
        case 'whitespace': return ' ';
        case 'surrogate': return '\ud800';
        case 'symbol': return Symbol('code');
        case 'bigint': return 65n;
        case 'array': return [65];
        case 'function': return function rawCode() {};
        case 'boxed': return new Number(65);
        case 'infinity': return Infinity;
        case 'object': return { [Symbol.toPrimitive](hint) {
          trace.push(['convert', hint]);
          if (behavior === 'conversion-error' || behavior === 'throw-undefined') throw marker;
          if (behavior === 'conversion-nested' && !nested) { nested = true; dispatchInput('nested'); }
          if (behavior === 'conversion-change') Object.defineProperty(activeEvent, 'charCode', { value: 0, configurable: true });
          return 65;
        }};
        default: throw new Error(`unknown kind ${kind}`);
      }
    }
    const payload = value();
    const event = new KeyboardEvent('keypress', { bubbles: true, cancelable: true });
    activeEvent = event;
    let charReads = 0;
    for (const name of ['charCode', 'which', 'keyCode', 'altKey', 'ctrlKey', 'metaKey', 'type']) {
      Object.defineProperty(event, name, { configurable: true, get() {
        trace.push(name);
        if (behavior === 'payload-getter-error' && name === 'charCode' && ++charReads === 2) throw marker;
        if (name === 'charCode') {
          if (behavior !== 'payload-getter-error') ++charReads;
          if (route === 'char') return payload;
          if (route === 'changed-char') return charReads === 1 ? 1 : payload;
          return route === 'which' ? '' : 0;
        }
        if (name === 'which') return route === 'which' ? payload : route === 'fallback' ? null : 0;
        if (name === 'keyCode') return route === 'fallback' ? payload : 65;
        if (name === 'type') return 'keypress';
        return name === `${modifier}Key`;
      }});
    }
    Object.defineProperty(event, 'getModifierState', { get() {
      trace.push('getModifierState');
      return function(name) { trace.push(['modifierState', this === event, name]); return false; };
    }});
    const trap = event => { thrown(event.error); event.preventDefault(); event.stopImmediatePropagation(); };
    let onKey, onData;
    try {
      if (rust) {
        onKey = terminal.onKey(key);
        onData = terminal.onData(data);
        window.addEventListener('error', trap, true);
        terminal.textarea.dispatchEvent(event);
      } else {
        try { method.call(receiver, event); } catch (error) { thrown(error); }
      }
      // Successful keypress suppresses the duplicate InputEvent. Conversion or
      // callback errors leave that flag false, making the probe observable.
      dispatchInput('probe');
      return trace;
    } finally {
      if (rust) {
        window.removeEventListener('error', trap, true);
        onKey?.dispose();
        onData?.dispose();
      }
    }
  }
}
