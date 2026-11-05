// Copyright (c) 2026 The xterm.js authors. All rights reserved.
// Licensed under MIT. Generated test-only upstream fixture. The cohesive
// source body exceeds 350 lines so its assertions stay auditable.
// Licensed under MIT. Original-protocol reference cases; no Rust result is fabricated.
const prototypeCases = (() => {
  const cases = [];
  const names = ['constructor', 'toString', 'valueOf', 'hasOwnProperty', 'isPrototypeOf',
    'propertyIsEnumerable', 'toLocaleString', '__defineGetter__', '__defineSetter__',
    '__lookupGetter__', '__lookupSetter__', '__proto__'];
  for (const name of names) for (const down of [false, true]) for (const mask of [0, 15]) {
    for (const location of ['code', 'key', 'both']) cases.push({ id: `builtin-${name}-${location}-${Number(down)}-${mask}`,
      kind: 'builtin', name, location, down, mask });
  }
  for (const value of ['undefined', 'null', 'false', 'true', 'zero', 'negative-zero', 'nan',
    'infinity', 'negative-infinity', 'empty', 'text', 'surrogate']) {
    for (const down of [false, true]) for (const mask of [0, 15]) {
      cases.push({ id: `primitive-${value}-${Number(down)}-${mask}`, kind: 'primitive', value, down, mask });
    }
  }
  for (const mask of [0, 15]) for (const kind of ['late-object', 'late-prototype-text', 'string-hint',
    'receiver-mutation', 'second-encoder-isolation', 'symbol-result', 'invalid-result']) {
    cases.push({ id: `${kind}-${mask}`, kind, mask, down: true });
  }
  for (const table of ['vk', 'sc', 'uc']) for (const kind of ['lookup-throw', 'coercion-throw']) {
    cases.push({ id: `${kind}-${table}`, kind, table, mask: 0, down: true });
  }
  return cases;
})();

// Self-contained so Node vm and actual browser realms run identical cases.
function capturePrototypeCase(Encoder, spec) {
  const trace = [];
  const results = [];
  const identities = new Map();
  let nextIdentity = 1;
  const id = value => {
    if (!identities.has(value)) identities.set(value, `value-${nextIdentity++}`);
    return identities.get(value);
  };
  const units = value => Array.from({ length: value.length }, (_, index) => value.charCodeAt(index));
  const describe = value => {
    if (value === undefined) return { kind: 'undefined' };
    if (value === null) return { kind: 'null' };
    if (typeof value === 'number') return { kind: 'number', value: Number.isNaN(value) ? 'NaN'
      : value === Infinity ? 'Infinity' : value === -Infinity ? '-Infinity' : Object.is(value, -0) ? '-0' : value };
    if (typeof value === 'boolean') return { kind: 'boolean', value };
    if (typeof value === 'string') return { kind: 'string', units: units(value) };
    return { kind: typeof value, identity: id(value) };
  };
  const marker = {};
  const encoder = new Encoder();
  const maps = new Map([[encoder._codeToVk, 'vk'], [encoder._codeToScancode, 'sc'], [encoder._keyToControlChar, 'uc']]);
  for (const [map, name] of maps) identities.set(map, `table-${name}`);
  const prototype = Object.getPrototypeOf(encoder._codeToVk);
  const probe = '__xterm_prototype_probe__';
  const savedProbe = Object.getOwnPropertyDescriptor(prototype, probe);
  const savedText = Object.getOwnPropertyDescriptor(prototype, 'toString');
  let phase = 'early';
  const primitive = name => ({ undefined: undefined, null: null, false: false, true: true,
    zero: 0, 'negative-zero': -0, nan: NaN, infinity: Infinity, 'negative-infinity': -Infinity,
    empty: '', text: 'field text', surrogate: '\ud800\udfff' })[name];
  const objects = new Map();
  const rawObject = table => {
    if (objects.has(table)) return objects.get(table);
    const value = spec.kind === 'string-hint' ? {
      toString() { trace.push({ op: 'toString', table }); return `string-${table}`; },
      valueOf() { trace.push({ op: 'valueOf', table }); return `default-${table}`; }
    } : { [Symbol.toPrimitive](hint) {
      trace.push({ op: 'coerce', table, hint, phase, receiver: this === value });
      if (spec.kind === 'coercion-throw' && spec.table === table) throw marker;
      if (spec.kind === 'symbol-result') return Symbol('source symbol');
      if (spec.kind === 'invalid-result') return {};
      return `${phase}-${table}`;
    } };
    objects.set(table, value);
    return value;
  };
  if (spec.kind !== 'builtin' && spec.kind !== 'late-prototype-text') {
    Object.defineProperty(prototype, probe, { configurable: true, get() {
      const table = maps.get(this) ?? 'foreign-table';
      trace.push({ op: 'lookup', table, receiver: id(this) });
      if (spec.kind === 'lookup-throw' && spec.table === table) throw marker;
      if (spec.kind === 'primitive') return primitive(spec.value);
      if (spec.kind === 'receiver-mutation' || spec.kind === 'second-encoder-isolation') {
        if (table === 'vk') { this.KeyA = 991; encoder._codeToScancode.KeyA = 123; }
        return table === 'vk' ? 1 : table === 'sc' ? 2 : 3;
      }
      return rawObject(table);
    } });
  }
  function execute(selected, number) {
    let codeReads = 0;
    const event = {};
    const mask = spec.mask ?? 0;
    const mutation = spec.kind === 'receiver-mutation' || spec.kind === 'second-encoder-isolation';
    const code = spec.kind === 'builtin' ? (spec.location === 'key' ? 'KeyA' : spec.name)
      : spec.kind === 'late-prototype-text' ? '__proto__' : probe;
    const key = spec.kind === 'builtin' ? (spec.location === 'code' ? 'a' : spec.name)
      : spec.kind === 'late-prototype-text' ? '__proto__' : mutation ? 'a' : probe;
    const field = (name, read) => Object.defineProperty(event, name, { get() {
      const value = read(); trace.push({ op: 'field', event: number, name, value: describe(value) }); return value;
    } });
    field('keyCode', () => 65);
    field('key', () => key);
    field('code', () => {
      codeReads++;
      const last = 3 + Number(!!(mask & 4)) + Number(!!(mask & 2));
      if (codeReads === last) {
        phase = 'late';
        if (spec.kind === 'late-prototype-text') Object.defineProperty(prototype, 'toString', {
          configurable: true, value() { trace.push({ op: 'late-prototype-toString' }); return '[late prototype]'; }
        });
      }
      if (mutation && (number > 0 || codeReads > 1)) return 'KeyA';
      return code;
    });
    for (const [name, bit] of [['shiftKey', 1], ['altKey', 2], ['ctrlKey', 4], ['metaKey', 8]]) field(name, () => !!(mask & bit));
    try {
      const output = selected.evaluateKeyboardEvent(event, spec.down);
      results.push({ returned: { type: output.type, cancel: output.cancel,
        key: output.key === undefined ? { kind: 'undefined' } : { kind: 'string', units: units(output.key) } } });
    } catch (error) {
      results.push({ threw: Object.is(error, marker) ? { marker: true }
        : { marker: false, name: String(error.name), message: units(String(error.message)) } });
    }
  }
  try {
    execute(encoder, 0);
    if (spec.kind === 'receiver-mutation') execute(encoder, 1);
    if (spec.kind === 'second-encoder-isolation') {
      const second = new Encoder();
      for (const [map, name] of [[second._codeToVk, 'vk-2'], [second._codeToScancode, 'sc-2'], [second._keyToControlChar, 'uc-2']]) {
        maps.set(map, name); identities.set(map, `table-${name}`);
      }
      execute(second, 1);
    }
    return { caseId: spec.id, results, trace, ownValues: {
      vkKeyA: describe(encoder._codeToVk.KeyA), scKeyA: describe(encoder._codeToScancode.KeyA) } };
  } finally {
    if (savedProbe) Object.defineProperty(prototype, probe, savedProbe); else delete prototype[probe];
    Object.defineProperty(prototype, 'toString', savedText);
  }
}

var KeyboardResultType = /*#__PURE__*/ function(KeyboardResultType) {
    KeyboardResultType[KeyboardResultType["SEND_KEY"] = 0] = "SEND_KEY";
    KeyboardResultType[KeyboardResultType["SELECT_ALL"] = 1] = "SELECT_ALL";
    KeyboardResultType[KeyboardResultType["PAGE_UP"] = 2] = "PAGE_UP";
    KeyboardResultType[KeyboardResultType["PAGE_DOWN"] = 3] = "PAGE_DOWN";
    return KeyboardResultType;
}(KeyboardResultType || {});

var C0 = /*#__PURE__*/ function(C0) {
    C0["NUL"] = "\0";
    C0["SOH"] = "\x01";
    C0["STX"] = "\x02";
    C0["ETX"] = "\x03";
    C0["EOT"] = "\x04";
    C0["ENQ"] = "\x05";
    C0["ACK"] = "\x06";
    C0["BEL"] = "\x07";
    C0["BS"] = "\b";
    C0["HT"] = "	";
    C0["LF"] = "\n";
    C0["VT"] = "\v";
    C0["FF"] = "\f";
    C0["CR"] = "\r";
    C0["SO"] = "\x0e";
    C0["SI"] = "\x0f";
    C0["DLE"] = "\x10";
    C0["DC1"] = "\x11";
    C0["DC2"] = "\x12";
    C0["DC3"] = "\x13";
    C0["DC4"] = "\x14";
    C0["NAK"] = "\x15";
    C0["SYN"] = "\x16";
    C0["ETB"] = "\x17";
    C0["CAN"] = "\x18";
    C0["EM"] = "\x19";
    C0["SUB"] = "\x1a";
    C0["ESC"] = "\x1b";
    C0["FS"] = "\x1c";
    C0["GS"] = "\x1d";
    C0["RS"] = "\x1e";
    C0["US"] = "\x1f";
    C0["SP"] = " ";
    C0["DEL"] = "";
    return C0;
}(C0 || {});
var C1 = /*#__PURE__*/ function(C1) {
    C1["PAD"] = "";
    C1["HOP"] = "";
    C1["BPH"] = "";
    C1["NBH"] = "";
    C1["IND"] = "";
    C1["NEL"] = "";
    C1["SSA"] = "";
    C1["ESA"] = "";
    C1["HTS"] = "";
    C1["HTJ"] = "";
    C1["VTS"] = "";
    C1["PLD"] = "";
    C1["PLU"] = "";
    C1["RI"] = "";
    C1["SS2"] = "";
    C1["SS3"] = "";
    C1["DCS"] = "";
    C1["PU1"] = "";
    C1["PU2"] = "";
    C1["STS"] = "";
    C1["CCH"] = "";
    C1["MW"] = "";
    C1["SPA"] = "";
    C1["EPA"] = "";
    C1["SOS"] = "";
    C1["SGCI"] = "";
    C1["SCI"] = "";
    C1["CSI"] = "";
    C1["ST"] = "";
    C1["OSC"] = "";
    C1["PM"] = "";
    C1["APC"] = "";
    return C1;
}(C1 || {});
var C1ESCAPED = /*#__PURE__*/ function(C1ESCAPED) {
    C1ESCAPED["ST"] = "\x1b\\";
    return C1ESCAPED;
}(C1ESCAPED || {});

var Win32ControlKeyState = /*#__PURE__*/ function(Win32ControlKeyState) {
    Win32ControlKeyState[Win32ControlKeyState["RIGHT_ALT_PRESSED"] = 1] = "RIGHT_ALT_PRESSED";
    Win32ControlKeyState[Win32ControlKeyState["LEFT_ALT_PRESSED"] = 2] = "LEFT_ALT_PRESSED";
    Win32ControlKeyState[Win32ControlKeyState["RIGHT_CTRL_PRESSED"] = 4] = "RIGHT_CTRL_PRESSED";
    Win32ControlKeyState[Win32ControlKeyState["LEFT_CTRL_PRESSED"] = 8] = "LEFT_CTRL_PRESSED";
    Win32ControlKeyState[Win32ControlKeyState["SHIFT_PRESSED"] = 16] = "SHIFT_PRESSED";
    Win32ControlKeyState[Win32ControlKeyState["NUMLOCK_ON"] = 32] = "NUMLOCK_ON";
    Win32ControlKeyState[Win32ControlKeyState["SCROLLLOCK_ON"] = 64] = "SCROLLLOCK_ON";
    Win32ControlKeyState[Win32ControlKeyState["CAPSLOCK_ON"] = 128] = "CAPSLOCK_ON";
    Win32ControlKeyState[Win32ControlKeyState["ENHANCED_KEY"] = 256] = "ENHANCED_KEY";
    return Win32ControlKeyState;
}(Win32ControlKeyState || {});
class Win32InputMode {
    _codeToVk = {
        'KeyA': 0x41,
        'KeyB': 0x42,
        'KeyC': 0x43,
        'KeyD': 0x44,
        'KeyE': 0x45,
        'KeyF': 0x46,
        'KeyG': 0x47,
        'KeyH': 0x48,
        'KeyI': 0x49,
        'KeyJ': 0x4A,
        'KeyK': 0x4B,
        'KeyL': 0x4C,
        'KeyM': 0x4D,
        'KeyN': 0x4E,
        'KeyO': 0x4F,
        'KeyP': 0x50,
        'KeyQ': 0x51,
        'KeyR': 0x52,
        'KeyS': 0x53,
        'KeyT': 0x54,
        'KeyU': 0x55,
        'KeyV': 0x56,
        'KeyW': 0x57,
        'KeyX': 0x58,
        'KeyY': 0x59,
        'KeyZ': 0x5A,
        'Digit0': 0x30,
        'Digit1': 0x31,
        'Digit2': 0x32,
        'Digit3': 0x33,
        'Digit4': 0x34,
        'Digit5': 0x35,
        'Digit6': 0x36,
        'Digit7': 0x37,
        'Digit8': 0x38,
        'Digit9': 0x39,
        'F1': 0x70,
        'F2': 0x71,
        'F3': 0x72,
        'F4': 0x73,
        'F5': 0x74,
        'F6': 0x75,
        'F7': 0x76,
        'F8': 0x77,
        'F9': 0x78,
        'F10': 0x79,
        'F11': 0x7A,
        'F12': 0x7B,
        'F13': 0x7C,
        'F14': 0x7D,
        'F15': 0x7E,
        'F16': 0x7F,
        'F17': 0x80,
        'F18': 0x81,
        'F19': 0x82,
        'F20': 0x83,
        'F21': 0x84,
        'F22': 0x85,
        'F23': 0x86,
        'F24': 0x87,
        'Numpad0': 0x60,
        'Numpad1': 0x61,
        'Numpad2': 0x62,
        'Numpad3': 0x63,
        'Numpad4': 0x64,
        'Numpad5': 0x65,
        'Numpad6': 0x66,
        'Numpad7': 0x67,
        'Numpad8': 0x68,
        'Numpad9': 0x69,
        'NumpadMultiply': 0x6A,
        'NumpadAdd': 0x6B,
        'NumpadSeparator': 0x6C,
        'NumpadSubtract': 0x6D,
        'NumpadDecimal': 0x6E,
        'NumpadDivide': 0x6F,
        'NumpadEnter': 0x0D,
        'NumLock': 0x90,
        'ArrowUp': 0x26,
        'ArrowDown': 0x28,
        'ArrowLeft': 0x25,
        'ArrowRight': 0x27,
        'Home': 0x24,
        'End': 0x23,
        'PageUp': 0x21,
        'PageDown': 0x22,
        'Insert': 0x2D,
        'Delete': 0x2E,
        'ShiftLeft': 0x10,
        'ShiftRight': 0x10,
        'ControlLeft': 0x11,
        'ControlRight': 0x11,
        'AltLeft': 0x12,
        'AltRight': 0x12,
        'MetaLeft': 0x5B,
        'MetaRight': 0x5C,
        'CapsLock': 0x14,
        'ScrollLock': 0x91,
        'Escape': 0x1B,
        'Enter': 0x0D,
        'Tab': 0x09,
        'Space': 0x20,
        'Backspace': 0x08,
        'Pause': 0x13,
        'ContextMenu': 0x5D,
        'PrintScreen': 0x2C,
        'Semicolon': 0xBA,
        'Equal': 0xBB,
        'Comma': 0xBC,
        'Minus': 0xBD,
        'Period': 0xBE,
        'Slash': 0xBF,
        'Backquote': 0xC0,
        'BracketLeft': 0xDB,
        'Backslash': 0xDC,
        'BracketRight': 0xDD,
        'Quote': 0xDE,
        'IntlBackslash': 0xE2
    };
    _codeToScancode = {
        'KeyQ': 0x10,
        'KeyW': 0x11,
        'KeyE': 0x12,
        'KeyR': 0x13,
        'KeyT': 0x14,
        'KeyY': 0x15,
        'KeyU': 0x16,
        'KeyI': 0x17,
        'KeyO': 0x18,
        'KeyP': 0x19,
        'KeyA': 0x1E,
        'KeyS': 0x1F,
        'KeyD': 0x20,
        'KeyF': 0x21,
        'KeyG': 0x22,
        'KeyH': 0x23,
        'KeyJ': 0x24,
        'KeyK': 0x25,
        'KeyL': 0x26,
        'KeyZ': 0x2C,
        'KeyX': 0x2D,
        'KeyC': 0x2E,
        'KeyV': 0x2F,
        'KeyB': 0x30,
        'KeyN': 0x31,
        'KeyM': 0x32,
        'Digit1': 0x02,
        'Digit2': 0x03,
        'Digit3': 0x04,
        'Digit4': 0x05,
        'Digit5': 0x06,
        'Digit6': 0x07,
        'Digit7': 0x08,
        'Digit8': 0x09,
        'Digit9': 0x0A,
        'Digit0': 0x0B,
        'F1': 0x3B,
        'F2': 0x3C,
        'F3': 0x3D,
        'F4': 0x3E,
        'F5': 0x3F,
        'F6': 0x40,
        'F7': 0x41,
        'F8': 0x42,
        'F9': 0x43,
        'F10': 0x44,
        'F11': 0x57,
        'F12': 0x58,
        'Numpad0': 0x52,
        'Numpad1': 0x4F,
        'Numpad2': 0x50,
        'Numpad3': 0x51,
        'Numpad4': 0x4B,
        'Numpad5': 0x4C,
        'Numpad6': 0x4D,
        'Numpad7': 0x47,
        'Numpad8': 0x48,
        'Numpad9': 0x49,
        'NumpadMultiply': 0x37,
        'NumpadAdd': 0x4E,
        'NumpadSubtract': 0x4A,
        'NumpadDecimal': 0x53,
        'NumpadDivide': 0x35,
        'NumpadEnter': 0x1C,
        'NumLock': 0x45,
        'ArrowUp': 0x48,
        'ArrowDown': 0x50,
        'ArrowLeft': 0x4B,
        'ArrowRight': 0x4D,
        'Home': 0x47,
        'End': 0x4F,
        'PageUp': 0x49,
        'PageDown': 0x51,
        'Insert': 0x52,
        'Delete': 0x53,
        'ShiftLeft': 0x2A,
        'ShiftRight': 0x36,
        'ControlLeft': 0x1D,
        'ControlRight': 0x1D,
        'AltLeft': 0x38,
        'AltRight': 0x38,
        'CapsLock': 0x3A,
        'ScrollLock': 0x46,
        'Escape': 0x01,
        'Enter': 0x1C,
        'Tab': 0x0F,
        'Space': 0x39,
        'Backspace': 0x0E,
        'Pause': 0x45,
        'Semicolon': 0x27,
        'Equal': 0x0D,
        'Comma': 0x33,
        'Minus': 0x0C,
        'Period': 0x34,
        'Slash': 0x35,
        'Backquote': 0x29,
        'BracketLeft': 0x1A,
        'Backslash': 0x2B,
        'BracketRight': 0x1B,
        'Quote': 0x28
    };
    _enhancedKeyCodes = new Set([
        'ArrowUp',
        'ArrowDown',
        'ArrowLeft',
        'ArrowRight',
        'Home',
        'End',
        'PageUp',
        'PageDown',
        'Insert',
        'Delete',
        'NumpadEnter',
        'NumpadDivide',
        'ControlRight',
        'AltRight',
        'PrintScreen',
        'Pause',
        'ContextMenu',
        'MetaLeft',
        'MetaRight'
    ]);
    _keyToControlChar = {
        'Enter': 0x0D,
        'Backspace': 0x08,
        'Tab': 0x09,
        'Escape': 0x1B
    };
    _getVirtualKeyCode(ev) {
        const vk = this._codeToVk[ev.code];
        if (vk !== undefined) {
            return vk;
        }
        return ev.keyCode || 0;
    }
    _getScanCode(ev) {
        return this._codeToScancode[ev.code] || 0;
    }
    _getUnicodeChar(ev) {
        if (ev.ctrlKey && !ev.altKey && !ev.metaKey) {
            if (ev.key === 'Enter') {
                return 0x0A;
            }
            if (ev.key === 'Backspace') {
                return 0x7F;
            }
        }
        const controlChar = this._keyToControlChar[ev.key];
        if (controlChar !== undefined) {
            return controlChar;
        }
        if (ev.key.length === 1) {
            const codePoint = ev.key.codePointAt(0) || 0;
            if (ev.ctrlKey && !ev.altKey && !ev.metaKey) {
                if (codePoint >= 0x41 && codePoint <= 0x5A) {
                    return codePoint - 0x40;
                }
                if (codePoint >= 0x61 && codePoint <= 0x7A) {
                    return codePoint - 0x60;
                }
            }
            return codePoint;
        }
        return 0;
    }
    _getControlKeyState(ev) {
        let state = 0;
        if (ev.shiftKey) {
            state |= 16;
        }
        if (ev.ctrlKey) {
            if (ev.code === 'ControlRight') {
                state |= 4;
            } else {
                state |= 8;
            }
        }
        if (ev.altKey) {
            if (ev.code === 'AltRight') {
                state |= 1;
            } else {
                state |= 2;
            }
        }
        if (this._enhancedKeyCodes.has(ev.code)) {
            state |= 256;
        }
        return state;
    }
    evaluateKeyboardEvent(ev, isKeyDown) {
        const vk = this._getVirtualKeyCode(ev);
        const sc = this._getScanCode(ev);
        const uc = this._getUnicodeChar(ev);
        const kd = isKeyDown ? 1 : 0;
        const cs = this._getControlKeyState(ev);
        const rc = 1;
        return {
            type: KeyboardResultType.SEND_KEY,
            cancel: true,
            key: `${C0.ESC}[${vk};${sc};${uc};${kd};${cs};${rc}_`
        };
    }
}

// Licensed under MIT. Independent task185 probes use own-property descriptors
// rather than the retained suite's Object.prototype injection.
const values = ['undefined', 'null', 'false', 'true', 'zero', 'negative-zero', 'nan',
  'infinity', 'negative-infinity', 'empty', 'text', 'high-surrogate', 'low-surrogate',
  'nul', 'bigint-zero', 'bigint-one', 'bigint-large', 'bigint-negative', 'symbol',
  'boxed-false', 'boxed-zero', 'array'];
const operations = ['proxy-hint', 'proxy-throw', 'lookup-throw', 'toString-throw',
  'valueOf-fallback', 'reentrant', 'late-table-mutation'];

function primitive(name) {
  switch (name) {
    case 'undefined': return undefined;
    case 'null': return null;
    case 'false': return false;
    case 'true': return true;
    case 'zero': return 0;
    case 'negative-zero': return -0;
    case 'nan': return NaN;
    case 'infinity': return Infinity;
    case 'negative-infinity': return -Infinity;
    case 'empty': return '';
    case 'text': return 'independent;field';
    case 'high-surrogate': return '\ud800';
    case 'low-surrogate': return '\udfff';
    case 'nul': return '\0';
    case 'bigint-zero': return 0n;
    case 'bigint-one': return 1n;
    case 'bigint-large': return 123456789012345678901234567890n;
    case 'bigint-negative': return -987654321987654321n;
    case 'symbol': return Symbol('independent');
    case 'boxed-false': return Object(false);
    case 'boxed-zero': return Object(0);
    case 'array': return ['a', '\ud800', 'b'];
    default: throw new Error('Unknown independent primitive');
  }
}
const units = value => Array.from({ length: value.length }, (_, index) => value.charCodeAt(index));
function record(value) {
  return { type: value.type, cancel: value.cancel, key: units(value.key) };
}

function capture(Encoder, spec) {
  const encoder = new Encoder();
  const maps = [encoder._codeToVk, encoder._codeToScancode, encoder._keyToControlChar];
  const trace = [];
  const marker = {};
  const probe = '__independent_win32__';
  let phase = 'before-modifiers';
  const value = () => {
    if (values.includes(spec.operation)) return primitive(spec.operation);
    const object = {
      toString() {
        trace.push(['toString', phase, this === object]);
        if (spec.operation === 'toString-throw') throw marker;
        if (spec.operation === 'valueOf-fallback') return {};
        if (spec.operation === 'reentrant') {
          // Reentry uses the same table owners while the outer raw value lives.
          const event = { code: 'KeyA', key: 'a', keyCode: 65,
            shiftKey: false, ctrlKey: false, altKey: false, metaKey: false };
          trace.push(['nested', record(encoder.evaluateKeyboardEvent(event, false))]);
        }
        return `own-${phase}`;
      },
      valueOf() { trace.push(['valueOf', this === object]); return 905; }
    };
    if (spec.operation.startsWith('proxy-')) return new Proxy(object, {
      get(target, name, receiver) {
        trace.push(['proxy-get', typeof name === 'symbol' ? 'Symbol.toPrimitive' : name,
          receiver === proxy]);
        if (spec.operation === 'proxy-throw' && name === Symbol.toPrimitive) throw marker;
        return Reflect.get(target, name, receiver);
      }
    });
    return object;
  };
  // Proxy receivers are checked without changing the receiver's identity.
  let proxy;
  const raw = value();
  proxy = raw;
  for (const [index, table] of maps.entries()) Object.defineProperty(table, probe, {
    configurable: true, get() {
      trace.push(['lookup', index, this === table]);
      if (index === spec.table && spec.operation === 'lookup-throw') throw marker;
      return index === spec.table ? raw : 11 + index;
    }
  });
  const event = {};
  for (const [name, result] of Object.entries({ code: probe, key: probe, keyCode: 211,
    shiftKey: !!(spec.mask & 1), altKey: !!(spec.mask & 2),
    ctrlKey: !!(spec.mask & 4), metaKey: !!(spec.mask & 8) })) {
    Object.defineProperty(event, name, { get() {
      trace.push(['field', name]);
      if (name === 'shiftKey') {
        phase = 'after-modifiers';
        if (spec.operation === 'late-table-mutation') {
          for (const table of maps) Object.defineProperty(table, probe, { value: 999, configurable: true });
        }
      }
      return result;
    } });
  }
  try {
    return { returned: record(encoder.evaluateKeyboardEvent(event, spec.down)), trace };
  } catch (error) {
    return { threw: error === marker ? { marker: true }
      : { name: error.name, message: units(error.message) }, trace };
  }
}

// A getter can change a one-unit key to empty before codePointAt. Source uses
// an explicit || 0 fallback here; preserve zero and all later field reads.
function captureMissingCodepoint(Encoder, mask, down) {
  const encoder = new Encoder();
  const trace = [];
  let keyReads = 0;
  const controlLookups = (mask & 4) && !(mask & 2) && !(mask & 8) ? 3 : 1;
  const event = {};
  const fields = { code: 'KeyA', key: '', keyCode: 65, shiftKey: !!(mask & 1),
    altKey: !!(mask & 2), ctrlKey: !!(mask & 4), metaKey: !!(mask & 8) };
  for (const [name, value] of Object.entries(fields)) Object.defineProperty(event, name, {
    get() {
      trace.push(name);
      if (name !== 'key') return value;
      keyReads++;
      return keyReads <= controlLookups ? '__missing_codepoint__'
        : keyReads === controlLookups + 1 ? 'a' : '';
    }
  });
  return { returned: record(encoder.evaluateKeyboardEvent(event, down)), trace };
}

function compareIndependentWin32(Source, Port) {
  const failures = [];
  let cases = 0;
  for (const operation of [...values, ...operations]) for (const table of [0, 1, 2]) {
    for (const down of [false, true]) for (const mask of [0, 15]) {
      const spec = { operation, table, down, mask };
      const id = `${operation}-${table}-${Number(down)}-${mask}`;
      cases++;
      const expected = capture(Source, spec);
      const actual = capture(Port, spec);
      if (JSON.stringify(actual) !== JSON.stringify(expected)) failures.push({ id, expected, actual });
    }
  }
  for (let mask = 0; mask < 16; mask++) for (const down of [false, true]) {
    cases++;
    const expected = captureMissingCodepoint(Source, mask, down);
    const actual = captureMissingCodepoint(Port, mask, down);
    if (JSON.stringify(actual) !== JSON.stringify(expected)) {
      failures.push({ id: `missing-codepoint-${mask}-${Number(down)}`, expected, actual });
    }
  }
  return { cases, failures };
}

// Test adapter only. The actual output is encoded by Rust through the callback.
function withPort(factory, run) {
  const owned = [];
  class Port {
    constructor() {
      this.native = factory();
      owned.push(this.native);
      this._codeToVk = this.native.table(0);
      this._codeToScancode = this.native.table(1);
      this._keyToControlChar = this.native.table(2);
    }
    evaluateKeyboardEvent(event, down) {
      return { type: 0, cancel: true, key: this.native.encode(event, down) };
    }
  }
  try { return run(Port); }
  finally { for (const native of owned) native.free(); }
}

export function compareWin32Prototypes(factory) {
  const failures = [];
  for (const spec of prototypeCases) {
    const expected = capturePrototypeCase(Win32InputMode, spec);
    const actual = withPort(factory, Port => capturePrototypeCase(Port, spec));
    if (JSON.stringify(actual) !== JSON.stringify(expected)) failures.push({spec, expected, actual});
  }
  const independent = withPort(factory, Port => compareIndependentWin32(Win32InputMode, Port));
  console.log(`Win32 comparison: ${prototypeCases.length} retained and ${independent.cases} independent cases`);
  return { cases: prototypeCases.length, failures, independent };
}
