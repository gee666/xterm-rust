import { clearFontFeatures, debugEnabled, deregisterJoiner, fallbackEnd, fallbackInBounds, fallbackNext, fallbackPattern, finishFontLoad, fontBytes, fontFailure, fontFamily, fontFeatures, fontWarning, refreshFont, registerJoiner, setFontFeatures, sortedFallbacks, startsWith, terminalElement } from './snippets/xterm-font-addons-91879fe097c75eac/src/ligature_host.js';
import { outlineLoadingScenario } from './snippets/xterm-font-addons-91879fe097c75eac/tests/support/font_outline_scenarios.mjs';
import { cellUint32 } from './snippets/xterm-wasm-support-11516dc9b8bb3743/inline1.js';
import { bufferAssignField } from './snippets/xterm-wasm-support-11516dc9b8bb3743/inline2.js';
import { unicodeProperties, unicodeVersion, unicodeWidth } from './snippets/xterm-wasm-support-11516dc9b8bb3743/inline7.js';
import { shapeBufferCell, shapeBufferLine, shapeExtendedCell } from './snippets/xterm-wasm-support-11516dc9b8bb3743/src/public_api/buffer_shape.js';
import { identifierClearPrefix, identifierCodeAt, identifierGreater, identifierLess, identifierProperty, identifierWord } from './snippets/xterm-wasm-support-11516dc9b8bb3743/src/public_api/parser_identifier_bridge.js';
import * as __wbg_star0 from './snippets/xterm-wasm-support-11516dc9b8bb3743/inline6.js';
import * as __wbg_star1 from './snippets/xterm-wasm-support-11516dc9b8bb3743/inline7.js';

let wasm;

function isLikeNone(x) {
    return x === undefined || x === null;
}

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_export_1.set(idx, obj);
    return idx;
}

let WASM_VECTOR_LEN = 0;

let cachedUint8ArrayMemory0 = null;

function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

const cachedTextEncoder = (typeof TextEncoder !== 'undefined' ? new TextEncoder('utf-8') : { encode: () => { throw Error('TextEncoder not available') } } );

const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
}
    : function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
        read: arg.length,
        written: buf.length
    };
});

function passStringToWasm0(arg, malloc, realloc) {

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = encodeString(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

let cachedDataViewMemory0 = null;

function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

const cachedTextDecoder = (typeof TextDecoder !== 'undefined' ? new TextDecoder('utf-8', { ignoreBOM: true, fatal: true }) : { decode: () => { throw Error('TextDecoder not available') } } );

if (typeof TextDecoder !== 'undefined') { cachedTextDecoder.decode(); };

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

let cachedUint16ArrayMemory0 = null;

function getUint16ArrayMemory0() {
    if (cachedUint16ArrayMemory0 === null || cachedUint16ArrayMemory0.byteLength === 0) {
        cachedUint16ArrayMemory0 = new Uint16Array(wasm.memory.buffer);
    }
    return cachedUint16ArrayMemory0;
}

function getArrayU16FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint16ArrayMemory0().subarray(ptr / 2, ptr / 2 + len);
}

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(state => {
    wasm.__wbindgen_export_5.get(state.dtor)(state.a, state.b)
});

function makeMutClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {
        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            if (--state.cnt === 0) {
                wasm.__wbindgen_export_5.get(state.dtor)(a, state.b);
                CLOSURE_DTORS.unregister(state);
            } else {
                state.a = a;
            }
        }
    };
    real.original = state;
    CLOSURE_DTORS.register(real, state, state);
    return real;
}

function makeClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {
        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        try {
            return f(state.a, state.b, ...args);
        } finally {
            if (--state.cnt === 0) {
                wasm.__wbindgen_export_5.get(state.dtor)(state.a, state.b);
                state.a = 0;
                CLOSURE_DTORS.unregister(state);
            }
        }
    };
    real.original = state;
    CLOSURE_DTORS.register(real, state, state);
    return real;
}

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches && builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_export_1.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}
/**
 * Register the source joiner directly, without the addon's open/style policy.
 * Omitted fallbacks mean an empty list, unlike the class constructor defaults.
 * @param {any} terminal
 * @param {any} fallbacks
 * @returns {any}
 */
export function enableLigatures(terminal, fallbacks) {
    const ret = wasm.enableLigatures(terminal, fallbacks);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @param {Array<any> | null} [fonts]
 * @returns {Promise<any>}
 */
export function loadFonts(fonts) {
    const ret = wasm.loadFonts(isLikeNone(fonts) ? 0 : addToExternrefTable0(fonts));
    return ret;
}

function passArrayJsValueToWasm0(array, malloc) {
    const ptr = malloc(array.length * 4, 4) >>> 0;
    for (let i = 0; i < array.length; i++) {
        const add = addToExternrefTable0(array[i]);
        getDataViewMemory0().setUint32(ptr + 4 * i, add, true);
    }
    WASM_VECTOR_LEN = array.length;
    return ptr;
}
/**
 * Handler for `console.log` invocations.
 *
 * If a test is currently running it takes the `args` array and stringifies
 * it and appends it to the current output of the test. Otherwise it passes
 * the arguments to the original `console.log` function, psased as
 * `original`.
 * @param {Array<any>} args
 */
export function __wbgtest_console_log(args) {
    wasm.__wbgtest_console_log(args);
}

/**
 * Handler for `console.debug` invocations. See above.
 * @param {Array<any>} args
 */
export function __wbgtest_console_debug(args) {
    wasm.__wbgtest_console_debug(args);
}

/**
 * Handler for `console.info` invocations. See above.
 * @param {Array<any>} args
 */
export function __wbgtest_console_info(args) {
    wasm.__wbgtest_console_info(args);
}

/**
 * Handler for `console.warn` invocations. See above.
 * @param {Array<any>} args
 */
export function __wbgtest_console_warn(args) {
    wasm.__wbgtest_console_warn(args);
}

/**
 * Handler for `console.error` invocations. See above.
 * @param {Array<any>} args
 */
export function __wbgtest_console_error(args) {
    wasm.__wbgtest_console_error(args);
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}
/**
 * @returns {Uint8Array | undefined}
 */
export function __wbgtest_cov_dump() {
    const ret = wasm.__wbgtest_cov_dump();
    let v1;
    if (ret[0] !== 0) {
        v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
    }
    return v1;
}

function __wbg_adapter_40(arg0, arg1, arg2) {
    const ret = wasm.closure2_externref_shim_multivalue_shim(arg0, arg1, arg2);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

function __wbg_adapter_43(arg0, arg1, arg2) {
    const ret = wasm.closure180_externref_shim_multivalue_shim(arg0, arg1, arg2);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

function __wbg_adapter_46(arg0, arg1, arg2) {
    const ret = wasm.closure178_externref_shim_multivalue_shim(arg0, arg1, arg2);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

function __wbg_adapter_49(arg0, arg1) {
    const ret = wasm._dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hda804e59e0622415_multivalue_shim(arg0, arg1);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

function __wbg_adapter_52(arg0, arg1) {
    wasm._dyn_core__ops__function__Fn_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__haabb29f2b0594031(arg0, arg1);
}

function __wbg_adapter_55(arg0, arg1, arg2) {
    wasm.closure642_externref_shim(arg0, arg1, arg2);
}

function __wbg_adapter_277(arg0, arg1) {
    wasm.wasm_bindgen__convert__closures__invoke0_mut__h6ccbabcefd85a5b8(arg0, arg1);
}

function __wbg_adapter_317(arg0, arg1, arg2, arg3, arg4) {
    wasm.closure673_externref_shim(arg0, arg1, arg2, arg3, arg4);
}

function __wbg_adapter_376(arg0, arg1, arg2, arg3) {
    wasm.closure665_externref_shim(arg0, arg1, arg2, arg3);
}

const BufferCellFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_buffercell_free(ptr >>> 0, 1));

export class BufferCell {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(BufferCell.prototype);
        obj.__wbg_ptr = ptr;
        BufferCellFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        BufferCellFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_buffercell_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    isUnderline() {
        const ret = wasm.buffercell_isUnderline(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * @returns {any}
     */
    getUnderlineStyle() {
        const ret = wasm.buffercell_getUnderlineStyle(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @returns {number}
     */
    getUnderlineColor() {
        const ret = wasm.buffercell_getUnderlineColor(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @returns {number}
     */
    getUnderlineColorMode() {
        const ret = wasm.buffercell_getUnderlineColorMode(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * @returns {boolean}
     */
    isUnderlineColorRGB() {
        const ret = wasm.buffercell_isUnderlineColorRGB(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
    }
    /**
     * @returns {boolean}
     */
    isUnderlineColorPalette() {
        const ret = wasm.buffercell_isUnderlineColorPalette(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
    }
    /**
     * @returns {boolean}
     */
    isUnderlineColorDefault() {
        const ret = wasm.buffercell_isUnderlineColorDefault(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
    }
    /**
     * @param {any} other
     * @returns {boolean}
     */
    attributesEquals(other) {
        const ret = wasm.buffercell_attributesEquals(this.__wbg_ptr, other);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
    }
    /**
     * @returns {any}
     */
    getChars() {
        const ret = wasm.buffercell_getChars(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    getCode() {
        const ret = wasm.buffercell_getCode(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @returns {number}
     */
    getWidth() {
        const ret = wasm.buffercell_getWidth(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    getFgColorMode() {
        const ret = wasm.buffercell_getFgColorMode(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    getBgColorMode() {
        const ret = wasm.buffercell_getBgColorMode(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    getFgColor() {
        const ret = wasm.buffercell_getFgColor(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    getBgColor() {
        const ret = wasm.buffercell_getBgColor(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    isBold() {
        const ret = wasm.buffercell_isBold(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    isItalic() {
        const ret = wasm.buffercell_isItalic(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    isDim() {
        const ret = wasm.buffercell_isDim(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    isBlink() {
        const ret = wasm.buffercell_isBlink(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    isInverse() {
        const ret = wasm.buffercell_isInverse(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    isInvisible() {
        const ret = wasm.buffercell_isInvisible(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    isStrikethrough() {
        const ret = wasm.buffercell_isStrikethrough(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    isOverline() {
        const ret = wasm.buffercell_isOverline(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {boolean}
     */
    isFgRGB() {
        const ret = wasm.buffercell_isFgRGB(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    isFgPalette() {
        const ret = wasm.buffercell_isFgPalette(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    isFgDefault() {
        const ret = wasm.buffercell_isFgDefault(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    isBgRGB() {
        const ret = wasm.buffercell_isBgRGB(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    isBgPalette() {
        const ret = wasm.buffercell_isBgPalette(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    isBgDefault() {
        const ret = wasm.buffercell_isBgDefault(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    isAttributeDefault() {
        const ret = wasm.buffercell_isAttributeDefault(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {number | undefined}
     */
    get content() {
        const ret = wasm.buffercell_content(this.__wbg_ptr);
        return ret === 0x100000001 ? undefined : ret;
    }
    /**
     * @param {number | null} [value]
     */
    set content(value) {
        wasm.buffercell_set_content(this.__wbg_ptr, isLikeNone(value) ? 0x100000001 : (value) >>> 0);
    }
    /**
     * @returns {number | undefined}
     */
    get fg() {
        const ret = wasm.buffercell_fg(this.__wbg_ptr);
        return ret === 0x100000001 ? undefined : ret;
    }
    /**
     * @param {number | null} [value]
     */
    set fg(value) {
        wasm.buffercell_set_fg(this.__wbg_ptr, isLikeNone(value) ? 0x100000001 : (value) >>> 0);
    }
    /**
     * @returns {number | undefined}
     */
    get bg() {
        const ret = wasm.buffercell_bg(this.__wbg_ptr);
        return ret === 0x100000001 ? undefined : ret;
    }
    /**
     * @param {number | null} [value]
     */
    set bg(value) {
        wasm.buffercell_set_bg(this.__wbg_ptr, isLikeNone(value) ? 0x100000001 : (value) >>> 0);
    }
    /**
     * @returns {string | undefined}
     */
    get combinedData() {
        const ret = wasm.buffercell_combinedData(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {string | null} [value]
     */
    set combinedData(value) {
        wasm.buffercell_set_combinedData(this.__wbg_ptr, isLikeNone(value) ? 0 : addToExternrefTable0(value));
    }
    /**
     * @returns {any}
     */
    get extended() {
        const ret = wasm.buffercell_extended(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {any} value
     */
    set extended(value) {
        wasm.buffercell_set_extended(this.__wbg_ptr, value);
    }
}

const BufferLineFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_bufferline_free(ptr >>> 0, 1));

export class BufferLine {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(BufferLine.prototype);
        obj.__wbg_ptr = ptr;
        BufferLineFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        BufferLineFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_bufferline_free(ptr, 0);
    }
    /**
     * Copies packed row storage without exporting a mutable WASM memory view.
     * @returns {Uint32Array}
     */
    get _data() {
        const ret = wasm.bufferline__data(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @returns {object}
     */
    get _combined() {
        const ret = wasm.bufferline__combined(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @returns {object}
     */
    get _extendedAttrs() {
        const ret = wasm.bufferline__extendedAttrs(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @returns {boolean}
     */
    get _isWrapped() {
        const ret = wasm.bufferline__isWrapped(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
    }
    /**
     * @returns {boolean}
     */
    get isWrapped() {
        const ret = wasm.bufferline_isWrapped(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
    }
    /**
     * @returns {number}
     */
    get length() {
        const ret = wasm.bufferline_length(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * @param {number} index
     * @param {any} target
     * @returns {any}
     */
    getCell(index, target) {
        const ret = wasm.bufferline_getCell(this.__wbg_ptr, index, target);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @param {boolean | null} [trim_right]
     * @param {number | null} [start]
     * @param {number | null} [end]
     * @returns {string}
     */
    translateToString(trim_right, start, end) {
        const ret = wasm.bufferline_translateToString(this.__wbg_ptr, isLikeNone(trim_right) ? 0xFFFFFF : trim_right ? 1 : 0, !isLikeNone(start), isLikeNone(start) ? 0 : start, !isLikeNone(end), isLikeNone(end) ? 0 : end);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
}

const BufferNamespaceFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_buffernamespace_free(ptr >>> 0, 1));

export class BufferNamespace {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        BufferNamespaceFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_buffernamespace_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    get normal() {
        const ret = wasm.buffernamespace_normal(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {any}
     */
    get alternate() {
        const ret = wasm.buffernamespace_alternate(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {any}
     */
    get active() {
        const ret = wasm.buffernamespace_active(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @returns {Function}
     */
    get onBufferChange() {
        const ret = wasm.buffernamespace_onBufferChange(this.__wbg_ptr);
        return ret;
    }
}

const BufferViewFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_bufferview_free(ptr >>> 0, 1));

export class BufferView {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        BufferViewFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_bufferview_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    get type() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.bufferview_type(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {number}
     */
    get cursorY() {
        const ret = wasm.bufferview_cursorY(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * @returns {number}
     */
    get cursorX() {
        const ret = wasm.bufferview_cursorX(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * @returns {number}
     */
    get viewportY() {
        const ret = wasm.bufferview_viewportY(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * @returns {number}
     */
    get baseY() {
        const ret = wasm.bufferview_baseY(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * @returns {number}
     */
    get length() {
        const ret = wasm.bufferview_length(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * @returns {BufferCell}
     */
    getNullCell() {
        const ret = wasm.bufferview_getNullCell(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @param {number} index
     * @returns {any}
     */
    getLine(index) {
        const ret = wasm.bufferview_getLine(this.__wbg_ptr, index);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
}

const ExtendedCellAttributesFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_extendedcellattributes_free(ptr >>> 0, 1));

export class ExtendedCellAttributes {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(ExtendedCellAttributes.prototype);
        obj.__wbg_ptr = ptr;
        ExtendedCellAttributesFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ExtendedCellAttributesFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_extendedcellattributes_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get _ext() {
        const ret = wasm.extendedcellattributes__ext(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} value
     */
    set _ext(value) {
        wasm.extendedcellattributes_set__ext(this.__wbg_ptr, value);
    }
    /**
     * @returns {number}
     */
    get _urlId() {
        const ret = wasm.extendedcellattributes__urlId(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} value
     */
    set _urlId(value) {
        const ret = wasm.extendedcellattributes_set__urlId(this.__wbg_ptr, value);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @returns {number}
     */
    get ext() {
        const ret = wasm.extendedcellattributes_ext(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} value
     */
    set ext(value) {
        wasm.extendedcellattributes_set_ext(this.__wbg_ptr, value);
    }
    /**
     * @returns {number}
     */
    get urlId() {
        const ret = wasm.extendedcellattributes_urlId(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} value
     */
    set urlId(value) {
        const ret = wasm.extendedcellattributes_set_urlId(this.__wbg_ptr, value);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @returns {number}
     */
    get underlineStyle() {
        const ret = wasm.extendedcellattributes_underlineStyle(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} value
     */
    set underlineStyle(value) {
        wasm.extendedcellattributes_set_underlineStyle(this.__wbg_ptr, value);
    }
    /**
     * @returns {number}
     */
    get underlineColor() {
        const ret = wasm.extendedcellattributes_underlineColor(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} value
     */
    set underlineColor(value) {
        wasm.extendedcellattributes_set_underlineColor(this.__wbg_ptr, value);
    }
    /**
     * @returns {number}
     */
    get underlineVariantOffset() {
        const ret = wasm.extendedcellattributes_underlineVariantOffset(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} value
     */
    set underlineVariantOffset(value) {
        wasm.extendedcellattributes_set_underlineVariantOffset(this.__wbg_ptr, value);
    }
    /**
     * @returns {any}
     */
    get payload() {
        const ret = wasm.extendedcellattributes_payload(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @param {any} value
     */
    set payload(value) {
        wasm.extendedcellattributes_set_payload(this.__wbg_ptr, value);
    }
    /**
     * @returns {boolean}
     */
    isEmpty() {
        const ret = wasm.extendedcellattributes_isEmpty(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {ExtendedCellAttributes}
     */
    clone() {
        const ret = wasm.extendedcellattributes_clone(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
}

const LigaturesAddonFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_ligaturesaddon_free(ptr >>> 0, 1));

export class LigaturesAddon {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(LigaturesAddon.prototype);
        obj.__wbg_ptr = ptr;
        LigaturesAddonFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        LigaturesAddonFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_ligaturesaddon_free(ptr, 0);
    }
    /**
     * @param {any} options
     */
    constructor(options) {
        const ret = wasm.ligaturesaddon_new(options);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        LigaturesAddonFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {any} terminal
     */
    activate(terminal) {
        const ret = wasm.ligaturesaddon_activate(this.__wbg_ptr, terminal);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    dispose() {
        const ret = wasm.ligaturesaddon_dispose(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
}

const ParserApiFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_parserapi_free(ptr >>> 0, 1));

export class ParserApi {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ParserApiFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_parserapi_free(ptr, 0);
    }
    /**
     * @param {any} id
     * @param {Function} callback
     * @returns {any}
     */
    registerCsiHandler(id, callback) {
        const ret = wasm.parserapi_registerCsiHandler(this.__wbg_ptr, id, callback);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @param {any} id
     * @param {Function} callback
     * @returns {any}
     */
    addCsiHandler(id, callback) {
        const ret = wasm.parserapi_addCsiHandler(this.__wbg_ptr, id, callback);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @param {any} id
     * @param {Function} callback
     * @returns {any}
     */
    registerDcsHandler(id, callback) {
        const ret = wasm.parserapi_registerDcsHandler(this.__wbg_ptr, id, callback);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @param {any} id
     * @param {Function} callback
     * @returns {any}
     */
    addDcsHandler(id, callback) {
        const ret = wasm.parserapi_addDcsHandler(this.__wbg_ptr, id, callback);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @param {any} id
     * @param {Function} callback
     * @returns {any}
     */
    registerEscHandler(id, callback) {
        const ret = wasm.parserapi_registerEscHandler(this.__wbg_ptr, id, callback);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @param {any} id
     * @param {Function} callback
     * @returns {any}
     */
    addEscHandler(id, callback) {
        const ret = wasm.parserapi_addEscHandler(this.__wbg_ptr, id, callback);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @param {number} id
     * @param {Function} callback
     * @returns {any}
     */
    registerOscHandler(id, callback) {
        const ret = wasm.parserapi_registerOscHandler(this.__wbg_ptr, id, callback);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @param {number} id
     * @param {Function} callback
     * @returns {any}
     */
    addOscHandler(id, callback) {
        const ret = wasm.parserapi_addOscHandler(this.__wbg_ptr, id, callback);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @param {any} id
     * @param {Function} callback
     * @returns {any}
     */
    registerApcHandler(id, callback) {
        const ret = wasm.parserapi_registerApcHandler(this.__wbg_ptr, id, callback);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
}

const UnicodeApiFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_unicodeapi_free(ptr >>> 0, 1));
/**
 * Each public terminal getter returns a fresh view over the same Rust service.
 */
export class UnicodeApi {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        UnicodeApiFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_unicodeapi_free(ptr, 0);
    }
    /**
     * @param {any} provider
     */
    register(provider) {
        const ret = wasm.unicodeapi_register(this.__wbg_ptr, provider);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @returns {Array<any>}
     */
    get versions() {
        const ret = wasm.unicodeapi_versions(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {string}
     */
    get activeVersion() {
        const ret = wasm.unicodeapi_activeVersion(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {string} value
     */
    set activeVersion(value) {
        const ret = wasm.unicodeapi_set_activeVersion(this.__wbg_ptr, value);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
}

const UnicodeServiceFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_unicodeservice_free(ptr >>> 0, 1));
/**
 * A retained view of the terminal's actual service, not a separate registry.
 * Only the width entry point is exported here; the public Unicode API owns
 * registration and version selection.
 */
export class UnicodeService {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        UnicodeServiceFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_unicodeservice_free(ptr, 0);
    }
    /**
     * @param {string} text
     * @returns {number}
     */
    getStringCellWidth(text) {
        const ret = wasm.unicodeservice_getStringCellWidth(this.__wbg_ptr, text);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
}

const WasmBindgenTestContextFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmbindgentestcontext_free(ptr >>> 0, 1));
/**
 * Runtime test harness support instantiated in JS.
 *
 * The node.js entry script instantiates a `Context` here which is used to
 * drive test execution.
 */
export class WasmBindgenTestContext {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmBindgenTestContextFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmbindgentestcontext_free(ptr, 0);
    }
    /**
     * Creates a new context ready to run tests.
     *
     * A `Context` is the main structure through which test execution is
     * coordinated, and this will collect output and results for all executed
     * tests.
     */
    constructor() {
        const ret = wasm.wasmbindgentestcontext_new();
        this.__wbg_ptr = ret >>> 0;
        WasmBindgenTestContextFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Handle `--include-ignored` flag.
     * @param {boolean} include_ignored
     */
    include_ignored(include_ignored) {
        wasm.wasmbindgentestcontext_include_ignored(this.__wbg_ptr, include_ignored);
    }
    /**
     * Handle filter argument.
     * @param {number} filtered
     */
    filtered_count(filtered) {
        wasm.wasmbindgentestcontext_filtered_count(this.__wbg_ptr, filtered);
    }
    /**
     * Executes a list of tests, returning a promise representing their
     * eventual completion.
     *
     * This is the main entry point for executing tests. All the tests passed
     * in are the JS `Function` object that was plucked off the
     * `WebAssembly.Instance` exports list.
     *
     * The promise returned resolves to either `true` if all tests passed or
     * `false` if at least one test failed.
     * @param {any[]} tests
     * @returns {Promise<any>}
     */
    run(tests) {
        const ptr0 = passArrayJsValueToWasm0(tests, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmbindgentestcontext_run(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
}

const WebFontsAddonFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_webfontsaddon_free(ptr >>> 0, 1));

export class WebFontsAddon {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WebFontsAddonFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_webfontsaddon_free(ptr, 0);
    }
    /**
     * @param {boolean | null} [initial_relayout]
     */
    constructor(initial_relayout) {
        const ret = wasm.webfontsaddon_new(isLikeNone(initial_relayout) ? 0xFFFFFF : initial_relayout ? 1 : 0);
        this.__wbg_ptr = ret >>> 0;
        WebFontsAddonFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @returns {boolean}
     */
    get initialRelayout() {
        const ret = wasm.webfontsaddon_initialRelayout(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} value
     */
    set initialRelayout(value) {
        wasm.webfontsaddon_set_initialRelayout(this.__wbg_ptr, value);
    }
    /**
     * @param {any} terminal
     */
    activate(terminal) {
        const ret = wasm.webfontsaddon_activate(this.__wbg_ptr, terminal);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    dispose() {
        wasm.webfontsaddon_dispose(this.__wbg_ptr);
    }
    /**
     * @param {Array<any> | null} [fonts]
     * @returns {Promise<any>}
     */
    loadFonts(fonts) {
        const ret = wasm.webfontsaddon_loadFonts(this.__wbg_ptr, isLikeNone(fonts) ? 0 : addToExternrefTable0(fonts));
        return ret;
    }
    /**
     * @returns {Promise<any>}
     */
    relayout() {
        const ret = wasm.webfontsaddon_relayout(this.__wbg_ptr);
        return ret;
    }
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                if (module.headers.get('Content-Type') != 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);

    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };

        } else {
            return instance;
        }
    }
}

function __wbg_get_imports() {
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbg_Deno_5e0f323eeba20aa4 = function(arg0) {
        const ret = arg0.Deno;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_String_6f4330ab5fd77ef6 = function(arg0, arg1) {
        const ret = String(arg1);
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_add_46f3deb6ca69c368 = function() { return handleError(function (arg0, arg1) {
        arg0.add(arg1);
    }, arguments) };
    imports.wbg.__wbg_all_d5bf227e8f68795d = function(arg0) {
        const ret = Promise.all(arg0);
        return ret;
    };
    imports.wbg.__wbg_bufferAssignField_01c7dc76d24fca88 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        bufferAssignField(arg0, getStringFromWasm0(arg1, arg2), arg3);
    }, arguments) };
    imports.wbg.__wbg_buffer_609cc3eee51ed158 = function(arg0) {
        const ret = arg0.buffer;
        return ret;
    };
    imports.wbg.__wbg_buffercell_new = function(arg0) {
        const ret = BufferCell.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_bufferline_new = function(arg0) {
        const ret = BufferLine.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_call_672a4d21634d4a24 = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.call(arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_call_7cccdd69e0791ae2 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.call(arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_call_833bed5770ea2041 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        const ret = arg0.call(arg1, arg2, arg3);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_cellUint32_f85bcd5a664547f9 = function() { return handleError(function (arg0) {
        const ret = cellUint32(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_charCodeAt_abe5953e37f4b5a6 = function(arg0, arg1) {
        const ret = arg0.charCodeAt(arg1 >>> 0);
        return ret;
    };
    imports.wbg.__wbg_clearFontFeatures_26531b8106412afe = function() { return handleError(function (arg0) {
        clearFontFeatures(arg0);
    }, arguments) };
    imports.wbg.__wbg_concat_9de968491c4340cf = function(arg0, arg1) {
        const ret = arg0.concat(arg1);
        return ret;
    };
    imports.wbg.__wbg_constructor_75c512bc262c1ab2 = function(arg0) {
        const ret = arg0.constructor;
        return ret;
    };
    imports.wbg.__wbg_debugEnabled_8809666607362290 = function() { return handleError(function (arg0) {
        const ret = debugEnabled(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_deregisterJoiner_38ade75d734fdce6 = function() { return handleError(function (arg0, arg1) {
        const ret = deregisterJoiner(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_document_d249400bd7bd996d = function(arg0) {
        const ret = arg0.document;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_done_769e5ede4b31c67b = function(arg0) {
        const ret = arg0.done;
        return ret;
    };
    imports.wbg.__wbg_error_1004b8c64097413f = function(arg0, arg1) {
        console.error(arg0, arg1);
    };
    imports.wbg.__wbg_error_f19acdc6b5fafb26 = function(arg0, arg1) {
        console.error(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbg_extendedcellattributes_new = function(arg0) {
        const ret = ExtendedCellAttributes.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_fallbackEnd_ad14dfd80cf98217 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = fallbackEnd(arg0, arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_fallbackInBounds_b590985b9638fbb2 = function() { return handleError(function (arg0, arg1) {
        const ret = fallbackInBounds(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_fallbackNext_ba20a9f4fa0f0a5b = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = fallbackNext(arg0, arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_fallbackPattern_c70d4a182a284889 = function() { return handleError(function (arg0, arg1) {
        const ret = fallbackPattern(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_finishFontLoad_1439aa5f49141067 = function() { return handleError(function (arg0, arg1, arg2) {
        finishFontLoad(arg0, arg1, arg2);
    }, arguments) };
    imports.wbg.__wbg_fontBytes_88c90f9df121f80c = function() { return handleError(function (arg0) {
        const ret = fontBytes(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_fontFailure_efffc89a4f1c2260 = function() { return handleError(function (arg0) {
        fontFailure(arg0);
    }, arguments) };
    imports.wbg.__wbg_fontFamily_5dabfea344c81081 = function() { return handleError(function (arg0) {
        const ret = fontFamily(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_fontFeatures_f0ae4b373a1b5507 = function() { return handleError(function (arg0) {
        const ret = fontFeatures(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_fontWarning_55f3eec8bebbbf01 = function() { return handleError(function (arg0, arg1) {
        fontWarning(getStringFromWasm0(arg0, arg1));
    }, arguments) };
    imports.wbg.__wbg_fonts_c31a8b1701c57868 = function(arg0) {
        const ret = arg0.fonts;
        return ret;
    };
    imports.wbg.__wbg_forEach_d6a05ca96422eff9 = function(arg0, arg1, arg2) {
        try {
            var state0 = {a: arg1, b: arg2};
            var cb0 = (arg0, arg1, arg2) => {
                const a = state0.a;
                state0.a = 0;
                try {
                    return __wbg_adapter_317(a, state0.b, arg0, arg1, arg2);
                } finally {
                    state0.a = a;
                }
            };
            arg0.forEach(cb0);
        } finally {
            state0.a = state0.b = 0;
        }
    };
    imports.wbg.__wbg_fromCharCode_5493f9b25e1ce0a1 = function(arg0, arg1) {
        const ret = String.fromCharCode(...getArrayU16FromWasm0(arg0, arg1));
        return ret;
    };
    imports.wbg.__wbg_from_2a5d3e218e67aa85 = function(arg0) {
        const ret = Array.from(arg0);
        return ret;
    };
    imports.wbg.__wbg_getElementById_727e92dd724c370c = function(arg0, arg1, arg2) {
        const ret = arg0.getElementById(getStringFromWasm0(arg1, arg2));
        return ret;
    };
    imports.wbg.__wbg_get_67b2ba62fc30de12 = function() { return handleError(function (arg0, arg1) {
        const ret = Reflect.get(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_get_b9b93047fe3cf45b = function(arg0, arg1) {
        const ret = arg0[arg1 >>> 0];
        return ret;
    };
    imports.wbg.__wbg_has_a5ea9117f258a0ec = function() { return handleError(function (arg0, arg1) {
        const ret = Reflect.has(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_identifierClearPrefix_74df0c4b4c0ae910 = function() { return handleError(function (arg0) {
        identifierClearPrefix(arg0);
    }, arguments) };
    imports.wbg.__wbg_identifierCodeAt_60c89c9b3adfe0a1 = function() { return handleError(function (arg0, arg1) {
        const ret = identifierCodeAt(arg0, arg1 >>> 0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_identifierGreater_9990bb2674c4b8c8 = function() { return handleError(function (arg0, arg1) {
        const ret = identifierGreater(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_identifierLess_d19e88d3b4a9fff1 = function() { return handleError(function (arg0, arg1) {
        const ret = identifierLess(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_identifierProperty_19c42485ff5cba22 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = identifierProperty(arg0, getStringFromWasm0(arg1, arg2));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_identifierWord_baa9dd2ab214b48c = function() { return handleError(function (arg0) {
        const ret = identifierWord(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_instanceof_ArrayBuffer_e14585432e3737fc = function(arg0) {
        let result;
        try {
            result = arg0 instanceof ArrayBuffer;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_Error_4d54113b22d20306 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof Error;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_FontFace_05ca60fd7c6286d4 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof FontFace;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_Promise_935168b8f4b49db3 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof Promise;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_Window_def73ea0955fc569 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof Window;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_isArray_a1eab7e0d067391b = function(arg0) {
        const ret = Array.isArray(arg0);
        return ret;
    };
    imports.wbg.__wbg_is_c7481c65e7e5df9e = function(arg0, arg1) {
        const ret = Object.is(arg0, arg1);
        return ret;
    };
    imports.wbg.__wbg_iterator_9a24c88df860dc65 = function() {
        const ret = Symbol.iterator;
        return ret;
    };
    imports.wbg.__wbg_join_f9ac4707dbaaa63e = function(arg0, arg1, arg2) {
        const ret = arg0.join(getStringFromWasm0(arg1, arg2));
        return ret;
    };
    imports.wbg.__wbg_length_a446193dc22c12f8 = function(arg0) {
        const ret = arg0.length;
        return ret;
    };
    imports.wbg.__wbg_length_d56737991078581b = function(arg0) {
        const ret = arg0.length;
        return ret;
    };
    imports.wbg.__wbg_length_e2d2a49132c1b256 = function(arg0) {
        const ret = arg0.length;
        return ret;
    };
    imports.wbg.__wbg_ligaturesaddon_new = function(arg0) {
        const ret = LigaturesAddon.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_load_01de601270b5b817 = function() { return handleError(function (arg0) {
        const ret = arg0.load();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_message_97a2af9b89d693a3 = function(arg0) {
        const ret = arg0.message;
        return ret;
    };
    imports.wbg.__wbg_name_0b327d569f00ebee = function(arg0) {
        const ret = arg0.name;
        return ret;
    };
    imports.wbg.__wbg_name_ac50ab7b6b731bdc = function(arg0, arg1) {
        const ret = arg1.name;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_new_1ab78df5e132f715 = function(arg0, arg1) {
        const ret = new RangeError(getStringFromWasm0(arg0, arg1));
        return ret;
    };
    imports.wbg.__wbg_new_23a2665fac83c611 = function(arg0, arg1) {
        try {
            var state0 = {a: arg0, b: arg1};
            var cb0 = (arg0, arg1) => {
                const a = state0.a;
                state0.a = 0;
                try {
                    return __wbg_adapter_376(a, state0.b, arg0, arg1);
                } finally {
                    state0.a = a;
                }
            };
            const ret = new Promise(cb0);
            return ret;
        } finally {
            state0.a = state0.b = 0;
        }
    };
    imports.wbg.__wbg_new_405e22f390576ce2 = function() {
        const ret = new Object();
        return ret;
    };
    imports.wbg.__wbg_new_78093c5bd701d017 = function() {
        const ret = new Error();
        return ret;
    };
    imports.wbg.__wbg_new_78feb108b6472713 = function() {
        const ret = new Array();
        return ret;
    };
    imports.wbg.__wbg_new_a12002a7f91c75be = function(arg0) {
        const ret = new Uint8Array(arg0);
        return ret;
    };
    imports.wbg.__wbg_new_b08a00743b8ae2f3 = function(arg0, arg1) {
        const ret = new TypeError(getStringFromWasm0(arg0, arg1));
        return ret;
    };
    imports.wbg.__wbg_new_c68d7209be747379 = function(arg0, arg1) {
        const ret = new Error(getStringFromWasm0(arg0, arg1));
        return ret;
    };
    imports.wbg.__wbg_new_e3b321dcfef89fc7 = function(arg0) {
        const ret = new Uint32Array(arg0);
        return ret;
    };
    imports.wbg.__wbg_newnoargs_105ed471475aaf50 = function(arg0, arg1) {
        const ret = new Function(getStringFromWasm0(arg0, arg1));
        return ret;
    };
    imports.wbg.__wbg_newwithbyteoffsetandlength_d97e637ebe145a9a = function(arg0, arg1, arg2) {
        const ret = new Uint8Array(arg0, arg1 >>> 0, arg2 >>> 0);
        return ret;
    };
    imports.wbg.__wbg_newwithbyteoffsetandlength_f1dead44d1fc7212 = function(arg0, arg1, arg2) {
        const ret = new Uint32Array(arg0, arg1 >>> 0, arg2 >>> 0);
        return ret;
    };
    imports.wbg.__wbg_next_25feadfc0913fea9 = function(arg0) {
        const ret = arg0.next;
        return ret;
    };
    imports.wbg.__wbg_next_6574e1a8a62d1055 = function() { return handleError(function (arg0) {
        const ret = arg0.next();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_now_832024c17817dc01 = function(arg0) {
        const ret = arg0.now();
        return ret;
    };
    imports.wbg.__wbg_of_66b3ee656cbd962b = function(arg0, arg1) {
        const ret = Array.of(arg0, arg1);
        return ret;
    };
    imports.wbg.__wbg_outlineLoadingScenario_b86c9fd163b43a57 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = outlineLoadingScenario(arg0, arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_parse_def2e24ef1252aff = function() { return handleError(function (arg0, arg1) {
        const ret = JSON.parse(getStringFromWasm0(arg0, arg1));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_performance_ae5e84dd26af6765 = function(arg0) {
        const ret = arg0.performance;
        return ret;
    };
    imports.wbg.__wbg_push_737cfc8c1432c2c6 = function(arg0, arg1) {
        const ret = arg0.push(arg1);
        return ret;
    };
    imports.wbg.__wbg_queueMicrotask_97d92b4fcc8a61c5 = function(arg0) {
        queueMicrotask(arg0);
    };
    imports.wbg.__wbg_queueMicrotask_d3219def82552485 = function(arg0) {
        const ret = arg0.queueMicrotask;
        return ret;
    };
    imports.wbg.__wbg_ready_2a81b60faf3ff18a = function() { return handleError(function (arg0) {
        const ret = arg0.ready;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_refreshFont_064d6f3b2dfbeebc = function() { return handleError(function (arg0) {
        refreshFont(arg0);
    }, arguments) };
    imports.wbg.__wbg_registerJoiner_5d415c18bb4ff250 = function() { return handleError(function (arg0, arg1) {
        const ret = registerJoiner(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_reject_b3fcf99063186ff7 = function(arg0) {
        const ret = Promise.reject(arg0);
        return ret;
    };
    imports.wbg.__wbg_resolve_4851785c9c5f573d = function(arg0) {
        const ret = Promise.resolve(arg0);
        return ret;
    };
    imports.wbg.__wbg_self_67c28212685dfde2 = function(arg0) {
        const ret = arg0.self;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_setFontFeatures_0a62be3e26b5837f = function() { return handleError(function (arg0, arg1) {
        setFontFeatures(arg0, arg1);
    }, arguments) };
    imports.wbg.__wbg_set_65595bdd868b3009 = function(arg0, arg1, arg2) {
        arg0.set(arg1, arg2 >>> 0);
    };
    imports.wbg.__wbg_set_bb8cecf6a62b9f46 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = Reflect.set(arg0, arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_settextcontent_4516425a935f461b = function(arg0, arg1, arg2) {
        arg0.textContent = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_shapeBufferCell_56ad0725e377fcf5 = function() { return handleError(function (arg0) {
        const ret = shapeBufferCell(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_shapeBufferLine_39cfa31edce9ed3b = function() { return handleError(function (arg0) {
        const ret = shapeBufferLine(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_shapeExtendedCell_56ad0725e377fcf5 = function() { return handleError(function (arg0) {
        const ret = shapeExtendedCell(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_sortedFallbacks_39afa437a608e45f = function() { return handleError(function (arg0, arg1) {
        const ret = sortedFallbacks(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_stack_54b23675007f71bb = function(arg0, arg1) {
        const ret = arg1.stack;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_stack_8e0a01ea26c1db11 = function(arg0, arg1) {
        const ret = arg1.stack;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_stack_d6edc2612f9df83f = function(arg0) {
        const ret = arg0.stack;
        return ret;
    };
    imports.wbg.__wbg_stack_f851a06e9c78be1d = function(arg0) {
        const ret = arg0.stack;
        return ret;
    };
    imports.wbg.__wbg_startsWith_c31ef9572e666f71 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = startsWith(arg0, arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_static_accessor_DOCUMENT_2badde8ae57cc9f7 = function() {
        const ret = document;
        return ret;
    };
    imports.wbg.__wbg_static_accessor_GLOBAL_88a902d13a557d07 = function() {
        const ret = typeof global === 'undefined' ? null : global;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_static_accessor_GLOBAL_THIS_56578be7e9f832b0 = function() {
        const ret = typeof globalThis === 'undefined' ? null : globalThis;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_static_accessor_SELF_37c5d418e4bf5819 = function() {
        const ret = typeof self === 'undefined' ? null : self;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_static_accessor_WINDOW_5de37043a91a9c40 = function() {
        const ret = typeof window === 'undefined' ? null : window;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_stringify_f7ed6987935b4a24 = function() { return handleError(function (arg0) {
        const ret = JSON.stringify(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_terminalElement_e84eede36bf31d0b = function() { return handleError(function (arg0) {
        const ret = terminalElement(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_textcontent_9581a1676431df94 = function(arg0, arg1) {
        const ret = arg1.textContent;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_then_44b73946d2fb3e7d = function(arg0, arg1) {
        const ret = arg0.then(arg1);
        return ret;
    };
    imports.wbg.__wbg_then_48b406749878a531 = function(arg0, arg1, arg2) {
        const ret = arg0.then(arg1, arg2);
        return ret;
    };
    imports.wbg.__wbg_unicodeProperties_d7e4b644729299b5 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = unicodeProperties(arg0, arg1 >>> 0, arg2 >>> 0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_unicodeVersion_fa12cdcd88e4a7a0 = function() { return handleError(function (arg0) {
        const ret = unicodeVersion(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_unicodeWidth_0c86765eca6cd377 = function() { return handleError(function (arg0, arg1) {
        const ret = unicodeWidth(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_value_cd1ffa7b1ab794f1 = function(arg0) {
        const ret = arg0.value;
        return ret;
    };
    imports.wbg.__wbg_wbgtestinvoke_d8e1edfc10304da7 = function() { return handleError(function (arg0, arg1) {
        try {
            var state0 = {a: arg0, b: arg1};
            var cb0 = () => {
                const a = state0.a;
                state0.a = 0;
                try {
                    return __wbg_adapter_277(a, state0.b, );
                } finally {
                    state0.a = a;
                }
            };
            __wbg_test_invoke(cb0);
        } finally {
            state0.a = state0.b = 0;
        }
    }, arguments) };
    imports.wbg.__wbg_wbgtestogconsolelog_2c6a12219a99cdb3 = function(arg0, arg1) {
        __wbgtest_og_console_log(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbg_wbgtestoutputwriteln_be640e64b2946d47 = function(arg0) {
        __wbg_test_output_writeln(arg0);
    };
    imports.wbg.__wbindgen_boolean_get = function(arg0) {
        const v = arg0;
        const ret = typeof(v) === 'boolean' ? (v ? 1 : 0) : 2;
        return ret;
    };
    imports.wbg.__wbindgen_cb_drop = function(arg0) {
        const obj = arg0.original;
        if (obj.cnt-- == 1) {
            obj.a = 0;
            return true;
        }
        const ret = false;
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper11616 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 643, __wbg_adapter_55);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper178 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 3, __wbg_adapter_40);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper2108 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 181, __wbg_adapter_43);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper2110 = function(arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 179, __wbg_adapter_46);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper2112 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 177, __wbg_adapter_49);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper6124 = function(arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 258, __wbg_adapter_52);
        return ret;
    };
    imports.wbg.__wbindgen_debug_string = function(arg0, arg1) {
        const ret = debugString(arg1);
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbindgen_init_externref_table = function() {
        const table = wasm.__wbindgen_export_1;
        const offset = table.grow(4);
        table.set(0, undefined);
        table.set(offset + 0, undefined);
        table.set(offset + 1, null);
        table.set(offset + 2, true);
        table.set(offset + 3, false);
        ;
    };
    imports.wbg.__wbindgen_is_falsy = function(arg0) {
        const ret = !arg0;
        return ret;
    };
    imports.wbg.__wbindgen_is_function = function(arg0) {
        const ret = typeof(arg0) === 'function';
        return ret;
    };
    imports.wbg.__wbindgen_is_null = function(arg0) {
        const ret = arg0 === null;
        return ret;
    };
    imports.wbg.__wbindgen_is_object = function(arg0) {
        const val = arg0;
        const ret = typeof(val) === 'object' && val !== null;
        return ret;
    };
    imports.wbg.__wbindgen_is_string = function(arg0) {
        const ret = typeof(arg0) === 'string';
        return ret;
    };
    imports.wbg.__wbindgen_is_undefined = function(arg0) {
        const ret = arg0 === undefined;
        return ret;
    };
    imports.wbg.__wbindgen_jsval_eq = function(arg0, arg1) {
        const ret = arg0 === arg1;
        return ret;
    };
    imports.wbg.__wbindgen_memory = function() {
        const ret = wasm.memory;
        return ret;
    };
    imports.wbg.__wbindgen_number_get = function(arg0, arg1) {
        const obj = arg1;
        const ret = typeof(obj) === 'number' ? obj : undefined;
        getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
    };
    imports.wbg.__wbindgen_number_new = function(arg0) {
        const ret = arg0;
        return ret;
    };
    imports.wbg.__wbindgen_string_get = function(arg0, arg1) {
        const obj = arg1;
        const ret = typeof(obj) === 'string' ? obj : undefined;
        var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbindgen_string_new = function(arg0, arg1) {
        const ret = getStringFromWasm0(arg0, arg1);
        return ret;
    };
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };
    imports['./snippets/xterm-wasm-support-11516dc9b8bb3743/inline6.js'] = __wbg_star0;
    imports['./snippets/xterm-wasm-support-11516dc9b8bb3743/inline7.js'] = __wbg_star1;

    return imports;
}

function __wbg_init_memory(imports, memory) {

}

function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    __wbg_init.__wbindgen_wasm_module = module;
    cachedDataViewMemory0 = null;
    cachedUint16ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;


    wasm.__wbindgen_start();
    return wasm;
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (typeof module !== 'undefined') {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();

    __wbg_init_memory(imports);

    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }

    const instance = new WebAssembly.Instance(module, imports);

    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (typeof module_or_path !== 'undefined') {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (typeof module_or_path === 'undefined') {
        module_or_path = new URL('suite_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    __wbg_init_memory(imports);

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync };
export default __wbg_init;
