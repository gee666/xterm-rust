// Copyright (c) 2025 The xterm.js authors. All rights reserved.
// Licensed under MIT. Generated test-only unchanged upstream body; exceeds
// 350 lines to preserve source auditability. Regenerate with kitty-lookup-fixture.mjs.
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

var KittyKeyboardFlags = /*#__PURE__*/ function(KittyKeyboardFlags) {
    KittyKeyboardFlags[KittyKeyboardFlags["NONE"] = 0] = "NONE";
    KittyKeyboardFlags[KittyKeyboardFlags["DISAMBIGUATE_ESCAPE_CODES"] = 1] = "DISAMBIGUATE_ESCAPE_CODES";
    KittyKeyboardFlags[KittyKeyboardFlags["REPORT_EVENT_TYPES"] = 2] = "REPORT_EVENT_TYPES";
    KittyKeyboardFlags[KittyKeyboardFlags["REPORT_ALTERNATE_KEYS"] = 4] = "REPORT_ALTERNATE_KEYS";
    KittyKeyboardFlags[KittyKeyboardFlags["REPORT_ALL_KEYS_AS_ESCAPE_CODES"] = 8] = "REPORT_ALL_KEYS_AS_ESCAPE_CODES";
    KittyKeyboardFlags[KittyKeyboardFlags["REPORT_ASSOCIATED_TEXT"] = 16] = "REPORT_ASSOCIATED_TEXT";
    return KittyKeyboardFlags;
}(KittyKeyboardFlags || {});
var KittyKeyboardEventType = /*#__PURE__*/ function(KittyKeyboardEventType) {
    KittyKeyboardEventType[KittyKeyboardEventType["PRESS"] = 1] = "PRESS";
    KittyKeyboardEventType[KittyKeyboardEventType["REPEAT"] = 2] = "REPEAT";
    KittyKeyboardEventType[KittyKeyboardEventType["RELEASE"] = 3] = "RELEASE";
    return KittyKeyboardEventType;
}(KittyKeyboardEventType || {});
var KittyKeyboardModifiers = /*#__PURE__*/ function(KittyKeyboardModifiers) {
    KittyKeyboardModifiers[KittyKeyboardModifiers["SHIFT"] = 1] = "SHIFT";
    KittyKeyboardModifiers[KittyKeyboardModifiers["ALT"] = 2] = "ALT";
    KittyKeyboardModifiers[KittyKeyboardModifiers["CTRL"] = 4] = "CTRL";
    KittyKeyboardModifiers[KittyKeyboardModifiers["SUPER"] = 8] = "SUPER";
    KittyKeyboardModifiers[KittyKeyboardModifiers["HYPER"] = 16] = "HYPER";
    KittyKeyboardModifiers[KittyKeyboardModifiers["META"] = 32] = "META";
    KittyKeyboardModifiers[KittyKeyboardModifiers["CAPS_LOCK"] = 64] = "CAPS_LOCK";
    KittyKeyboardModifiers[KittyKeyboardModifiers["NUM_LOCK"] = 128] = "NUM_LOCK";
    return KittyKeyboardModifiers;
}(KittyKeyboardModifiers || {});
class KittyKeyboard {
    _functionalKeyCodes = {
        'Escape': 27,
        'Enter': 13,
        'Tab': 9,
        'Backspace': 127,
        'CapsLock': 57358,
        'ScrollLock': 57359,
        'NumLock': 57360,
        'PrintScreen': 57361,
        'Pause': 57362,
        'ContextMenu': 57363,
        'F13': 57376,
        'F14': 57377,
        'F15': 57378,
        'F16': 57379,
        'F17': 57380,
        'F18': 57381,
        'F19': 57382,
        'F20': 57383,
        'F21': 57384,
        'F22': 57385,
        'F23': 57386,
        'F24': 57387,
        'F25': 57388,
        'KP_0': 57399,
        'KP_1': 57400,
        'KP_2': 57401,
        'KP_3': 57402,
        'KP_4': 57403,
        'KP_5': 57404,
        'KP_6': 57405,
        'KP_7': 57406,
        'KP_8': 57407,
        'KP_9': 57408,
        'KP_Decimal': 57409,
        'KP_Divide': 57410,
        'KP_Multiply': 57411,
        'KP_Subtract': 57412,
        'KP_Add': 57413,
        'KP_Enter': 57414,
        'KP_Equal': 57415,
        'ShiftLeft': 57441,
        'ShiftRight': 57447,
        'ControlLeft': 57442,
        'ControlRight': 57448,
        'AltLeft': 57443,
        'AltRight': 57449,
        'MetaLeft': 57444,
        'MetaRight': 57450,
        'MediaPlayPause': 57430,
        'MediaStop': 57432,
        'MediaTrackNext': 57435,
        'MediaTrackPrevious': 57436,
        'AudioVolumeDown': 57438,
        'AudioVolumeUp': 57439,
        'AudioVolumeMute': 57440
    };
    _csiTildeKeys = {
        'Insert': 2,
        'Delete': 3,
        'PageUp': 5,
        'PageDown': 6,
        'F5': 15,
        'F6': 17,
        'F7': 18,
        'F8': 19,
        'F9': 20,
        'F10': 21,
        'F11': 23,
        'F12': 24
    };
    _csiLetterKeys = {
        'ArrowUp': 'A',
        'ArrowDown': 'B',
        'ArrowRight': 'C',
        'ArrowLeft': 'D',
        'Home': 'H',
        'End': 'F'
    };
    _ss3FunctionKeys = {
        'F1': 'P',
        'F2': 'Q',
        'F3': 'R',
        'F4': 'S'
    };
    _getNumpadKeyCode(ev) {
        if (ev.code.startsWith('Numpad')) {
            const suffix = ev.code.slice(6);
            if (suffix >= '0' && suffix <= '9') {
                return 57399 + parseInt(suffix, 10);
            }
            switch(suffix){
                case 'Decimal':
                    return 57409;
                case 'Divide':
                    return 57410;
                case 'Multiply':
                    return 57411;
                case 'Subtract':
                    return 57412;
                case 'Add':
                    return 57413;
                case 'Enter':
                    return 57414;
                case 'Equal':
                    return 57415;
            }
        }
        return undefined;
    }
    _getModifierKeyCode(ev) {
        switch(ev.code){
            case 'ShiftLeft':
                return 57441;
            case 'ShiftRight':
                return 57447;
            case 'ControlLeft':
                return 57442;
            case 'ControlRight':
                return 57448;
            case 'AltLeft':
                return 57443;
            case 'AltRight':
                return 57449;
            case 'MetaLeft':
                return 57444;
            case 'MetaRight':
                return 57450;
        }
        return undefined;
    }
    _encodeModifiers(ev) {
        let mods = 0;
        if (ev.shiftKey) mods |= 1;
        if (ev.altKey) mods |= 2;
        if (ev.ctrlKey) mods |= 4;
        if (ev.metaKey) mods |= 8;
        return mods > 0 ? mods + 1 : 0;
    }
    _getKeyCode(ev, macOptionAsAlt) {
        const numpadCode = this._getNumpadKeyCode(ev);
        if (numpadCode !== undefined) {
            return numpadCode;
        }
        const modifierCode = this._getModifierKeyCode(ev);
        if (modifierCode !== undefined) {
            return modifierCode;
        }
        const funcCode = this._functionalKeyCodes[ev.key];
        if (funcCode !== undefined) {
            return funcCode;
        }
        if ((ev.shiftKey || macOptionAsAlt && ev.altKey) && ev.code) {
            if (ev.code.startsWith('Digit') && ev.code.length === 6) {
                const digit = ev.code.charAt(5);
                if (digit >= '0' && digit <= '9') {
                    return digit.charCodeAt(0);
                }
            }
            if (ev.code.startsWith('Key') && ev.code.length === 4) {
                const letter = ev.code.charAt(3).toLowerCase();
                return letter.charCodeAt(0);
            }
        }
        if (ev.key.length === 1) {
            const code = ev.key.codePointAt(0);
            if (code >= 65 && code <= 90) {
                return code + 32;
            }
            return code;
        }
        return undefined;
    }
    _isModifierKey(ev) {
        return ev.key === 'Shift' || ev.key === 'Control' || ev.key === 'Alt' || ev.key === 'Meta';
    }
    _isLockKey(ev) {
        return ev.key === 'CapsLock' || ev.key === 'NumLock' || ev.key === 'ScrollLock';
    }
    _buildCsiLetterSequence(letter, modifiers, eventType, reportEventTypes) {
        const needsEventType = reportEventTypes && eventType !== 1;
        if (modifiers > 0 || needsEventType) {
            let seq = C0.ESC + '[1;' + (modifiers > 0 ? modifiers : '1');
            if (needsEventType) {
                seq += ':' + eventType;
            }
            seq += letter;
            return seq;
        }
        return C0.ESC + '[' + letter;
    }
    _buildSs3Sequence(letter, modifiers, eventType, reportEventTypes) {
        const needsEventType = reportEventTypes && eventType !== 1;
        if (modifiers > 0 || needsEventType) {
            let seq = C0.ESC + '[1;' + (modifiers > 0 ? modifiers : '1');
            if (needsEventType) {
                seq += ':' + eventType;
            }
            seq += letter;
            return seq;
        }
        return C0.ESC + 'O' + letter;
    }
    _buildCsiTildeSequence(number, modifiers, eventType, reportEventTypes) {
        const needsEventType = reportEventTypes && eventType !== 1;
        let seq = C0.ESC + '[' + number;
        if (modifiers > 0 || needsEventType) {
            seq += ';' + (modifiers > 0 ? modifiers : '1');
            if (needsEventType) {
                seq += ':' + eventType;
            }
        }
        seq += '~';
        return seq;
    }
    _buildCsiUSequence(ev, keyCode, modifiers, eventType, flags, isFunc, isMod) {
        const reportEventTypes = !!(flags & 2);
        const reportAlternateKeys = !!(flags & 4);
        let seq = C0.ESC + '[' + keyCode;
        let shiftedKey;
        if (reportAlternateKeys && ev.shiftKey && ev.key.length === 1 && !isFunc && !isMod) {
            shiftedKey = ev.key.codePointAt(0);
            seq += ':' + shiftedKey;
        }
        const reportAssociatedText = !!(flags & 16) && eventType !== 3 && ev.key.length === 1 && !isFunc && !isMod && !ev.ctrlKey;
        const textCode = reportAssociatedText ? ev.key.codePointAt(0) : undefined;
        const needsEventType = reportEventTypes && eventType !== 1 && (eventType === 3 || textCode === undefined);
        if (modifiers > 0 || needsEventType || textCode !== undefined) {
            seq += ';';
            if (modifiers > 0) {
                seq += modifiers;
            } else if (needsEventType) {
                seq += '1';
            }
            if (needsEventType) {
                seq += ':' + eventType;
            }
        }
        if (textCode !== undefined) {
            seq += ';' + textCode;
        }
        seq += 'u';
        return seq;
    }
    evaluate(ev, flags, eventType = 1, macOptionAsAlt = false) {
        const result = {
            type: KeyboardResultType.SEND_KEY,
            cancel: false,
            key: undefined
        };
        const modifiers = this._encodeModifiers(ev);
        const isMod = this._isModifierKey(ev);
        const reportEventTypes = !!(flags & 2);
        if (!reportEventTypes && eventType === 3) {
            return result;
        }
        if (isMod && !(flags & 8)) {
            return result;
        }
        if (this._isLockKey(ev) && !(flags & 8)) {
            return result;
        }
        const csiLetter = this._csiLetterKeys[ev.key];
        if (csiLetter) {
            result.key = this._buildCsiLetterSequence(csiLetter, modifiers, eventType, reportEventTypes);
            result.cancel = true;
            return result;
        }
        const ss3Letter = this._ss3FunctionKeys[ev.key];
        if (ss3Letter) {
            result.key = this._buildSs3Sequence(ss3Letter, modifiers, eventType, reportEventTypes);
            result.cancel = true;
            return result;
        }
        const tildeCode = this._csiTildeKeys[ev.key];
        if (tildeCode !== undefined) {
            result.key = this._buildCsiTildeSequence(tildeCode, modifiers, eventType, reportEventTypes);
            result.cancel = true;
            return result;
        }
        const keyCode = this._getKeyCode(ev, macOptionAsAlt);
        if (keyCode === undefined) {
            return result;
        }
        const specialKey = keyCode === 13 || keyCode === 9 || keyCode === 127;
        if (specialKey && eventType === 3 && !(flags & 8)) {
            return result;
        }
        const isFunc = this._functionalKeyCodes[ev.key] !== undefined || this._getNumpadKeyCode(ev) !== undefined;
        const useCsiU = !!(flags & 8 || reportEventTypes && eventType === 3 || (flags & 1 || reportEventTypes) && (isFunc && !specialKey || modifiers > 0 && ev.key.length !== 1 || modifiers - 1 > 1));
        if (useCsiU) {
            result.key = this._buildCsiUSequence(ev, keyCode, modifiers, eventType, flags, isFunc, isMod);
            result.cancel = true;
        } else {
            const legacyByte = keyCode === 13 ? '\r' : keyCode === 9 ? '\t' : keyCode === 127 ? '\x7f' : undefined;
            if (legacyByte) {
                result.key = legacyByte;
            } else if (ev.key.length === 1 && !ev.ctrlKey && !ev.altKey && !ev.metaKey) {
                result.key = ev.key;
            }
        }
        return result;
    }
    static shouldUseProtocol(flags) {
        return flags > 0;
    }
}

// Licensed under MIT. Realm-local source/port probes, never production logic.
const rawNames = ['undefined', 'null', 'false', 'true', 'zero', 'negative-zero', 'nan',
  'infinity', 'empty', 'text', 'high-surrogate', 'low-surrogate', 'bigint-zero',
  'bigint-large', 'symbol', 'boxed-false', 'boxed-zero', 'array', 'enter', 'tab',
  'backspace', 'boxed-enter', 'native-function', 'native-object'];
const operations = ['primitive-hint', 'valueOf-first', 'toString-fallback', 'conversion-throw',
  'lookup-throw', 'proxy', 'proxy-throw', 'reentry', 'persistent-mutation', 'cached-functional'];
const fields = ['_functionalKeyCodes', '_csiTildeKeys', '_csiLetterKeys', '_ss3FunctionKeys'];
const units = text => text === undefined ? null :
  Array.from({ length: text.length }, (_, i) => text.charCodeAt(i));
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
    case 'empty': return '';
    case 'text': return 'raw;value';
    case 'high-surrogate': return '\ud800';
    case 'low-surrogate': return '\udfff';
    case 'bigint-zero': return 0n;
    case 'bigint-large': return 123456789012345678901234567890n;
    case 'symbol': return Symbol('kitty');
    case 'boxed-false': return Object(false);
    case 'boxed-zero': return Object(0);
    case 'array': return ['a', '\ud800', 'b'];
    case 'enter': return 13;
    case 'tab': return 9;
    case 'backspace': return 127;
    case 'boxed-enter': return Object(13);
    case 'native-function': return Object.prototype.toString;
    case 'native-object': return Object.prototype;
    default: throw new Error('Unknown raw value');
  }
}
function record(value) { return { type: value.type, cancel: value.cancel, key: units(value.key) }; }

function captureKittyLookup(Encoder, spec) {
  const encoder = new Encoder();
  const other = new Encoder();
  const maps = fields.map(field => encoder[field]);
  const trace = [];
  const marker = {};
  const probe = spec.intrinsicKey ?? '__kitty_raw_lookup__';
  let phase = 'before-lookups';
  let functionalReads = 0;
  const base = { key: probe, code: '', keyCode: 0, shiftKey: !!(spec.mask & 1),
    altKey: !!(spec.mask & 2), ctrlKey: !!(spec.mask & 4), metaKey: !!(spec.mask & 8) };
  const rawObject = {
    valueOf() {
      trace.push(['valueOf', this === raw, phase]);
      if (spec.operation === 'conversion-throw') throw marker;
      if (spec.operation === 'valueOf-first') return 77;
      if (spec.operation === 'persistent-mutation') {
        Object.defineProperty(maps[spec.table], probe, { value: 'persisted', configurable: true });
      }
      if (spec.operation === 'reentry') {
        trace.push(['nested', record(encoder.evaluate({ ...base, key: 'a' }, 8, 1, false))]);
      }
      return {};
    },
    toString() { trace.push(['toString', this === raw, phase]); return 'raw-\ud800'; }
  };
  if (spec.operation === 'primitive-hint') Object.defineProperty(rawObject, Symbol.toPrimitive, {
    value(hint) { trace.push(['toPrimitive', hint, this === rawObject, phase]); return 'hint-\udfff'; }
  });
  let raw = rawNames.includes(spec.operation) ? primitive(spec.operation) : rawObject;
  if (spec.operation.startsWith('proxy')) raw = new Proxy(rawObject, {
    get(target, name, receiver) {
      trace.push(['proxy-get', typeof name === 'symbol' ? 'Symbol.toPrimitive' : name, receiver === raw]);
      if (spec.operation === 'proxy-throw' && name === Symbol.toPrimitive) throw marker;
      return Reflect.get(target, name, receiver);
    }
  });
  function getValue(index, receiver) {
    trace.push(['lookup', index, receiver === maps[index]]);
    if (index === spec.table && spec.operation === 'lookup-throw') throw marker;
    phase = 'lookup-' + index;
    if (index === 0 && ++functionalReads === 2) {
      phase = 'second-functional';
      if (spec.operation === 'cached-functional') return undefined;
    }
    return index === spec.table ? raw : undefined;
  }
  const original = Object.getOwnPropertyDescriptor(Object.prototype, probe);
  if (spec.intrinsicKey === undefined && spec.inherited) Object.defineProperty(Object.prototype, probe, {
    configurable: true, get() {
      const index = maps.indexOf(this);
      // The second encoder must not share the first encoder's table identity.
      if (index < 0) return undefined;
      return getValue(index, this);
    }
  });
  else if (spec.intrinsicKey === undefined) maps.forEach((map, index) => Object.defineProperty(map, probe, {
    configurable: true, get() { return getValue(index, this); }
  }));
  const event = {};
  for (const [name, value] of Object.entries(base)) Object.defineProperty(event, name, {
    get() { trace.push(['field', name]); return value; }
  });
  try {
    const result = record(encoder.evaluate(event, spec.flags, spec.type, false));
    if (spec.operation === 'persistent-mutation') {
      trace.push(['again', record(encoder.evaluate(event, spec.flags, spec.type, false))]);
    }
    if (spec.operation === 'persistent-mutation' || spec.operation === 'reentry') {
      trace.push(['isolated', record(other.evaluate(base, 8, 1, false))]);
    }
    return { result, trace };
  } catch (error) {
    return { threw: error === marker ? { marker: true }
      : { name: error.name, message: units(error.message) }, trace };
  } finally {
    if (spec.intrinsicKey === undefined && spec.inherited) {
      if (original) Object.defineProperty(Object.prototype, probe, original);
      else delete Object.prototype[probe];
    }
  }
}

const kittyLookupSpecs = [];
for (const operation of [...rawNames, ...operations]) for (const table of [0, 1, 2, 3]) {
  for (const mask of [0, 1, 15]) for (const flags of [0, 2, 8, 31]) {
    for (const type of [1, 2, 3]) for (const inherited of [false, true]) {
      kittyLookupSpecs.push({ operation, table, mask, flags, type, inherited });
    }
  }
}
for (const intrinsicKey of ['constructor', '__defineGetter__', '__defineSetter__', 'hasOwnProperty',
  '__lookupGetter__', '__lookupSetter__', 'isPrototypeOf', 'propertyIsEnumerable',
  'toString', 'valueOf', '__proto__', 'toLocaleString']) {
  for (const mask of [0, 1, 15]) for (const flags of [0, 2, 8, 31]) for (const type of [1, 2, 3]) {
    kittyLookupSpecs.push({ operation: 'intrinsic', intrinsicKey, mask, flags, type });
  }
}
function compareKittyLookups(Source, Port) {
  const failures = [];
  for (const spec of kittyLookupSpecs) {
    const expected = captureKittyLookup(Source, spec);
    const actual = captureKittyLookup(Port, spec);
    if (JSON.stringify(actual) !== JSON.stringify(expected)) failures.push({ spec, expected, actual });
  }
  return { cases: kittyLookupSpecs.length, failures };
}

// Test-only adapter: the callback reaches real Rust with production LiveKeyEvent.
export function kittySourceExpected(event) {
  return new KittyKeyboard().evaluate(event, 8, 1, false).key;
}
export function installKittyDomProbe(callback) {
  const name = '__kitty_dom_probe__';
  const original = Object.getOwnPropertyDescriptor(Object.prototype, name);
  Object.defineProperty(Object.prototype, name, {
    configurable: true, get() { return callback(this); }
  });
  return () => {
    if (original) Object.defineProperty(Object.prototype, name, original);
    else delete Object.prototype[name];
  };
}
export function kittyThrowValue(marker) { return { valueOf() { throw marker; } }; }
export function compareKittyRealm(factory) {
  const owned = [];
  class Port {
    constructor() {
      this.native = factory();
      owned.push(this.native);
      ['_functionalKeyCodes', '_csiTildeKeys', '_csiLetterKeys', '_ss3FunctionKeys']
        .forEach((field, index) => { this[field] = this.native.table(index); });
    }
    evaluate(event, flags, type, mac) { return this.native.encode(event, flags, type, mac); }
  }
  try {
    const result = compareKittyLookups(KittyKeyboard, Port);
    console.log('Kitty realm comparisons: ' + result.cases + ', failures: ' + result.failures.length);
    return result;
  } finally {
    for (const native of owned) native.free();
  }
}
