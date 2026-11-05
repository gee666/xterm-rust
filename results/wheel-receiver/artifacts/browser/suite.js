import { clearFontFeatures, debugEnabled, deregisterJoiner, fallbackEnd, fallbackInBounds, fallbackNext, fallbackPattern, finishFontLoad, fontBytes, fontFailure, fontFamily, fontFeatures, fontWarning, refreshFont, registerJoiner, setFontFeatures, sortedFallbacks, startsWith, terminalElement } from './snippets/xterm-font-addons-91879fe097c75eac/src/ligature_host.js';
import { assignOptions, copyOptionsDefaults, defineOption, initializeOptions, optionLayout, optionMembership, publicOptions } from './snippets/xterm-wasm-support-11516dc9b8bb3743/inline0.js';
import { cellUint32 } from './snippets/xterm-wasm-support-11516dc9b8bb3743/inline1.js';
import { bufferAssignField } from './snippets/xterm-wasm-support-11516dc9b8bb3743/inline2.js';
import { xtermBindLogger, xtermConsoleMethod, xtermInvokeLog, xtermLogPropertyKey } from './snippets/xterm-wasm-support-11516dc9b8bb3743/inline4.js';
import { optionAdd, optionConcat, optionData, optionError, optionLessEqual, optionLess, optionMultiply, optionNumber, optionProperty, optionString } from './snippets/xterm-wasm-support-11516dc9b8bb3743/inline5.js';
import { collectDisposable } from './snippets/xterm-wasm-support-11516dc9b8bb3743/inline6.js';
import { unicodeProperties, unicodeVersion, unicodeWidth } from './snippets/xterm-wasm-support-11516dc9b8bb3743/inline7.js';
import { activateAddon, addonDisposeProperty, addonRecord, applyAddonDispose, disposeAddon, installAddonDispose, pushAddonEntry, readAddonOwner, removeAddon, weakAddonOwner } from './snippets/xterm-wasm-support-11516dc9b8bb3743/src/public_api/addon_bridge.js';
import { shapeBufferCell, shapeBufferLine, shapeExtendedCell } from './snippets/xterm-wasm-support-11516dc9b8bb3743/src/public_api/buffer_shape.js';
import { identifierClearPrefix, identifierCodeAt, identifierGreater, identifierLess, identifierProperty, identifierWord } from './snippets/xterm-wasm-support-11516dc9b8bb3743/src/public_api/parser_identifier_bridge.js';
import { joinerCall, joinerGet, joinerInsert, joinerLessEqual, joinerLess, joinerMaxEnd, joinerMinStart, joinerPushFrom, joinerPush, joinerRemove, joinerReport, joinerSet, joinerSubtractOne } from './snippets/xterm-web-f147851b6cb0c0ef/inline0.js';
import { checkSerializeArrayLength } from './snippets/xterm-web-f147851b6cb0c0ef/inline1.js';
import { arrowClass } from './snippets/xterm-web-f147851b6cb0c0ef/inline10.js';
import { fastAssignStyle, fastAttribute, fastBeginStyle, fastCacheSet, fastCachedClass, fastCachedStyle, fastLayerHint, fastToggle } from './snippets/xterm-web-f147851b6cb0c0ef/inline11.js';
import { scrollChainEntry, scrollWeakWindow } from './snippets/xterm-web-f147851b6cb0c0ef/inline12.js';
import { scrollClock, scrollDivide, scrollGet, scrollMethod0, scrollMethod1, scrollMethod2, scrollMultiply, scrollNegate, scrollSet, scrollSubtract } from './snippets/xterm-web-f147851b6cb0c0ef/inline13.js';
import { dimensionsEvent, keyEvent, pairEvent } from './snippets/xterm-web-f147851b6cb0c0ef/inline14.js';
import { modesObject } from './snippets/xterm-web-f147851b6cb0c0ef/inline15.js';
import { xtermAssignProperty } from './snippets/xterm-web-f147851b6cb0c0ef/inline16.js';
import { disposeMarkerResource, markerPosition, validateMarkerOffset } from './snippets/xterm-web-f147851b6cb0c0ef/inline17.js';
import { missingTouchRecord, touchCall1, touchField } from './snippets/xterm-web-f147851b6cb0c0ef/inline18.js';
import { xtermOffscreenCanvasAvailable } from './snippets/xterm-web-f147851b6cb0c0ef/inline19.js';
import { statusClearPrefix } from './snippets/xterm-web-f147851b6cb0c0ef/inline2.js';
import { prepareLinkRegExp } from './snippets/xterm-web-f147851b6cb0c0ef/inline21.js';
import { accessibility_attribute, accessibility_less, accessibility_position, accessibility_text_length } from './snippets/xterm-web-f147851b6cb0c0ef/inline22.js';
import { imageStorageInitialPixels, imageStoragePixels } from './snippets/xterm-web-f147851b6cb0c0ef/inline23.js';
import { arrowClearTimeout, arrowSetTimeout } from './snippets/xterm-web-f147851b6cb0c0ef/inline24.js';
import { monitorReleaseCapture, monitorSetCapture, monitorWindow } from './snippets/xterm-web-f147851b6cb0c0ef/inline25.js';
import { allBytes, byteLength, completeWrite, debugBytes, rangeBytes } from './snippets/xterm-web-f147851b6cb0c0ef/inline26.js';
import { inputProperty } from './snippets/xterm-web-f147851b6cb0c0ef/inline27.js';
import { xtermMouseBit, xtermMouseDivide, xtermMouseLess, xtermMouseMultiply, xtermMouseNumber, xtermMousePadding, xtermMouseProperty, xtermMouseSubtract, xtermMouseWindow } from './snippets/xterm-web-f147851b6cb0c0ef/inline28.js';
import { rowAdd, rowAddress, rowGet, rowIncrement, rowLessEqual, rowLess, rowMin, rowMultiply, rowShift, rowSubtractOne, rowSubtract } from './snippets/xterm-web-f147851b6cb0c0ef/inline29.js';
import { accessibility_string_add } from './snippets/xterm-web-f147851b6cb0c0ef/inline3.js';
import { statusLess, statusNumber, statusProperty } from './snippets/xterm-web-f147851b6cb0c0ef/inline30.js';
import { imageAddonOptions, imageOptionNumber } from './snippets/xterm-web-f147851b6cb0c0ef/inline32.js';
import { searchAddonValidTerm } from './snippets/xterm-web-f147851b6cb0c0ef/inline33.js';
import { xtermMeasureNumber } from './snippets/xterm-web-f147851b6cb0c0ef/inline4.js';
import { searchAddonFlag, searchAddonGet, searchAddonLimitEnd, searchAddonLimitReached, searchAddonRefresh, searchAddonRegex } from './snippets/xterm-web-f147851b6cb0c0ef/inline5.js';
import { serializeMissingCell } from './snippets/xterm-web-f147851b6cb0c0ef/inline6.js';
import { clipboardAppendByte, clipboardAtob, clipboardBtoa, clipboardCopyByte, clipboardDecodeBytes, clipboardDecodeModern, clipboardDecode, clipboardEncodeBytes, clipboardEncode, clipboardFallbackBytes, clipboardHasByte, clipboardPromise, clipboardReadThen, clipboardRead, clipboardString, clipboardWriteThen, clipboardWrite } from './snippets/xterm-web-f147851b6cb0c0ef/inline7.js';
import { addonBytes, addonCall, addonProperty, addonSet } from './snippets/xterm-web-f147851b6cb0c0ef/inline8.js';
import { clipboardInvoke1, clipboardInvoke2, clipboardPrevent, clipboardProperty, clipboardStop, clipboardSubtract } from './snippets/xterm-web-f147851b6cb0c0ef/inline9.js';
import { queueFocusCache } from './snippets/xterm-web-f147851b6cb0c0ef/src/core_browser_microtask.js';
import { imageHandlerInspection, kittyHandlerInspection } from './snippets/xterm-web-f147851b6cb0c0ef/src/image/addon/handlers.js';
import { imageStorageInspection } from './snippets/xterm-web-f147851b6cb0c0ef/src/image/addon/inspection.js';
import { imageOptionInspection } from './snippets/xterm-web-f147851b6cb0c0ef/src/image/addon/option_inspection.js';
import { pngBitmapFallback } from './snippets/xterm-web-f147851b6cb0c0ef/src/image/kitty_host.js';
import { assignPlaceholderBitmap } from './snippets/xterm-web-f147851b6cb0c0ef/src/image/placeholder.js';
import { sixelTrapText } from './snippets/xterm-web-f147851b6cb0c0ef/src/image/sixel_errors.js';
import { keyboardDefaultText } from './snippets/xterm-web-f147851b6cb0c0ef/src/input/kitty_tables.js';
import { win32TemplateText } from './snippets/xterm-web-f147851b6cb0c0ef/src/input/win32_tables.js';
import { linkAdd, linkCall, linkDescriptors, linkIncrement, linkLessEqual, linkLess, linkMap, linkMultiply, linkProperty, linkSet, linkSubtract } from './snippets/xterm-web-f147851b6cb0c0ef/src/links/raw_link_bridge.js';
import { oscBrowserLink } from './snippets/xterm-web-f147851b6cb0c0ef/src/links/raw_osc_bridge.js';
import { queueDecorationSync } from './snippets/xterm-web-f147851b6cb0c0ef/src/renderer/decoration_microtask.js';
import { requireQueueRuntime, warnQueue } from './snippets/xterm-web-f147851b6cb0c0ef/src/task_queue_bridge.js';
import { dimensionsView } from './snippets/xterm-web-f147851b6cb0c0ef/src/terminal/internal_render.js';
import { cancelWrite, requestWrite } from './snippets/xterm-web-f147851b6cb0c0ef/src/write_scheduler_bridge.js';
import { liveWheelReceiver, wheelReentryAndReplacement, wheelResultsAndThrows } from './snippets/xterm-web-f147851b6cb0c0ef/tests/browser_wheel_receiver_cases.js';
import * as __wbg_star0 from './snippets/xterm-wasm-support-11516dc9b8bb3743/inline0.js';
import * as __wbg_star1 from './snippets/xterm-wasm-support-11516dc9b8bb3743/inline4.js';
import * as __wbg_star2 from './snippets/xterm-wasm-support-11516dc9b8bb3743/inline5.js';
import * as __wbg_star3 from './snippets/xterm-wasm-support-11516dc9b8bb3743/inline6.js';
import * as __wbg_star4 from './snippets/xterm-wasm-support-11516dc9b8bb3743/inline7.js';
import * as __wbg_star5 from './snippets/xterm-wasm-support-11516dc9b8bb3743/src/public_api/addon_bridge.js';
import * as __wbg_star6 from './snippets/xterm-web-f147851b6cb0c0ef/inline0.js';
import * as __wbg_star7 from './snippets/xterm-web-f147851b6cb0c0ef/inline11.js';
import * as __wbg_star8 from './snippets/xterm-web-f147851b6cb0c0ef/inline17.js';
import * as __wbg_star9 from './snippets/xterm-web-f147851b6cb0c0ef/inline31.js';
import * as __wbg_star10 from './snippets/xterm-web-f147851b6cb0c0ef/src/input/kitty_tables.js';
import * as __wbg_star11 from './snippets/xterm-web-f147851b6cb0c0ef/src/input/win32_tables.js';
import * as __wbg_star12 from './snippets/xterm-web-f147851b6cb0c0ef/src/links/raw_link_bridge.js';
import * as __wbg_star13 from './snippets/xterm-web-f147851b6cb0c0ef/src/task_queue_bridge.js';
import * as __wbg_star14 from './snippets/xterm-web-f147851b6cb0c0ef/src/write_scheduler_bridge.js';

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

const cachedTextDecoder = (typeof TextDecoder !== 'undefined' ? new TextDecoder('utf-8', { ignoreBOM: true, fatal: true }) : { decode: () => { throw Error('TextDecoder not available') } } );

if (typeof TextDecoder !== 'undefined') { cachedTextDecoder.decode(); };

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
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

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_export_1.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}
/**
 * @param {Array<any> | null} [fonts]
 * @returns {Promise<any>}
 */
export function loadFonts(fonts) {
    const ret = wasm.loadFonts(isLikeNone(fonts) ? 0 : addToExternrefTable0(fonts));
    return ret;
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

function __wbg_adapter_42(arg0, arg1) {
    const ret = wasm.closure623_externref_shim(arg0, arg1);
    return ret;
}

function __wbg_adapter_45(arg0, arg1, arg2) {
    const ret = wasm.closure650_externref_shim_multivalue_shim(arg0, arg1, arg2);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

function __wbg_adapter_48(arg0, arg1, arg2) {
    wasm.closure646_externref_shim(arg0, arg1, arg2);
}

function __wbg_adapter_51(arg0, arg1, arg2) {
    const ret = wasm.closure660_externref_shim_multivalue_shim(arg0, arg1, arg2);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

function __wbg_adapter_54(arg0, arg1) {
    const ret = wasm._dyn_core__ops__function__Fn_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hcd0dcfec4fbab121_multivalue_shim(arg0, arg1);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

function __wbg_adapter_57(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h6f4174b98aaf8323(arg0, arg1, arg2);
}

function __wbg_adapter_60(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hb91f40168af055a9(arg0, arg1, arg2);
}

function __wbg_adapter_63(arg0, arg1) {
    const ret = wasm._dyn_core__ops__function__Fn_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hadcafc84a18506cc_multivalue_shim(arg0, arg1);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

function __wbg_adapter_66(arg0, arg1, arg2, arg3) {
    const ret = wasm.closure656_externref_shim_multivalue_shim(arg0, arg1, arg2, arg3);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

function __wbg_adapter_69(arg0, arg1, arg2, arg3) {
    const ret = wasm.closure654_externref_shim_multivalue_shim(arg0, arg1, arg2, arg3);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

function __wbg_adapter_72(arg0, arg1) {
    wasm._dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hdb88ec8af928c1f1(arg0, arg1);
}

function __wbg_adapter_75(arg0, arg1, arg2) {
    wasm.closure648_externref_shim(arg0, arg1, arg2);
}

function __wbg_adapter_78(arg0, arg1, arg2, arg3) {
    const ret = wasm.closure668_externref_shim(arg0, arg1, arg2, arg3);
    return ret;
}

function __wbg_adapter_81(arg0, arg1, arg2) {
    const ret = wasm._dyn_core__ops__function__Fn__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hcaae87c30ffd7e1f_multivalue_shim(arg0, arg1, arg2);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

function __wbg_adapter_84(arg0, arg1, arg2) {
    const ret = wasm.closure644_externref_shim_multivalue_shim(arg0, arg1, arg2);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

function __wbg_adapter_87(arg0, arg1, arg2) {
    const ret = wasm.closure664_externref_shim(arg0, arg1, arg2);
    return ret;
}

function __wbg_adapter_90(arg0, arg1, arg2) {
    const ret = wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h4bd2fa9f02bba4d0_multivalue_shim(arg0, arg1, arg2);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

function __wbg_adapter_93(arg0, arg1, arg2) {
    const ret = wasm.closure1861_externref_shim_multivalue_shim(arg0, arg1, arg2);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

function __wbg_adapter_96(arg0, arg1) {
    const ret = wasm._dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h6ba1f6b7b7654eee_multivalue_shim(arg0, arg1);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

function __wbg_adapter_99(arg0, arg1, arg2) {
    const ret = wasm.closure1863_externref_shim_multivalue_shim(arg0, arg1, arg2);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

function __wbg_adapter_102(arg0, arg1) {
    const ret = wasm._dyn_core__ops__function__Fn_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h6bb8594aa947c132_multivalue_shim(arg0, arg1);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

function __wbg_adapter_105(arg0, arg1, arg2) {
    const ptr0 = passStringToWasm0(arg2, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm._dyn_core__ops__function__Fn__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hcf60c50c026fa841_multivalue_shim(arg0, arg1, ptr0, len0);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

function __wbg_adapter_108(arg0, arg1) {
    wasm._dyn_core__ops__function__Fn_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__haabb29f2b0594031(arg0, arg1);
}

function __wbg_adapter_111(arg0, arg1, arg2) {
    const ret = wasm.closure2125_externref_shim_multivalue_shim(arg0, arg1, arg2);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

function __wbg_adapter_114(arg0, arg1, arg2) {
    const ret = wasm.closure2127_externref_shim_multivalue_shim(arg0, arg1, arg2);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return ret[0] !== 0;
}

function __wbg_adapter_117(arg0, arg1, arg2) {
    const ret = wasm.closure2121_externref_shim_multivalue_shim(arg0, arg1, arg2);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

function __wbg_adapter_120(arg0, arg1, arg2, arg3) {
    const ptr0 = passStringToWasm0(arg2, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.closure2131_externref_shim_multivalue_shim(arg0, arg1, ptr0, len0, arg3);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

function __wbg_adapter_123(arg0, arg1, arg2, arg3) {
    const ret = wasm.closure2129_externref_shim_multivalue_shim(arg0, arg1, arg2, arg3);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

function __wbg_adapter_126(arg0, arg1, arg2, arg3, arg4) {
    const ret = wasm.closure2133_externref_shim_multivalue_shim(arg0, arg1, arg2, arg3, arg4);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

function __wbg_adapter_129(arg0, arg1, arg2) {
    wasm.closure2922_externref_shim(arg0, arg1, arg2);
}

function __wbg_adapter_1295(arg0, arg1) {
    wasm.wasm_bindgen__convert__closures__invoke0_mut__h6ccbabcefd85a5b8(arg0, arg1);
}

function __wbg_adapter_1335(arg0, arg1, arg2, arg3, arg4) {
    wasm.closure2953_externref_shim(arg0, arg1, arg2, arg3, arg4);
}

function __wbg_adapter_1446(arg0, arg1, arg2, arg3) {
    wasm.closure2945_externref_shim(arg0, arg1, arg2, arg3);
}

const AttachAddonFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_attachaddon_free(ptr >>> 0, 1));

export class AttachAddon {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        AttachAddonFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_attachaddon_free(ptr, 0);
    }
    /**
     * @param {WebSocket} socket
     * @param {any} options
     */
    constructor(socket, options) {
        const ret = wasm.attachaddon_new(socket, options);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        AttachAddonFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {Terminal} terminal
     */
    activate(terminal) {
        _assertClass(terminal, Terminal);
        const ret = wasm.attachaddon_activate(this.__wbg_ptr, terminal.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    dispose() {
        wasm.attachaddon_dispose(this.__wbg_ptr);
    }
}

const Base64Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_base64_free(ptr >>> 0, 1));

export class Base64 {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(Base64.prototype);
        obj.__wbg_ptr = ptr;
        Base64Finalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        Base64Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_base64_free(ptr, 0);
    }
    constructor() {
        const ret = wasm.base64_new();
        this.__wbg_ptr = ret >>> 0;
        Base64Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {any} value
     * @returns {any}
     */
    encodeText(value) {
        const ret = wasm.base64_encodeText(this.__wbg_ptr, value);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @param {any} value
     * @returns {any}
     */
    decodeText(value) {
        const ret = wasm.base64_decodeText(this.__wbg_ptr, value);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
}

const BrowserClipboardProviderFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_browserclipboardprovider_free(ptr >>> 0, 1));

export class BrowserClipboardProvider {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(BrowserClipboardProvider.prototype);
        obj.__wbg_ptr = ptr;
        BrowserClipboardProviderFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        BrowserClipboardProviderFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_browserclipboardprovider_free(ptr, 0);
    }
    constructor() {
        const ret = wasm.browserclipboardprovider_new();
        this.__wbg_ptr = ret >>> 0;
        BrowserClipboardProviderFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {any} _selection
     * @returns {any}
     */
    readText(_selection) {
        const ret = wasm.browserclipboardprovider_readText(this.__wbg_ptr, _selection);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @param {any} _selection
     * @param {any} text
     * @returns {any}
     */
    writeText(_selection, text) {
        const ret = wasm.browserclipboardprovider_writeText(this.__wbg_ptr, _selection, text);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
}

const BrowserHostFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_browserhost_free(ptr >>> 0, 1));

export class BrowserHost {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        BrowserHostFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_browserhost_free(ptr, 0);
    }
    constructor() {
        const ret = wasm.browserhost_new();
        this.__wbg_ptr = ret >>> 0;
        BrowserHostFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Reopening does not recreate nodes or move the original terminal root.
     * @param {HTMLElement} parent
     */
    open(parent) {
        const ret = wasm.browserhost_open(this.__wbg_ptr, parent);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @returns {HTMLElement | undefined}
     */
    get element() {
        const ret = wasm.browserhost_element(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {HTMLElement | undefined}
     */
    get screenElement() {
        const ret = wasm.browserhost_screenElement(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {HTMLTextAreaElement | undefined}
     */
    get textarea() {
        const ret = wasm.browserhost_textarea(this.__wbg_ptr);
        return ret;
    }
    focus() {
        const ret = wasm.browserhost_focus(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    blur() {
        const ret = wasm.browserhost_blur(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    dispose() {
        wasm.browserhost_dispose(this.__wbg_ptr);
    }
}

const BrowserMouseStateFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_browsermousestate_free(ptr >>> 0, 1));

export class BrowserMouseState {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(BrowserMouseState.prototype);
        obj.__wbg_ptr = ptr;
        BrowserMouseStateFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        BrowserMouseStateFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_browsermousestate_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    get activeProtocol() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.browsermousestate_activeProtocol(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Built-in string keys share native validation and protocol-change events.
     * @param {string} name
     */
    set activeProtocol(name) {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.browsermousestate_set_activeProtocol(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @returns {string}
     */
    get activeEncoding() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.browsermousestate_activeEncoding(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @param {string} name
     */
    set activeEncoding(name) {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.browsermousestate_set_activeEncoding(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @returns {boolean}
     */
    get areMouseEventsActive() {
        const ret = wasm.browsermousestate_areMouseEventsActive(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    get isDefaultEncoding() {
        const ret = wasm.browsermousestate_isDefaultEncoding(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    get isPixelEncoding() {
        const ret = wasm.browsermousestate_isPixelEncoding(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {Function}
     */
    get onProtocolChange() {
        const ret = wasm.browsermousestate_onProtocolChange(this.__wbg_ptr);
        return ret;
    }
    reset() {
        const ret = wasm.browsermousestate_reset(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
}

const BrowserRuntimeFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_browserruntime_free(ptr >>> 0, 1));

export class BrowserRuntime {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        BrowserRuntimeFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_browserruntime_free(ptr, 0);
    }
    constructor() {
        const ret = wasm.browserruntime_new();
        this.__wbg_ptr = ret >>> 0;
        BrowserRuntimeFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @returns {object}
     */
    get strings() {
        const ret = wasm.browserruntime_strings(this.__wbg_ptr);
        return ret;
    }
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

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(BufferNamespace.prototype);
        obj.__wbg_ptr = ptr;
        BufferNamespaceFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

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

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(BufferView.prototype);
        obj.__wbg_ptr = ptr;
        BufferViewFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

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

const ClipboardAddonFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_clipboardaddon_free(ptr >>> 0, 1));

export class ClipboardAddon {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ClipboardAddonFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_clipboardaddon_free(ptr, 0);
    }
    /**
     * @param {any} codec
     * @param {any} provider
     */
    constructor(codec, provider) {
        const ret = wasm.clipboardaddon_new(codec, provider);
        this.__wbg_ptr = ret >>> 0;
        ClipboardAddonFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {Terminal} terminal
     */
    activate(terminal) {
        _assertClass(terminal, Terminal);
        const ret = wasm.clipboardaddon_activate(this.__wbg_ptr, terminal.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    dispose() {
        wasm.clipboardaddon_dispose(this.__wbg_ptr);
    }
}

const CoreDisposalStoreFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_coredisposalstore_free(ptr >>> 0, 1));
/**
 * Read-only inspection of the real core's disposal flag, not a second store.
 */
export class CoreDisposalStore {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(CoreDisposalStore.prototype);
        obj.__wbg_ptr = ptr;
        CoreDisposalStoreFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        CoreDisposalStoreFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_coredisposalstore_free(ptr, 0);
    }
    /**
     * @returns {boolean}
     */
    get _isDisposed() {
        const ret = wasm.coredisposalstore__isDisposed(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    get isDisposed() {
        const ret = wasm.coredisposalstore_isDisposed(this.__wbg_ptr);
        return ret !== 0;
    }
}

const CoreModeServiceViewFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_coremodeserviceview_free(ptr >>> 0, 1));

export class CoreModeServiceView {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(CoreModeServiceView.prototype);
        obj.__wbg_ptr = ptr;
        CoreModeServiceViewFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        CoreModeServiceViewFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_coremodeserviceview_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    get decPrivateModes() {
        const ret = wasm.coremodeserviceview_decPrivateModes(this.__wbg_ptr);
        return ret;
    }
}

const CoreServicesFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_coreservices_free(ptr >>> 0, 1));

export class CoreServices {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(CoreServices.prototype);
        obj.__wbg_ptr = ptr;
        CoreServicesFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        CoreServicesFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_coreservices_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    get _store() {
        const ret = wasm.coreservices__store(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {any}
     */
    get _renderService() {
        const ret = wasm.coreservices__renderService(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {any}
     */
    get unicodeService() {
        const ret = wasm.coreservices_unicodeService(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {any}
     */
    get mouseStateService() {
        const ret = wasm.coreservices_mouseStateService(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {any}
     */
    get coreService() {
        const ret = wasm.coreservices_coreService(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {any}
     */
    get _bufferService() {
        const ret = wasm.coreservices__bufferService(this.__wbg_ptr);
        return ret;
    }
}

const DecPrivateModesViewFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_decprivatemodesview_free(ptr >>> 0, 1));

export class DecPrivateModesView {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(DecPrivateModesView.prototype);
        obj.__wbg_ptr = ptr;
        DecPrivateModesViewFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        DecPrivateModesViewFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_decprivatemodesview_free(ptr, 0);
    }
    /**
     * @returns {boolean}
     */
    get applicationCursorKeys() {
        const ret = wasm.decprivatemodesview_applicationCursorKeys(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    get applicationKeypad() {
        const ret = wasm.decprivatemodesview_applicationKeypad(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    get bracketedPasteMode() {
        const ret = wasm.decprivatemodesview_bracketedPasteMode(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    get colorSchemeUpdates() {
        const ret = wasm.decprivatemodesview_colorSchemeUpdates(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {any}
     */
    get cursorBlink() {
        const ret = wasm.decprivatemodesview_cursorBlink(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {any}
     */
    get cursorStyle() {
        const ret = wasm.decprivatemodesview_cursorStyle(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {boolean}
     */
    get origin() {
        const ret = wasm.decprivatemodesview_origin(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    get reverseWraparound() {
        const ret = wasm.decprivatemodesview_reverseWraparound(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    get sendFocus() {
        const ret = wasm.decprivatemodesview_sendFocus(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    get synchronizedOutput() {
        const ret = wasm.decprivatemodesview_synchronizedOutput(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    get win32InputMode() {
        const ret = wasm.decprivatemodesview_win32InputMode(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    get wraparound() {
        const ret = wasm.decprivatemodesview_wraparound(this.__wbg_ptr);
        return ret !== 0;
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

const FitAddonFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_fitaddon_free(ptr >>> 0, 1));

export class FitAddon {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        FitAddonFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_fitaddon_free(ptr, 0);
    }
    constructor() {
        const ret = wasm.fitaddon_new();
        this.__wbg_ptr = ret >>> 0;
        FitAddonFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {Terminal} terminal
     */
    activate(terminal) {
        _assertClass(terminal, Terminal);
        wasm.fitaddon_activate(this.__wbg_ptr, terminal.__wbg_ptr);
    }
    /**
     * @returns {any}
     */
    proposeDimensions() {
        const ret = wasm.fitaddon_proposeDimensions(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    fit() {
        const ret = wasm.fitaddon_fit(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    dispose() {
        wasm.fitaddon_dispose(this.__wbg_ptr);
    }
}

const ImageAddonFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_imageaddon_free(ptr >>> 0, 1));
/**
 * Image protocols, control sequences and image storage for the Rust terminal.
 */
export class ImageAddon {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ImageAddonFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_imageaddon_free(ptr, 0);
    }
    /**
     * @param {any} options
     */
    constructor(options) {
        const ret = wasm.imageaddon_new(options);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        ImageAddonFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {Terminal} terminal
     */
    activate(terminal) {
        _assertClass(terminal, Terminal);
        const ret = wasm.imageaddon_activate(this.__wbg_ptr, terminal.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    dispose() {
        const ret = wasm.imageaddon_dispose(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @returns {boolean}
     */
    reset() {
        const ret = wasm.imageaddon_reset(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
    }
    /**
     * Live read-only storage inspection. Writable source-private fields are not
     * part of this binding; image decoding and storage remain authoritative Rust.
     * @returns {any}
     */
    get _storage() {
        const ret = wasm.imageaddon__storage(this.__wbg_ptr);
        return ret;
    }
    /**
     * Read-only native handler inspection, not a writable private Map.
     * @returns {any}
     */
    get _handlers() {
        const ret = wasm.imageaddon__handlers(this.__wbg_ptr);
        return ret;
    }
    /**
     * Live read-only options. Placeholder and storage-limit values retain their
     * JavaScript identity; other fields remain typed projections.
     * @returns {any}
     */
    get _opts() {
        const ret = wasm.imageaddon__opts(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Function}
     */
    get onImageAdded() {
        const ret = wasm.imageaddon_onImageAdded(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get storageLimit() {
        const ret = wasm.imageaddon_storageLimit(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {any} limit
     */
    set storageLimit(limit) {
        const ret = wasm.imageaddon_set_storageLimit(this.__wbg_ptr, limit);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @returns {number}
     */
    get storageUsage() {
        const ret = wasm.imageaddon_storageUsage(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Return the original JavaScript value, not its rendering truthiness.
     * @returns {any}
     */
    get showPlaceholder() {
        const ret = wasm.imageaddon_showPlaceholder(this.__wbg_ptr);
        return ret;
    }
    /**
     * Store before rendering; host failures do not roll back this value.
     * @param {any} value
     */
    set showPlaceholder(value) {
        const ret = wasm.imageaddon_set_showPlaceholder(this.__wbg_ptr, value);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @param {number} x
     * @param {number} y
     * @returns {any}
     */
    getImageAtBufferCell(x, y) {
        const ret = wasm.imageaddon_getImageAtBufferCell(this.__wbg_ptr, x, y);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @param {number} x
     * @param {number} y
     * @returns {any}
     */
    extractTileAtBufferCell(x, y) {
        const ret = wasm.imageaddon_extractTileAtBufferCell(this.__wbg_ptr, x, y);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
}

const ImageHandlerInspectionFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_imagehandlerinspection_free(ptr >>> 0, 1));

export class ImageHandlerInspection {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(ImageHandlerInspection.prototype);
        obj.__wbg_ptr = ptr;
        ImageHandlerInspectionFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ImageHandlerInspectionFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_imagehandlerinspection_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get identity() {
        const ret = wasm.imagehandlerinspection_identity(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {string}
     */
    get kind() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.imagehandlerinspection_kind(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    reset() {
        const ret = wasm.imagehandlerinspection_reset(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @returns {any}
     */
    kitty() {
        const ret = wasm.imagehandlerinspection_kitty(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
}

const ImageHandlersIteratorFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_imagehandlersiterator_free(ptr >>> 0, 1));

export class ImageHandlersIterator {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(ImageHandlersIterator.prototype);
        obj.__wbg_ptr = ptr;
        ImageHandlersIteratorFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ImageHandlersIteratorFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_imagehandlersiterator_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    next() {
        const ret = wasm.imagehandlersiterator_next(this.__wbg_ptr);
        return ret;
    }
}

const ImageHandlersReaderFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_imagehandlersreader_free(ptr >>> 0, 1));

export class ImageHandlersReader {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(ImageHandlersReader.prototype);
        obj.__wbg_ptr = ptr;
        ImageHandlersReaderFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ImageHandlersReaderFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_imagehandlersreader_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get size() {
        const ret = wasm.imagehandlersreader_size(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {any} key
     * @returns {boolean}
     */
    has(key) {
        const ret = wasm.imagehandlersreader_has(this.__wbg_ptr, key);
        return ret !== 0;
    }
    /**
     * @param {any} key
     * @returns {any}
     */
    get(key) {
        const ret = wasm.imagehandlersreader_get(this.__wbg_ptr, key);
        return ret;
    }
    /**
     * @returns {ImageHandlersIterator}
     */
    entries() {
        const ret = wasm.imagehandlersreader_entries(this.__wbg_ptr);
        return ImageHandlersIterator.__wrap(ret);
    }
}

const ImageOptionsReaderFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_imageoptionsreader_free(ptr >>> 0, 1));

export class ImageOptionsReader {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(ImageOptionsReader.prototype);
        obj.__wbg_ptr = ptr;
        ImageOptionsReaderFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ImageOptionsReaderFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_imageoptionsreader_free(ptr, 0);
    }
    /**
     * @param {string} key
     * @returns {any}
     */
    get(key) {
        const ptr0 = passStringToWasm0(key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.imageoptionsreader_get(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
}

const ImageSpecInspectionFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_imagespecinspection_free(ptr >>> 0, 1));

export class ImageSpecInspection {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(ImageSpecInspection.prototype);
        obj.__wbg_ptr = ptr;
        ImageSpecInspectionFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ImageSpecInspectionFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_imagespecinspection_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    get orig() {
        const ret = wasm.imagespecinspection_orig(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @returns {any}
     */
    get actual() {
        const ret = wasm.imagespecinspection_actual(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @returns {any}
     */
    get origCellSize() {
        const ret = wasm.imagespecinspection_origCellSize(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @returns {any}
     */
    get actualCellSize() {
        const ret = wasm.imagespecinspection_actualCellSize(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @returns {number}
     */
    get tileCount() {
        const ret = wasm.imagespecinspection_tileCount(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {string}
     */
    get bufferType() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.imagespecinspection_bufferType(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    get layer() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.imagespecinspection_layer(this.__wbg_ptr);
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
    get zIndex() {
        const ret = wasm.imagespecinspection_zIndex(this.__wbg_ptr);
        return ret;
    }
}

const ImageStoreIteratorFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_imagestoreiterator_free(ptr >>> 0, 1));

export class ImageStoreIterator {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(ImageStoreIterator.prototype);
        obj.__wbg_ptr = ptr;
        ImageStoreIteratorFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ImageStoreIteratorFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_imagestoreiterator_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    next() {
        const ret = wasm.imagestoreiterator_next(this.__wbg_ptr);
        return ret;
    }
}

const ImageStoreReaderFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_imagestorereader_free(ptr >>> 0, 1));

export class ImageStoreReader {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(ImageStoreReader.prototype);
        obj.__wbg_ptr = ptr;
        ImageStoreReaderFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ImageStoreReaderFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_imagestorereader_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get size() {
        const ret = wasm.imagestorereader_size(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {any} key
     * @returns {boolean}
     */
    has(key) {
        const ret = wasm.imagestorereader_has(this.__wbg_ptr, key);
        return ret !== 0;
    }
    /**
     * @param {any} key
     * @returns {any}
     */
    get(key) {
        const ret = wasm.imagestorereader_get(this.__wbg_ptr, key);
        return ret;
    }
    /**
     * @returns {ImageStoreIterator}
     */
    entries() {
        const ret = wasm.imagestorereader_entries(this.__wbg_ptr);
        return ImageStoreIterator.__wrap(ret);
    }
}

const InternalBufferServiceViewFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_internalbufferserviceview_free(ptr >>> 0, 1));

export class InternalBufferServiceView {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(InternalBufferServiceView.prototype);
        obj.__wbg_ptr = ptr;
        InternalBufferServiceViewFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        InternalBufferServiceViewFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_internalbufferserviceview_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get cols() {
        const ret = wasm.internalbufferserviceview_cols(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    get rows() {
        const ret = wasm.internalbufferserviceview_rows(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {any}
     */
    get buffer() {
        const ret = wasm.internalbufferserviceview_buffer(this.__wbg_ptr);
        return ret;
    }
}

const InternalBufferViewFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_internalbufferview_free(ptr >>> 0, 1));

export class InternalBufferView {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(InternalBufferView.prototype);
        obj.__wbg_ptr = ptr;
        InternalBufferViewFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        InternalBufferViewFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_internalbufferview_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get scrollTop() {
        const ret = wasm.internalbufferview_scrollTop(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * @returns {number}
     */
    get scrollBottom() {
        const ret = wasm.internalbufferview_scrollBottom(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * @returns {number}
     */
    get x() {
        const ret = wasm.internalbufferview_x(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * @returns {number}
     */
    get y() {
        const ret = wasm.internalbufferview_y(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * @returns {number}
     */
    get ybase() {
        const ret = wasm.internalbufferview_ybase(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * @returns {number}
     */
    get ydisp() {
        const ret = wasm.internalbufferview_ydisp(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
}

const InternalRenderServiceViewFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_internalrenderserviceview_free(ptr >>> 0, 1));
/**
 * Internal diagnostic view. Raw writable service replacement remains unsupported.
 */
export class InternalRenderServiceView {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(InternalRenderServiceView.prototype);
        obj.__wbg_ptr = ptr;
        InternalRenderServiceViewFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        InternalRenderServiceViewFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_internalrenderserviceview_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    get dimensions() {
        const ret = wasm.internalrenderserviceview_dimensions(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
}

const KittyImageInspectionFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_kittyimageinspection_free(ptr >>> 0, 1));

export class KittyImageInspection {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(KittyImageInspection.prototype);
        obj.__wbg_ptr = ptr;
        KittyImageInspectionFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        KittyImageInspectionFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_kittyimageinspection_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get identity() {
        const ret = wasm.kittyimageinspection_identity(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    get id() {
        const ret = wasm.kittyimageinspection_id(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get width() {
        const ret = wasm.kittyimageinspection_width(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get height() {
        const ret = wasm.kittyimageinspection_height(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get format() {
        const ret = wasm.kittyimageinspection_format(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {string}
     */
    get compression() {
        const ret = wasm.kittyimageinspection_compression(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {any}
     */
    get data() {
        const ret = wasm.kittyimageinspection_data(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
}

const KittyImagesReaderFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_kittyimagesreader_free(ptr >>> 0, 1));

export class KittyImagesReader {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(KittyImagesReader.prototype);
        obj.__wbg_ptr = ptr;
        KittyImagesReaderFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        KittyImagesReaderFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_kittyimagesreader_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get size() {
        const ret = wasm.kittyimagesreader_size(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * @param {any} key
     * @returns {boolean}
     */
    has(key) {
        const ret = wasm.kittyimagesreader_has(this.__wbg_ptr, key);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
    }
    /**
     * @param {any} key
     * @returns {any}
     */
    get(key) {
        const ret = wasm.kittyimagesreader_get(this.__wbg_ptr, key);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @returns {KittyKeys}
     */
    entries() {
        const ret = wasm.kittyimagesreader_entries(this.__wbg_ptr);
        return KittyKeys.__wrap(ret);
    }
}

const KittyKeysFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_kittykeys_free(ptr >>> 0, 1));

export class KittyKeys {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(KittyKeys.prototype);
        obj.__wbg_ptr = ptr;
        KittyKeysFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        KittyKeysFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_kittykeys_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    next() {
        const ret = wasm.kittykeys_next(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
}

const KittyPendingReaderFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_kittypendingreader_free(ptr >>> 0, 1));
/**
 * Membership-only diagnostics. Pending command/decoder objects are deliberately
 * not presented as detached snapshots of the source's mutable records.
 */
export class KittyPendingReader {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(KittyPendingReader.prototype);
        obj.__wbg_ptr = ptr;
        KittyPendingReaderFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        KittyPendingReaderFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_kittypendingreader_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get size() {
        const ret = wasm.kittypendingreader_size(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * @param {any} key
     * @returns {boolean}
     */
    has(key) {
        const ret = wasm.kittypendingreader_has(this.__wbg_ptr, key);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
    }
    /**
     * @returns {KittyKeys}
     */
    entries() {
        const ret = wasm.kittypendingreader_entries(this.__wbg_ptr);
        return KittyKeys.__wrap(ret);
    }
}

const KittyPlacementsReaderFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_kittyplacementsreader_free(ptr >>> 0, 1));

export class KittyPlacementsReader {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(KittyPlacementsReader.prototype);
        obj.__wbg_ptr = ptr;
        KittyPlacementsReaderFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        KittyPlacementsReaderFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_kittyplacementsreader_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get size() {
        const ret = wasm.kittyplacementsreader_size(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * @param {any} key
     * @returns {boolean}
     */
    has(key) {
        const ret = wasm.kittyplacementsreader_has(this.__wbg_ptr, key);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
    }
    /**
     * @param {any} key
     * @returns {any}
     */
    get(key) {
        const ret = wasm.kittyplacementsreader_get(this.__wbg_ptr, key);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @returns {KittyKeys}
     */
    entries() {
        const ret = wasm.kittyplacementsreader_entries(this.__wbg_ptr);
        return KittyKeys.__wrap(ret);
    }
}

const LigaturesAddonFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_ligaturesaddon_free(ptr >>> 0, 1));

export class LigaturesAddon {

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

const MarkerFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_marker_free(ptr >>> 0, 1));

export class Marker {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(Marker.prototype);
        obj.__wbg_ptr = ptr;
        MarkerFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        MarkerFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_marker_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get id() {
        const ret = wasm.marker_id(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {any}
     */
    get line() {
        const ret = wasm.marker_line(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {boolean}
     */
    get isDisposed() {
        const ret = wasm.marker_isDisposed(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {Function}
     */
    get onDispose() {
        const ret = wasm.marker_onDispose(this.__wbg_ptr);
        return ret;
    }
    /**
     * Retain the original raw resource without inspecting it until disposal.
     * Duplicate and late registrations are intentional source behavior.
     * @param {any} resource
     * @returns {any}
     */
    register(resource) {
        const ret = wasm.marker_register(this.__wbg_ptr, resource);
        return ret;
    }
    dispose() {
        const ret = wasm.marker_dispose(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
}

const ParserApiFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_parserapi_free(ptr >>> 0, 1));

export class ParserApi {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(ParserApi.prototype);
        obj.__wbg_ptr = ptr;
        ParserApiFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

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

const ProgramStatusAddonFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_programstatusaddon_free(ptr >>> 0, 1));

export class ProgramStatusAddon {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ProgramStatusAddonFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_programstatusaddon_free(ptr, 0);
    }
    constructor() {
        const ret = wasm.programstatusaddon_new();
        this.__wbg_ptr = ret >>> 0;
        ProgramStatusAddonFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {Terminal} terminal
     */
    activate(terminal) {
        _assertClass(terminal, Terminal);
        const ret = wasm.programstatusaddon_activate(this.__wbg_ptr, terminal.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @returns {any}
     */
    get records() {
        const ret = wasm.programstatusaddon_records(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @returns {any}
     */
    get onChange() {
        const ret = wasm.programstatusaddon_onChange(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {any} id
     */
    clear(id) {
        const ret = wasm.programstatusaddon_clear(this.__wbg_ptr, id);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    processExited() {
        const ret = wasm.programstatusaddon_processExited(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    promptStarted() {
        const ret = wasm.programstatusaddon_promptStarted(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    dispose() {
        wasm.programstatusaddon_dispose(this.__wbg_ptr);
    }
}

const ProgressAddonFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_progressaddon_free(ptr >>> 0, 1));

export class ProgressAddon {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ProgressAddonFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_progressaddon_free(ptr, 0);
    }
    constructor() {
        const ret = wasm.progressaddon_new();
        this.__wbg_ptr = ret >>> 0;
        ProgressAddonFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {Terminal} terminal
     */
    activate(terminal) {
        _assertClass(terminal, Terminal);
        const ret = wasm.progressaddon_activate(this.__wbg_ptr, terminal.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @returns {any}
     */
    get progress() {
        const ret = wasm.progressaddon_progress(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @param {any} progress
     */
    set progress(progress) {
        const ret = wasm.progressaddon_set_progress(this.__wbg_ptr, progress);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @returns {any}
     */
    get onChange() {
        const ret = wasm.progressaddon_onChange(this.__wbg_ptr);
        return ret;
    }
    dispose() {
        wasm.progressaddon_dispose(this.__wbg_ptr);
    }
}

const SearchAddonFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_searchaddon_free(ptr >>> 0, 1));

export class SearchAddon {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        SearchAddonFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_searchaddon_free(ptr, 0);
    }
    /**
     * @param {any} initial_options
     */
    constructor(initial_options) {
        const ret = wasm.searchaddon_new(initial_options);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        SearchAddonFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {Terminal} terminal
     */
    activate(terminal) {
        _assertClass(terminal, Terminal);
        const ret = wasm.searchaddon_activate(this.__wbg_ptr, terminal.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @param {any} term
     * @param {any} options
     * @param {any | null} [internal_options]
     * @returns {boolean}
     */
    findNext(term, options, internal_options) {
        const ret = wasm.searchaddon_findNext(this.__wbg_ptr, term, options, isLikeNone(internal_options) ? 0 : addToExternrefTable0(internal_options));
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
    }
    /**
     * @param {any} term
     * @param {any} options
     * @param {any | null} [internal_options]
     * @returns {boolean}
     */
    findPrevious(term, options, internal_options) {
        const ret = wasm.searchaddon_findPrevious(this.__wbg_ptr, term, options, isLikeNone(internal_options) ? 0 : addToExternrefTable0(internal_options));
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
    }
    /**
     * @param {any | null} [retain_cached_term]
     */
    clearDecorations(retain_cached_term) {
        const ret = wasm.searchaddon_clearDecorations(this.__wbg_ptr, isLikeNone(retain_cached_term) ? 0 : addToExternrefTable0(retain_cached_term));
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    clearActiveDecoration() {
        const ret = wasm.searchaddon_clearActiveDecoration(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    dispose() {
        const ret = wasm.searchaddon_dispose(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @returns {Function}
     */
    get onBeforeSearch() {
        const ret = wasm.searchaddon_onBeforeSearch(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Function}
     */
    get onAfterSearch() {
        const ret = wasm.searchaddon_onAfterSearch(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Function}
     */
    get onDidChangeResults() {
        const ret = wasm.searchaddon_onDidChangeResults(this.__wbg_ptr);
        return ret;
    }
}

const SearchSubscriptionFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_searchsubscription_free(ptr >>> 0, 1));

export class SearchSubscription {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(SearchSubscription.prototype);
        obj.__wbg_ptr = ptr;
        SearchSubscriptionFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        SearchSubscriptionFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_searchsubscription_free(ptr, 0);
    }
    dispose() {
        wasm.searchsubscription_dispose(this.__wbg_ptr);
    }
}

const SerializeAddonFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_serializeaddon_free(ptr >>> 0, 1));
/**
 * The compatible JS facade retains its host in a JS field. Rust stores only a
 * weak link, so an unreachable manager/addon/Terminal cycle is collectible.
 */
export class SerializeAddon {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        SerializeAddonFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_serializeaddon_free(ptr, 0);
    }
    constructor() {
        const ret = wasm.serializeaddon_new();
        this.__wbg_ptr = ret >>> 0;
        SerializeAddonFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {Terminal} terminal
     */
    activate(terminal) {
        _assertClass(terminal, Terminal);
        wasm.serializeaddon_activate(this.__wbg_ptr, terminal.__wbg_ptr);
    }
    /**
     * @param {any | null} [options]
     * @returns {string}
     */
    serialize(options) {
        const ret = wasm.serializeaddon_serialize(this.__wbg_ptr, isLikeNone(options) ? 0 : addToExternrefTable0(options));
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @param {any | null} [options]
     * @returns {string}
     */
    serializeAsHTML(options) {
        const ret = wasm.serializeaddon_serializeAsHTML(this.__wbg_ptr, isLikeNone(options) ? 0 : addToExternrefTable0(options));
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    dispose() {
        wasm.serializeaddon_dispose(this.__wbg_ptr);
    }
}

const TerminalFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_terminal_free(ptr >>> 0, 1));
/**
 * Browser API over the Rust engine. No upstream JavaScript terminal is loaded.
 */
export class Terminal {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(Terminal.prototype);
        obj.__wbg_ptr = ptr;
        TerminalFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        TerminalFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_terminal_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    get modes() {
        const ret = wasm.terminal_modes(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @param {any} parent
     */
    open(parent) {
        const ret = wasm.terminal_open(this.__wbg_ptr, parent);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @param {number} start
     * @param {number} end
     */
    refresh(start, end) {
        const ret = wasm.terminal_refresh(this.__wbg_ptr, start, end);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @returns {any}
     */
    get dimensions() {
        const ret = wasm.terminal_dimensions(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @param {Function} handler
     */
    attachCustomKeyEventHandler(handler) {
        wasm.terminal_attachCustomKeyEventHandler(this.__wbg_ptr, handler);
    }
    /**
     * @param {Function} handler
     */
    attachCustomWheelEventHandler(handler) {
        wasm.terminal_attachCustomWheelEventHandler(this.__wbg_ptr, handler);
    }
    /**
     * @param {string} data
     */
    paste(data) {
        const ret = wasm.terminal_paste(this.__wbg_ptr, data);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @returns {any}
     */
    get _core() {
        const ret = wasm.terminal__core(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {any} provider
     * @returns {any}
     */
    registerLinkProvider(provider) {
        const ret = wasm.terminal_registerLinkProvider(this.__wbg_ptr, provider);
        return ret;
    }
    /**
     * @param {any} offset
     * @returns {any}
     */
    registerMarker(offset) {
        const ret = wasm.terminal_registerMarker(this.__wbg_ptr, offset);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * A live read-only Array view, preserving list and marker handle identity.
     * Source's writable Array operations remain unsupported.
     * @returns {Array<any>}
     */
    get markers() {
        const ret = wasm.terminal_markers(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @param {any} initial_options
     */
    constructor(initial_options) {
        const ret = wasm.terminal_new(initial_options);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        TerminalFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {BrowserRuntime} runtime
     */
    _setRuntime(runtime) {
        _assertClass(runtime, BrowserRuntime);
        const ret = wasm.terminal__setRuntime(this.__wbg_ptr, runtime.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @returns {number}
     */
    get cols() {
        const ret = wasm.terminal_cols(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @returns {number}
     */
    get rows() {
        const ret = wasm.terminal_rows(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @returns {object}
     */
    get options() {
        const ret = wasm.terminal_options(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {any} value
     */
    set options(value) {
        const ret = wasm.terminal_set_options(this.__wbg_ptr, value);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @returns {any}
     */
    get buffer() {
        const ret = wasm.terminal_buffer(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {any}
     */
    get parser() {
        const ret = wasm.terminal_parser(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {UnicodeApi}
     */
    get unicode() {
        const ret = wasm.terminal_unicode(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return UnicodeApi.__wrap(ret[0]);
    }
    /**
     * @returns {HTMLElement | undefined}
     */
    get element() {
        const ret = wasm.terminal_element(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {HTMLElement | undefined}
     */
    get screenElement() {
        const ret = wasm.terminal_screenElement(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {HTMLTextAreaElement | undefined}
     */
    get textarea() {
        const ret = wasm.terminal_textarea(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Function}
     */
    get onBell() {
        const ret = wasm.terminal_onBell(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Function}
     */
    get onBinary() {
        const ret = wasm.terminal_onBinary(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Function}
     */
    get onCursorMove() {
        const ret = wasm.terminal_onCursorMove(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Function}
     */
    get onData() {
        const ret = wasm.terminal_onData(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Function}
     */
    get onLineFeed() {
        const ret = wasm.terminal_onLineFeed(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Function}
     */
    get onResize() {
        const ret = wasm.terminal_onResize(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Function}
     */
    get onScroll() {
        const ret = wasm.terminal_onScroll(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Function}
     */
    get onTitleChange() {
        const ret = wasm.terminal_onTitleChange(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Function}
     */
    get onWriteParsed() {
        const ret = wasm.terminal_onWriteParsed(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Function}
     */
    get onKey() {
        const ret = wasm.terminal_onKey(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Function}
     */
    get onRender() {
        const ret = wasm.terminal_onRender(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Function}
     */
    get onSelectionChange() {
        const ret = wasm.terminal_onSelectionChange(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Function}
     */
    get onDimensionsChange() {
        const ret = wasm.terminal_onDimensionsChange(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {Function} handler
     * @returns {number}
     */
    registerCharacterJoiner(handler) {
        const ret = wasm.terminal_registerCharacterJoiner(this.__wbg_ptr, handler);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {number} id
     */
    deregisterCharacterJoiner(id) {
        const ret = wasm.terminal_deregisterCharacterJoiner(this.__wbg_ptr, id);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @returns {boolean}
     */
    hasSelection() {
        const ret = wasm.terminal_hasSelection(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
    }
    /**
     * @returns {string}
     */
    getSelection() {
        const ret = wasm.terminal_getSelection(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @returns {any}
     */
    getSelectionPosition() {
        const ret = wasm.terminal_getSelectionPosition(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @param {number} column
     * @param {number} row
     * @param {number} length
     */
    select(column, row, length) {
        const ret = wasm.terminal_select(this.__wbg_ptr, column, row, length);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    clearSelection() {
        const ret = wasm.terminal_clearSelection(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    selectAll() {
        const ret = wasm.terminal_selectAll(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @param {number} start
     * @param {number} end
     */
    selectLines(start, end) {
        const ret = wasm.terminal_selectLines(this.__wbg_ptr, start, end);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @param {any} value
     * @param {Function | null} [completion]
     */
    write(value, completion) {
        const ret = wasm.terminal_write(this.__wbg_ptr, value, isLikeNone(completion) ? 0 : addToExternrefTable0(completion));
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @param {any} value
     * @param {Function | null} [completion]
     */
    writeln(value, completion) {
        const ret = wasm.terminal_writeln(this.__wbg_ptr, value, isLikeNone(completion) ? 0 : addToExternrefTable0(completion));
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @param {string} value
     * @param {boolean | null} [was_user_input]
     */
    input(value, was_user_input) {
        const ret = wasm.terminal_input(this.__wbg_ptr, value, isLikeNone(was_user_input) ? 0xFFFFFF : was_user_input ? 1 : 0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @param {number} columns
     * @param {number} rows
     */
    resize(columns, rows) {
        const ret = wasm.terminal_resize(this.__wbg_ptr, columns, rows);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    reset() {
        const ret = wasm.terminal_reset(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    clear() {
        const ret = wasm.terminal_clear(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @param {number} amount
     */
    scrollLines(amount) {
        const ret = wasm.terminal_scrollLines(this.__wbg_ptr, amount);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @param {number} amount
     */
    scrollPages(amount) {
        const ret = wasm.terminal_scrollPages(this.__wbg_ptr, amount);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    scrollToTop() {
        const ret = wasm.terminal_scrollToTop(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    scrollToBottom() {
        const ret = wasm.terminal_scrollToBottom(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @param {number} line
     */
    scrollToLine(line) {
        const ret = wasm.terminal_scrollToLine(this.__wbg_ptr, line);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    focus() {
        const ret = wasm.terminal_focus(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    blur() {
        const ret = wasm.terminal_blur(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @returns {Array<any>}
     */
    _takeAddonOwner() {
        const ret = wasm.terminal__takeAddonOwner(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @param {any} public_terminal
     * @param {any} addon
     */
    _loadAddon(public_terminal, addon) {
        const ret = wasm.terminal__loadAddon(this.__wbg_ptr, public_terminal, addon);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    dispose() {
        const ret = wasm.terminal_dispose(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    _disposeCore() {
        const ret = wasm.terminal__disposeCore(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
}

const Unicode11AddonFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_unicode11addon_free(ptr >>> 0, 1));

export class Unicode11Addon {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        Unicode11AddonFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_unicode11addon_free(ptr, 0);
    }
    constructor() {
        const ret = wasm.unicode11addon_new();
        this.__wbg_ptr = ret >>> 0;
        Unicode11AddonFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {Terminal} terminal
     */
    activate(terminal) {
        _assertClass(terminal, Terminal);
        const ret = wasm.unicode11addon_activate(this.__wbg_ptr, terminal.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Source disposal leaves the registered provider available for selection.
     */
    dispose() {
        wasm.unicode11addon_dispose(this.__wbg_ptr);
    }
}

const UnicodeApiFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_unicodeapi_free(ptr >>> 0, 1));
/**
 * Each public terminal getter returns a fresh view over the same Rust service.
 */
export class UnicodeApi {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(UnicodeApi.prototype);
        obj.__wbg_ptr = ptr;
        UnicodeApiFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

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

const UnicodeGraphemesAddonFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_unicodegraphemesaddon_free(ptr >>> 0, 1));

export class UnicodeGraphemesAddon {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        UnicodeGraphemesAddonFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_unicodegraphemesaddon_free(ptr, 0);
    }
    constructor() {
        const ret = wasm.unicodegraphemesaddon_new();
        this.__wbg_ptr = ret >>> 0;
        UnicodeGraphemesAddonFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {Terminal} terminal
     */
    activate(terminal) {
        _assertClass(terminal, Terminal);
        const ret = wasm.unicodegraphemesaddon_activate(this.__wbg_ptr, terminal.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    dispose() {
        const ret = wasm.unicodegraphemesaddon_dispose(this.__wbg_ptr);
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

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(UnicodeService.prototype);
        obj.__wbg_ptr = ptr;
        UnicodeServiceFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

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

const WebLinksAddonFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_weblinksaddon_free(ptr >>> 0, 1));

export class WebLinksAddon {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WebLinksAddonFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_weblinksaddon_free(ptr, 0);
    }
    /**
     * @param {any} handler
     * @param {any} options
     */
    constructor(handler, options) {
        const ret = wasm.weblinksaddon_new(handler, options);
        this.__wbg_ptr = ret >>> 0;
        WebLinksAddonFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * The unchanged upstream suite updates this live constructor options bag.
     * @returns {any}
     */
    get _options() {
        const ret = wasm.weblinksaddon__options(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {Terminal} terminal
     */
    activate(terminal) {
        _assertClass(terminal, Terminal);
        const ret = wasm.weblinksaddon_activate(this.__wbg_ptr, terminal.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    dispose() {
        wasm.weblinksaddon_dispose(this.__wbg_ptr);
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
    imports.wbg.__wbg_accessibilityattribute_8979ee5d9d70a678 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = accessibility_attribute(arg0, getStringFromWasm0(arg1, arg2));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_accessibilityless_284ff5d7de4f657c = function() { return handleError(function (arg0, arg1) {
        const ret = accessibility_less(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_accessibilityposition_2ceb29f526a3816f = function() { return handleError(function (arg0) {
        const ret = accessibility_position(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_accessibilitystringadd_863bb396c8552cdc = function() { return handleError(function (arg0, arg1) {
        const ret = accessibility_string_add(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_accessibilitytextlength_d218e30024c638a0 = function() { return handleError(function (arg0) {
        const ret = accessibility_text_length(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_activateAddon_9c4902dc82a3f23b = function() { return handleError(function (arg0, arg1) {
        activateAddon(arg0, arg1);
    }, arguments) };
    imports.wbg.__wbg_activeElement_367599fdfa7ad115 = function(arg0) {
        const ret = arg0.activeElement;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_addEventListener_7b8506ed3daef7d5 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        arg0.addEventListener(getStringFromWasm0(arg1, arg2), arg3, arg4 !== 0);
    }, arguments) };
    imports.wbg.__wbg_addEventListener_84ae3eac6e15480a = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        arg0.addEventListener(getStringFromWasm0(arg1, arg2), arg3, arg4);
    }, arguments) };
    imports.wbg.__wbg_addEventListener_90e553fdce254421 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        arg0.addEventListener(getStringFromWasm0(arg1, arg2), arg3);
    }, arguments) };
    imports.wbg.__wbg_add_46f3deb6ca69c368 = function() { return handleError(function (arg0, arg1) {
        arg0.add(arg1);
    }, arguments) };
    imports.wbg.__wbg_add_883d9432f9188ef2 = function(arg0, arg1) {
        const ret = arg0.add(arg1);
        return ret;
    };
    imports.wbg.__wbg_add_9b5191a4a4f767dc = function() { return handleError(function (arg0, arg1, arg2) {
        arg0.add(getStringFromWasm0(arg1, arg2));
    }, arguments) };
    imports.wbg.__wbg_addonBytes_db1f92056647ae98 = function() { return handleError(function (arg0) {
        const ret = addonBytes(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_addonCall_b1555cb0af59677e = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        const ret = addonCall(arg0, getStringFromWasm0(arg1, arg2), arg3);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_addonDisposeProperty_eb6b507bb95f2f52 = function() { return handleError(function (arg0) {
        const ret = addonDisposeProperty(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_addonProperty_33345298e67defd2 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = addonProperty(arg0, getStringFromWasm0(arg1, arg2));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_addonRecord_19f7358402f920d9 = function() { return handleError(function (arg0, arg1) {
        const ret = addonRecord(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_addonSet_19bbd38bd4d740c3 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        addonSet(arg0, getStringFromWasm0(arg1, arg2), arg3);
    }, arguments) };
    imports.wbg.__wbg_allBytes_b5e0d411e6dc4d6a = function() { return handleError(function (arg0, arg1) {
        const ret = allBytes(arg1);
        const ptr1 = passArray8ToWasm0(ret, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_all_d5bf227e8f68795d = function(arg0) {
        const ret = Promise.all(arg0);
        return ret;
    };
    imports.wbg.__wbg_altKey_c33c03aed82e4275 = function(arg0) {
        const ret = arg0.altKey;
        return ret;
    };
    imports.wbg.__wbg_altKey_d7495666df921121 = function(arg0) {
        const ret = arg0.altKey;
        return ret;
    };
    imports.wbg.__wbg_anchorNode_4494b2f4749e70c8 = function(arg0) {
        const ret = arg0.anchorNode;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_anchorOffset_1f9f9a7098289f3f = function(arg0) {
        const ret = arg0.anchorOffset;
        return ret;
    };
    imports.wbg.__wbg_appendChild_8204974b7328bf98 = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.appendChild(arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_applyAddonDispose_3ee9022408fd1cce = function() { return handleError(function (arg0, arg1) {
        applyAddonDispose(arg0, arg1);
    }, arguments) };
    imports.wbg.__wbg_apply_36be6a55257c99bf = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.apply(arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_apply_eb9e9b97497f91e4 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = Reflect.apply(arg0, arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_arrowClass_db5f2255f0cf6dcd = function() { return handleError(function (arg0, arg1) {
        arrowClass(arg0, arg1);
    }, arguments) };
    imports.wbg.__wbg_arrowClearTimeout_863fc1d262c8aa26 = function() { return handleError(function (arg0) {
        arrowClearTimeout(arg0);
    }, arguments) };
    imports.wbg.__wbg_arrowSetTimeout_62e366528d552a47 = function() { return handleError(function (arg0, arg1) {
        const ret = arrowSetTimeout(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_assignOptions_15de1d1231cb247f = function() { return handleError(function (arg0, arg1) {
        assignOptions(arg0, arg1);
    }, arguments) };
    imports.wbg.__wbg_assignPlaceholderBitmap_b093321846f9bcb7 = function() { return handleError(function (arg0, arg1) {
        assignPlaceholderBitmap(arg0, arg1);
    }, arguments) };
    imports.wbg.__wbg_base64_new = function(arg0) {
        const ret = Base64.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_blur_c2ad8cc71bac3974 = function() { return handleError(function (arg0) {
        arg0.blur();
    }, arguments) };
    imports.wbg.__wbg_browserclipboardprovider_new = function(arg0) {
        const ret = BrowserClipboardProvider.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_browsermousestate_new = function(arg0) {
        const ret = BrowserMouseState.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_bufferAssignField_01c7dc76d24fca88 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        bufferAssignField(arg0, getStringFromWasm0(arg1, arg2), arg3);
    }, arguments) };
    imports.wbg.__wbg_buffer_09165b52af8c5237 = function(arg0) {
        const ret = arg0.buffer;
        return ret;
    };
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
    imports.wbg.__wbg_buffernamespace_new = function(arg0) {
        const ret = BufferNamespace.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_bufferview_new = function(arg0) {
        const ret = BufferView.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_button_f75c56aec440ea04 = function(arg0) {
        const ret = arg0.button;
        return ret;
    };
    imports.wbg.__wbg_byteLength_96249ba49ce30a3f = function() { return handleError(function (arg0) {
        const ret = byteLength(arg0);
        return ret;
    }, arguments) };
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
    imports.wbg.__wbg_call_b8adc8b1d0a0d8eb = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        const ret = arg0.call(arg1, arg2, arg3, arg4);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_cancelAnimationFrame_089b48301c362fde = function() { return handleError(function (arg0, arg1) {
        arg0.cancelAnimationFrame(arg1);
    }, arguments) };
    imports.wbg.__wbg_cancelIdleCallback_669eb1ed294c8b8b = function(arg0, arg1) {
        arg0.cancelIdleCallback(arg1 >>> 0);
    };
    imports.wbg.__wbg_cancelWrite_a26dfa431fc474b8 = function() { return handleError(function (arg0) {
        cancelWrite(arg0);
    }, arguments) };
    imports.wbg.__wbg_cellUint32_f85bcd5a664547f9 = function() { return handleError(function (arg0) {
        const ret = cellUint32(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_charCodeAt_abe5953e37f4b5a6 = function(arg0, arg1) {
        const ret = arg0.charCodeAt(arg1 >>> 0);
        return ret;
    };
    imports.wbg.__wbg_checkSerializeArrayLength_949f3eadef0ff4d3 = function() { return handleError(function (arg0) {
        checkSerializeArrayLength(arg0);
    }, arguments) };
    imports.wbg.__wbg_childNodes_c4423003f3a9441f = function(arg0) {
        const ret = arg0.childNodes;
        return ret;
    };
    imports.wbg.__wbg_children_9b8d0778c0b363f8 = function(arg0) {
        const ret = arg0.children;
        return ret;
    };
    imports.wbg.__wbg_classList_3fa995ef71da9e8e = function(arg0) {
        const ret = arg0.classList;
        return ret;
    };
    imports.wbg.__wbg_className_c085027b30c195d3 = function(arg0, arg1) {
        const ret = arg1.className;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_clearFontFeatures_26531b8106412afe = function() { return handleError(function (arg0) {
        clearFontFeatures(arg0);
    }, arguments) };
    imports.wbg.__wbg_clearInterval_ad2594253cc39c4b = function(arg0, arg1) {
        arg0.clearInterval(arg1);
    };
    imports.wbg.__wbg_clearRect_8e4ba7ea0e06711a = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.clearRect(arg1, arg2, arg3, arg4);
    };
    imports.wbg.__wbg_clearTimeout_b2651b7485c58446 = function(arg0, arg1) {
        arg0.clearTimeout(arg1);
    };
    imports.wbg.__wbg_clear_fca8ee600282eba4 = function(arg0) {
        arg0.clear();
    };
    imports.wbg.__wbg_clientX_5eb380a5f1fec6fd = function(arg0) {
        const ret = arg0.clientX;
        return ret;
    };
    imports.wbg.__wbg_clientY_d8b9c7f0c4e2e677 = function(arg0) {
        const ret = arg0.clientY;
        return ret;
    };
    imports.wbg.__wbg_clipboardAppendByte_4382e7d3894b82df = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = clipboardAppendByte(arg0, arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_clipboardAtob_9d618562889660bf = function() { return handleError(function (arg0) {
        const ret = clipboardAtob(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_clipboardBtoa_d9ed437f52d5f3f0 = function() { return handleError(function (arg0) {
        const ret = clipboardBtoa(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_clipboardCopyByte_56d872860cdb9a3f = function() { return handleError(function (arg0, arg1, arg2) {
        clipboardCopyByte(arg0, arg1, arg2);
    }, arguments) };
    imports.wbg.__wbg_clipboardDecodeBytes_85c8e813d25bcca9 = function() { return handleError(function (arg0) {
        const ret = clipboardDecodeBytes(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_clipboardDecodeModern_219177568d1ea153 = function() { return handleError(function (arg0) {
        const ret = clipboardDecodeModern(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_clipboardDecode_7f9392ef427216a0 = function() { return handleError(function (arg0, arg1) {
        const ret = clipboardDecode(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_clipboardEncodeBytes_46c6da198ea258d2 = function() { return handleError(function (arg0) {
        const ret = clipboardEncodeBytes(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_clipboardEncode_6fba526d330b0e45 = function() { return handleError(function (arg0, arg1) {
        const ret = clipboardEncode(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_clipboardFallbackBytes_bf4e84d06d98b2f0 = function() { return handleError(function (arg0) {
        const ret = clipboardFallbackBytes(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_clipboardHasByte_c8a9f54ae97a8a2f = function() { return handleError(function (arg0, arg1) {
        const ret = clipboardHasByte(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_clipboardInvoke1_9de7e228b6d6d8d0 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = clipboardInvoke1(arg0, arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_clipboardInvoke2_5b9425cf809a4c21 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        const ret = clipboardInvoke2(arg0, arg1, arg2, arg3);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_clipboardPrevent_4e92799ee3d736ba = function() { return handleError(function (arg0) {
        clipboardPrevent(arg0);
    }, arguments) };
    imports.wbg.__wbg_clipboardPromise_be19fe4467c93cd5 = function() { return handleError(function (arg0) {
        const ret = clipboardPromise(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_clipboardProperty_bf8862ee6c0ee1b9 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = clipboardProperty(arg0, getStringFromWasm0(arg1, arg2));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_clipboardReadThen_f82b7d847cf25077 = function() { return handleError(function (arg0, arg1) {
        const ret = clipboardReadThen(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_clipboardRead_34d242e4f0a517b0 = function() { return handleError(function (arg0, arg1) {
        const ret = clipboardRead(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_clipboardStop_c0dc5535279408ab = function() { return handleError(function (arg0) {
        clipboardStop(arg0);
    }, arguments) };
    imports.wbg.__wbg_clipboardString_f4a5101167041a1e = function() { return handleError(function (arg0) {
        const ret = clipboardString(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_clipboardSubtract_3c7ad3aef1dcf6ab = function() { return handleError(function (arg0, arg1) {
        const ret = clipboardSubtract(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_clipboardWriteThen_da82366ba8289562 = function() { return handleError(function (arg0) {
        const ret = clipboardWriteThen(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_clipboardWrite_ce1761d8811b9e40 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = clipboardWrite(arg0, arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_collectDisposable_0a6d1b0ba6f860d5 = function() { return handleError(function (arg0, arg1) {
        collectDisposable(arg0, arg1);
    }, arguments) };
    imports.wbg.__wbg_compareDocumentPosition_e9dddbfce23cda8b = function(arg0, arg1) {
        const ret = arg0.compareDocumentPosition(arg1);
        return ret;
    };
    imports.wbg.__wbg_completeWrite_6f057a117c6aaa64 = function() { return handleError(function (arg0) {
        completeWrite(arg0);
    }, arguments) };
    imports.wbg.__wbg_composedPath_977ce97a0ef39358 = function(arg0) {
        const ret = arg0.composedPath();
        return ret;
    };
    imports.wbg.__wbg_concat_9de968491c4340cf = function(arg0, arg1) {
        const ret = arg0.concat(arg1);
        return ret;
    };
    imports.wbg.__wbg_construct_b91ff0e53b60c0c3 = function() { return handleError(function (arg0, arg1) {
        const ret = Reflect.construct(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_constructor_75c512bc262c1ab2 = function(arg0) {
        const ret = arg0.constructor;
        return ret;
    };
    imports.wbg.__wbg_contains_3361c7eda6c95afd = function(arg0, arg1) {
        const ret = arg0.contains(arg1);
        return ret;
    };
    imports.wbg.__wbg_contains_687eea5148ddb87c = function(arg0, arg1, arg2) {
        const ret = arg0.contains(getStringFromWasm0(arg1, arg2));
        return ret;
    };
    imports.wbg.__wbg_copyOptionsDefaults_0ecafc8d3f8e8add = function() { return handleError(function (arg0) {
        const ret = copyOptionsDefaults(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_coredisposalstore_new = function(arg0) {
        const ret = CoreDisposalStore.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_coremodeserviceview_new = function(arg0) {
        const ret = CoreModeServiceView.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_coreservices_new = function(arg0) {
        const ret = CoreServices.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_createDocumentFragment_42d904f69d1931e9 = function(arg0) {
        const ret = arg0.createDocumentFragment();
        return ret;
    };
    imports.wbg.__wbg_createElement_8c9931a732ee2fea = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.createElement(getStringFromWasm0(arg1, arg2));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_createEvent_fec3d2b0ce98814d = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.createEvent(getStringFromWasm0(arg1, arg2));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_createLinearGradient_d9f235a84f77b603 = function(arg0, arg1, arg2, arg3, arg4) {
        const ret = arg0.createLinearGradient(arg1, arg2, arg3, arg4);
        return ret;
    };
    imports.wbg.__wbg_data_d1ed736c1e42b10e = function(arg0, arg1) {
        const ret = arg1.data;
        const ptr1 = passArray8ToWasm0(ret, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_debugBytes_e4abfda576ddf40e = function() { return handleError(function (arg0) {
        const ret = debugBytes(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_debugEnabled_8809666607362290 = function() { return handleError(function (arg0) {
        const ret = debugEnabled(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_decprivatemodesview_new = function(arg0) {
        const ret = DecPrivateModesView.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_defaultView_51f191496dcfd54e = function(arg0) {
        const ret = arg0.defaultView;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_defineOption_c1696464c7e69851 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
        defineOption(arg0, getStringFromWasm0(arg1, arg2), arg3 !== 0, arg4, arg5);
    }, arguments) };
    imports.wbg.__wbg_delete_36c8630e530a2a1a = function(arg0, arg1) {
        const ret = arg0.delete(arg1);
        return ret;
    };
    imports.wbg.__wbg_deregisterJoiner_38ade75d734fdce6 = function() { return handleError(function (arg0, arg1) {
        const ret = deregisterJoiner(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_detail_8d2c47c0be6ddc7c = function(arg0) {
        const ret = arg0.detail;
        return ret;
    };
    imports.wbg.__wbg_dimensionsEvent_c35634a48fa5de1a = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
        const ret = dimensionsEvent(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_dimensionsView_76ad87e36558455a = function() { return handleError(function (arg0) {
        const ret = dimensionsView(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_disconnect_2118016d75479985 = function(arg0) {
        arg0.disconnect();
    };
    imports.wbg.__wbg_dispatchEvent_9e259d7c1d603dfb = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.dispatchEvent(arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_disposeAddon_fa182e2fec1012e1 = function() { return handleError(function (arg0) {
        disposeAddon(arg0);
    }, arguments) };
    imports.wbg.__wbg_disposeMarkerResource_f642e1c2888b070d = function() { return handleError(function (arg0) {
        disposeMarkerResource(arg0);
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
    imports.wbg.__wbg_error_524f506f44df1645 = function(arg0) {
        console.error(arg0);
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
    imports.wbg.__wbg_fastAssignStyle_92236c11e4d1acba = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        fastAssignStyle(arg0, getStringFromWasm0(arg1, arg2), arg3);
    }, arguments) };
    imports.wbg.__wbg_fastAttribute_24c93b0d6634ecfc = function() { return handleError(function (arg0, arg1, arg2) {
        fastAttribute(arg0, arg1, arg2);
    }, arguments) };
    imports.wbg.__wbg_fastBeginStyle_e90dc2ec70db6605 = function() { return handleError(function (arg0) {
        const ret = fastBeginStyle(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_fastCacheSet_ccf94dce0cfdf032 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        fastCacheSet(arg0, getStringFromWasm0(arg1, arg2), arg3);
    }, arguments) };
    imports.wbg.__wbg_fastCachedClass_6975ae36ad2a00b8 = function() { return handleError(function (arg0, arg1) {
        fastCachedClass(arg0, arg1);
    }, arguments) };
    imports.wbg.__wbg_fastCachedStyle_2a5b975d61b9d2ab = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        fastCachedStyle(arg0, getStringFromWasm0(arg1, arg2), arg3);
    }, arguments) };
    imports.wbg.__wbg_fastLayerHint_889e32733603fef0 = function() { return handleError(function (arg0, arg1) {
        fastLayerHint(arg0, arg1 !== 0);
    }, arguments) };
    imports.wbg.__wbg_fastToggle_66d2fdec62cf206e = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = fastToggle(arg0, arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_fillRect_c38d5d56492a2368 = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.fillRect(arg1, arg2, arg3, arg4);
    };
    imports.wbg.__wbg_fillStyle_8f56a83d79c226ec = function(arg0) {
        const ret = arg0.fillStyle;
        return ret;
    };
    imports.wbg.__wbg_fill_c938e65a1f487194 = function(arg0, arg1, arg2, arg3) {
        const ret = arg0.fill(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0);
        return ret;
    };
    imports.wbg.__wbg_finishFontLoad_1439aa5f49141067 = function() { return handleError(function (arg0, arg1, arg2) {
        finishFontLoad(arg0, arg1, arg2);
    }, arguments) };
    imports.wbg.__wbg_firstChild_b0603462b5172539 = function(arg0) {
        const ret = arg0.firstChild;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_focusNode_107fd18531714268 = function(arg0) {
        const ret = arg0.focusNode;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_focusOffset_df38301425e75cb5 = function(arg0) {
        const ret = arg0.focusOffset;
        return ret;
    };
    imports.wbg.__wbg_focus_1fcb7709d3a22023 = function() { return handleError(function (arg0, arg1) {
        arg0.focus(arg1);
    }, arguments) };
    imports.wbg.__wbg_focus_7d08b55eba7b368d = function() { return handleError(function (arg0) {
        arg0.focus();
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
                    return __wbg_adapter_1335(a, state0.b, arg0, arg1, arg2);
                } finally {
                    state0.a = a;
                }
            };
            arg0.forEach(cb0);
        } finally {
            state0.a = state0.b = 0;
        }
    };
    imports.wbg.__wbg_freeze_ef6d70cf38e8d948 = function(arg0) {
        const ret = Object.freeze(arg0);
        return ret;
    };
    imports.wbg.__wbg_fromCharCode_5493f9b25e1ce0a1 = function(arg0, arg1) {
        const ret = String.fromCharCode(...getArrayU16FromWasm0(arg0, arg1));
        return ret;
    };
    imports.wbg.__wbg_fromEntries_524679eecb0bdc2e = function() { return handleError(function (arg0) {
        const ret = Object.fromEntries(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_from_2a5d3e218e67aa85 = function(arg0) {
        const ret = Array.from(arg0);
        return ret;
    };
    imports.wbg.__wbg_getAttribute_ea5166be2deba45e = function(arg0, arg1, arg2, arg3) {
        const ret = arg1.getAttribute(getStringFromWasm0(arg2, arg3));
        var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_getBoundingClientRect_9073b0ff7574d76b = function(arg0) {
        const ret = arg0.getBoundingClientRect();
        return ret;
    };
    imports.wbg.__wbg_getComputedStyle_046dd6472f8e7f1d = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.getComputedStyle(arg1);
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    }, arguments) };
    imports.wbg.__wbg_getContext_e9cf379449413580 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.getContext(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    }, arguments) };
    imports.wbg.__wbg_getContext_f65a0debd1e8f8e8 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.getContext(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    }, arguments) };
    imports.wbg.__wbg_getContext_fc19859df6331073 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        const ret = arg0.getContext(getStringFromWasm0(arg1, arg2), arg3);
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    }, arguments) };
    imports.wbg.__wbg_getElementById_727e92dd724c370c = function(arg0, arg1, arg2) {
        const ret = arg0.getElementById(getStringFromWasm0(arg1, arg2));
        return ret;
    };
    imports.wbg.__wbg_getImageData_c02374a30b126dab = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        const ret = arg0.getImageData(arg1, arg2, arg3, arg4);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_getPropertyValue_e623c23a05dfb30c = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        const ret = arg1.getPropertyValue(getStringFromWasm0(arg2, arg3));
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_getSelection_186393c62b575af5 = function() { return handleError(function (arg0) {
        const ret = arg0.getSelection();
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    }, arguments) };
    imports.wbg.__wbg_get_0b9dd635dc599246 = function(arg0, arg1) {
        const ret = arg0.get(arg1);
        return ret;
    };
    imports.wbg.__wbg_get_13495dac72693ecc = function(arg0, arg1) {
        const ret = arg0.get(arg1);
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
    imports.wbg.__wbg_has_76ca66e2f25d1c49 = function(arg0, arg1) {
        const ret = arg0.has(arg1);
        return ret;
    };
    imports.wbg.__wbg_has_7984823c9e23a04a = function(arg0, arg1) {
        const ret = arg0.has(arg1);
        return ret;
    };
    imports.wbg.__wbg_has_a5ea9117f258a0ec = function() { return handleError(function (arg0, arg1) {
        const ret = Reflect.has(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_height_592a89ec0fb63726 = function(arg0) {
        const ret = arg0.height;
        return ret;
    };
    imports.wbg.__wbg_height_838cee19ba8597db = function(arg0) {
        const ret = arg0.height;
        return ret;
    };
    imports.wbg.__wbg_host_1fb122db1f00d55f = function(arg0, arg1) {
        const ret = arg1.host;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
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
    imports.wbg.__wbg_imageAddonOptions_e927d3f84a9ccec7 = function() { return handleError(function (arg0) {
        const ret = imageAddonOptions(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_imageHandlerInspection_274a5c99051d7f4f = function() { return handleError(function (arg0) {
        const ret = imageHandlerInspection(ImageHandlersReader.__wrap(arg0));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_imageOptionInspection_c1946cff6f19d6ab = function() { return handleError(function (arg0) {
        const ret = imageOptionInspection(ImageOptionsReader.__wrap(arg0));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_imageOptionNumber_cb0849d7b54bc152 = function() { return handleError(function (arg0) {
        const ret = imageOptionNumber(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_imageStorageInitialPixels_7da8f61a3d24bc75 = function() { return handleError(function (arg0) {
        const ret = imageStorageInitialPixels(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_imageStorageInspection_0b2efa7028a3eba6 = function() { return handleError(function (arg0) {
        const ret = imageStorageInspection(ImageStoreReader.__wrap(arg0));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_imageStoragePixels_4c046937d6bf592b = function() { return handleError(function (arg0) {
        const ret = imageStoragePixels(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_imagehandlerinspection_new = function(arg0) {
        const ret = ImageHandlerInspection.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_imagespecinspection_new = function(arg0) {
        const ret = ImageSpecInspection.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_initializeOptions_f538461b14dfade1 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = initializeOptions(arg0, arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_inputProperty_e774659713e6ae7c = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = inputProperty(arg0, getStringFromWasm0(arg1, arg2));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_insertBefore_c181fb91844cd959 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.insertBefore(arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_installAddonDispose_623b2bb05695546e = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        installAddonDispose(arg0, arg1, arg2, arg3);
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
    imports.wbg.__wbg_instanceof_Document_917b7ac52e42682e = function(arg0) {
        let result;
        try {
            result = arg0 instanceof Document;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_Element_0af65443936d5154 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof Element;
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
    imports.wbg.__wbg_instanceof_HtmlElement_51378c201250b16c = function(arg0) {
        let result;
        try {
            result = arg0 instanceof HTMLElement;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_MouseEvent_ea92df42ebd8c1f9 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof MouseEvent;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_Node_fbc6b87f5ed2e230 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof Node;
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
    imports.wbg.__wbg_instanceof_Text_d4b85616d699cb19 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof Text;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_Uint8Array_17156bcf118086a9 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof Uint8Array;
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
    imports.wbg.__wbg_internalbufferserviceview_new = function(arg0) {
        const ret = InternalBufferServiceView.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_internalbufferview_new = function(arg0) {
        const ret = InternalBufferView.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_internalrenderserviceview_new = function(arg0) {
        const ret = InternalRenderServiceView.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_isArray_a1eab7e0d067391b = function(arg0) {
        const ret = Array.isArray(arg0);
        return ret;
    };
    imports.wbg.__wbg_isCollapsed_6ed2f2b94b285dd1 = function(arg0) {
        const ret = arg0.isCollapsed;
        return ret;
    };
    imports.wbg.__wbg_isSameNode_2a012c6251fe238d = function(arg0, arg1) {
        const ret = arg0.isSameNode(arg1);
        return ret;
    };
    imports.wbg.__wbg_is_c7481c65e7e5df9e = function(arg0, arg1) {
        const ret = Object.is(arg0, arg1);
        return ret;
    };
    imports.wbg.__wbg_item_8be407c958853a13 = function(arg0, arg1) {
        const ret = arg0.item(arg1 >>> 0);
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_iterator_9a24c88df860dc65 = function() {
        const ret = Symbol.iterator;
        return ret;
    };
    imports.wbg.__wbg_join_f9ac4707dbaaa63e = function(arg0, arg1, arg2) {
        const ret = arg0.join(getStringFromWasm0(arg1, arg2));
        return ret;
    };
    imports.wbg.__wbg_joinerCall_66e802737b0795c4 = function() { return handleError(function (arg0, arg1) {
        const ret = joinerCall(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_joinerGet_284da78f5c519d21 = function() { return handleError(function (arg0, arg1) {
        const ret = joinerGet(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_joinerInsert_67aaf0d89b9b768e = function() { return handleError(function (arg0, arg1, arg2) {
        joinerInsert(arg0, arg1, arg2);
    }, arguments) };
    imports.wbg.__wbg_joinerLessEqual_184a56448cc774a0 = function() { return handleError(function (arg0, arg1) {
        const ret = joinerLessEqual(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_joinerLess_261b4f0fc9ed035a = function() { return handleError(function (arg0, arg1) {
        const ret = joinerLess(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_joinerMaxEnd_c5eb949fef36da0f = function() { return handleError(function (arg0, arg1) {
        const ret = joinerMaxEnd(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_joinerMinStart_1bc35e3e667ee456 = function() { return handleError(function (arg0, arg1) {
        const ret = joinerMinStart(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_joinerPushFrom_a90374293d18c616 = function() { return handleError(function (arg0, arg1, arg2) {
        joinerPushFrom(arg0, arg1, arg2);
    }, arguments) };
    imports.wbg.__wbg_joinerPush_a08c534b209adcf7 = function() { return handleError(function (arg0, arg1) {
        joinerPush(arg0, arg1);
    }, arguments) };
    imports.wbg.__wbg_joinerRemove_b77aebdf925bb62c = function() { return handleError(function (arg0, arg1) {
        joinerRemove(arg0, arg1);
    }, arguments) };
    imports.wbg.__wbg_joinerReport_9b5e13566673637e = function() { return handleError(function (arg0) {
        joinerReport(arg0);
    }, arguments) };
    imports.wbg.__wbg_joinerSet_49846dcc3dbba203 = function() { return handleError(function (arg0, arg1, arg2) {
        joinerSet(arg0, arg1, arg2);
    }, arguments) };
    imports.wbg.__wbg_joinerSubtractOne_0f234c8d063e9801 = function() { return handleError(function (arg0) {
        const ret = joinerSubtractOne(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_keyEvent_d90e2188ef5b95e3 = function() { return handleError(function (arg0, arg1) {
        const ret = keyEvent(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_keyboardDefaultText_c12fcaadfa9b8d44 = function() { return handleError(function (arg0) {
        const ret = keyboardDefaultText(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_keys_4e7df9a04572b339 = function(arg0) {
        const ret = arg0.keys();
        return ret;
    };
    imports.wbg.__wbg_kittyHandlerInspection_5608d9c8350aaab2 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = kittyHandlerInspection(KittyImagesReader.__wrap(arg0), KittyPendingReader.__wrap(arg1), KittyPlacementsReader.__wrap(arg2));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_kittyimageinspection_new = function(arg0) {
        const ret = KittyImageInspection.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_left_e46801720267b66d = function(arg0) {
        const ret = arg0.left;
        return ret;
    };
    imports.wbg.__wbg_length_347907d14a9ed873 = function(arg0) {
        const ret = arg0.length;
        return ret;
    };
    imports.wbg.__wbg_length_6ca527665d89694d = function(arg0) {
        const ret = arg0.length;
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
    imports.wbg.__wbg_linkAdd_9aae1367cf35c90f = function() { return handleError(function (arg0, arg1) {
        const ret = linkAdd(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_linkCall_9598b5238acea016 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = linkCall(arg0, arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_linkDescriptors_3d103ad834fa9ecc = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
        linkDescriptors(arg0, arg1, arg2, arg3, arg4, arg5);
    }, arguments) };
    imports.wbg.__wbg_linkIncrement_84ae53ee53f67de6 = function() { return handleError(function (arg0) {
        const ret = linkIncrement(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_linkLessEqual_d0759869fe77649e = function() { return handleError(function (arg0, arg1) {
        const ret = linkLessEqual(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_linkLess_8a3116911ee9fafb = function() { return handleError(function (arg0, arg1) {
        const ret = linkLess(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_linkMap_12c671212c36d76f = function() { return handleError(function (arg0) {
        const ret = linkMap(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_linkMultiply_5c272346063cadc8 = function() { return handleError(function (arg0, arg1) {
        const ret = linkMultiply(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_linkProperty_aaf8fc2a407b10b1 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = linkProperty(arg0, getStringFromWasm0(arg1, arg2));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_linkSet_2ab87826270f2ee1 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        linkSet(arg0, getStringFromWasm0(arg1, arg2), arg3);
    }, arguments) };
    imports.wbg.__wbg_linkSubtract_36eddfe7ad962699 = function() { return handleError(function (arg0, arg1) {
        const ret = linkSubtract(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_liveWheelReceiver_5cb28ff243cf7bff = function() { return handleError(function (arg0) {
        const ret = liveWheelReceiver(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_load_01de601270b5b817 = function() { return handleError(function (arg0) {
        const ret = arg0.load();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_markerPosition_76c5605330997ccb = function() { return handleError(function (arg0, arg1) {
        const ret = markerPosition(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_marker_new = function(arg0) {
        const ret = Marker.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_maxTouchPoints_763ee20e665a4536 = function(arg0) {
        const ret = arg0.maxTouchPoints;
        return ret;
    };
    imports.wbg.__wbg_message_97a2af9b89d693a3 = function(arg0) {
        const ret = arg0.message;
        return ret;
    };
    imports.wbg.__wbg_missingTouchRecord_6ef767ec6e982151 = function() { return handleError(function (arg0) {
        const ret = missingTouchRecord(arg0 !== 0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_modesObject_bb547d9b21eca3e2 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11, arg12) {
        const ret = modesObject(arg0 !== 0, arg1 !== 0, arg2 !== 0, arg3 !== 0, getStringFromWasm0(arg4, arg5), arg6 !== 0, arg7 !== 0, arg8 !== 0, arg9 !== 0, arg10 !== 0, arg11 !== 0, arg12 !== 0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_monitorReleaseCapture_c6baf1069d6aef7c = function() { return handleError(function (arg0, arg1) {
        monitorReleaseCapture(arg0, arg1);
    }, arguments) };
    imports.wbg.__wbg_monitorSetCapture_94998814db31720b = function() { return handleError(function (arg0, arg1) {
        monitorSetCapture(arg0, arg1);
    }, arguments) };
    imports.wbg.__wbg_monitorWindow_78258e597d18ca6e = function() { return handleError(function (arg0) {
        const ret = monitorWindow(arg0);
        return ret;
    }, arguments) };
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
    imports.wbg.__wbg_navigator_1577371c070c8947 = function(arg0) {
        const ret = arg0.navigator;
        return ret;
    };
    imports.wbg.__wbg_new_1ab78df5e132f715 = function(arg0, arg1) {
        const ret = new RangeError(getStringFromWasm0(arg0, arg1));
        return ret;
    };
    imports.wbg.__wbg_new_215d9bc2d34af331 = function(arg0, arg1) {
        const ret = new WebAssembly.RuntimeError(getStringFromWasm0(arg0, arg1));
        return ret;
    };
    imports.wbg.__wbg_new_23a2665fac83c611 = function(arg0, arg1) {
        try {
            var state0 = {a: arg0, b: arg1};
            var cb0 = (arg0, arg1) => {
                const a = state0.a;
                state0.a = 0;
                try {
                    return __wbg_adapter_1446(a, state0.b, arg0, arg1);
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
    imports.wbg.__wbg_new_2ef971087cb43792 = function() { return handleError(function (arg0, arg1) {
        const ret = new OffscreenCanvas(arg0 >>> 0, arg1 >>> 0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_new_405e22f390576ce2 = function() {
        const ret = new Object();
        return ret;
    };
    imports.wbg.__wbg_new_49d1d36ae5bd514a = function(arg0, arg1) {
        const ret = new WebAssembly.LinkError(getStringFromWasm0(arg0, arg1));
        return ret;
    };
    imports.wbg.__wbg_new_5e0be73521bc8c17 = function() {
        const ret = new Map();
        return ret;
    };
    imports.wbg.__wbg_new_61dc378c2412d7e1 = function(arg0, arg1) {
        const ret = new ReferenceError(getStringFromWasm0(arg0, arg1));
        return ret;
    };
    imports.wbg.__wbg_new_6ef6888614f8a2cc = function(arg0, arg1) {
        const ret = new SyntaxError(getStringFromWasm0(arg0, arg1));
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
    imports.wbg.__wbg_new_9ffbe0a71eff35e3 = function() { return handleError(function (arg0, arg1) {
        const ret = new URL(getStringFromWasm0(arg0, arg1));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_new_a12002a7f91c75be = function(arg0) {
        const ret = new Uint8Array(arg0);
        return ret;
    };
    imports.wbg.__wbg_new_a239edaa1dc2968f = function(arg0) {
        const ret = new Set(arg0);
        return ret;
    };
    imports.wbg.__wbg_new_aa9c15785d2aa302 = function() {
        const ret = new WeakMap();
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
    imports.wbg.__wbg_now_807e54c39636c349 = function() {
        const ret = Date.now();
        return ret;
    };
    imports.wbg.__wbg_now_832024c17817dc01 = function(arg0) {
        const ret = arg0.now();
        return ret;
    };
    imports.wbg.__wbg_now_d18023d54d4e5500 = function(arg0) {
        const ret = arg0.now();
        return ret;
    };
    imports.wbg.__wbg_observe_eafddfc5a0c60e02 = function(arg0, arg1) {
        arg0.observe(arg1);
    };
    imports.wbg.__wbg_of_1483b62f6e3f9868 = function(arg0, arg1, arg2, arg3) {
        const ret = Array.of(arg0, arg1, arg2, arg3);
        return ret;
    };
    imports.wbg.__wbg_of_2eaf5a02d443ef03 = function(arg0) {
        const ret = Array.of(arg0);
        return ret;
    };
    imports.wbg.__wbg_of_4a05197bfc89556f = function(arg0, arg1, arg2) {
        const ret = Array.of(arg0, arg1, arg2);
        return ret;
    };
    imports.wbg.__wbg_of_66b3ee656cbd962b = function(arg0, arg1) {
        const ret = Array.of(arg0, arg1);
        return ret;
    };
    imports.wbg.__wbg_open_a9f9f906ba53e934 = function() { return handleError(function (arg0) {
        const ret = arg0.open();
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    }, arguments) };
    imports.wbg.__wbg_optionAdd_caf9ec0ae01f4ea9 = function() { return handleError(function (arg0, arg1) {
        const ret = optionAdd(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_optionConcat_e03d3589e213eb64 = function() { return handleError(function (arg0) {
        const ret = optionConcat(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_optionData_ab8b2ca6f121fc37 = function() { return handleError(function (arg0, arg1, arg2) {
        optionData(arg0, arg1, arg2);
    }, arguments) };
    imports.wbg.__wbg_optionError_0e95b5f6adbacda6 = function(arg0, arg1) {
        const ret = optionError(arg0, arg1 !== 0);
        return ret;
    };
    imports.wbg.__wbg_optionLayout_76c0c7d10ecc239a = function() { return handleError(function (arg0) {
        const ret = optionLayout(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_optionLessEqual_c986af6aa8e6524a = function() { return handleError(function (arg0, arg1) {
        const ret = optionLessEqual(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_optionLess_6f3fdb777da9ccf8 = function() { return handleError(function (arg0, arg1) {
        const ret = optionLess(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_optionMembership_7c81a7d2cec86bd5 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = optionMembership(arg0, getStringFromWasm0(arg1, arg2));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_optionMultiply_cd6ac28d28718524 = function() { return handleError(function (arg0, arg1) {
        const ret = optionMultiply(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_optionNumber_4412ba873da650f5 = function() { return handleError(function (arg0) {
        const ret = optionNumber(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_optionProperty_dcf16c43d7021b87 = function() { return handleError(function (arg0, arg1) {
        const ret = optionProperty(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_optionString_43ba64c9a833a9d1 = function() { return handleError(function (arg0) {
        const ret = optionString(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_oscBrowserLink_65133b12fe45abc3 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        const ret = oscBrowserLink(arg0, arg1, arg2, arg3);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_ownerDocument_243e06ce3350b842 = function(arg0) {
        const ret = arg0.ownerDocument;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_pairEvent_9dd22a628636a60a = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
        const ret = pairEvent(getStringFromWasm0(arg0, arg1), arg2, getStringFromWasm0(arg3, arg4), arg5);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_parentElement_be28a1a931f9c9b7 = function(arg0) {
        const ret = arg0.parentElement;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_parentNode_9de97a0e7973ea4e = function(arg0) {
        const ret = arg0.parentNode;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_parserapi_new = function(arg0) {
        const ret = ParserApi.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_password_5a1026040932bbb5 = function(arg0, arg1) {
        const ret = arg1.password;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_performance_ae5e84dd26af6765 = function(arg0) {
        const ret = arg0.performance;
        return ret;
    };
    imports.wbg.__wbg_performance_c185c0cdc2766575 = function(arg0) {
        const ret = arg0.performance;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_platform_faf02c487289f206 = function() { return handleError(function (arg0, arg1) {
        const ret = arg1.platform;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_pngBitmapFallback_3de80dcc650adbd2 = function() { return handleError(function (arg0) {
        const ret = pngBitmapFallback(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_prepareLinkRegExp_e221906e5bd70189 = function() { return handleError(function (arg0) {
        const ret = prepareLinkRegExp(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_preventDefault_c2314fd813c02b3c = function(arg0) {
        arg0.preventDefault();
    };
    imports.wbg.__wbg_protocol_46b13974dabb4d0c = function(arg0, arg1) {
        const ret = arg1.protocol;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_publicOptions_185f0a6f03364edb = function() { return handleError(function (arg0, arg1) {
        const ret = publicOptions(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_pushAddonEntry_4443165dcfc8478e = function() { return handleError(function (arg0, arg1) {
        pushAddonEntry(arg0, arg1);
    }, arguments) };
    imports.wbg.__wbg_push_737cfc8c1432c2c6 = function(arg0, arg1) {
        const ret = arg0.push(arg1);
        return ret;
    };
    imports.wbg.__wbg_queueDecorationSync_5b5ac3ae5f880034 = function() { return handleError(function (arg0) {
        queueDecorationSync(arg0);
    }, arguments) };
    imports.wbg.__wbg_queueFocusCache_bae6690d3e85d69d = function() { return handleError(function (arg0) {
        queueFocusCache(arg0);
    }, arguments) };
    imports.wbg.__wbg_queueMicrotask_97d92b4fcc8a61c5 = function(arg0) {
        queueMicrotask(arg0);
    };
    imports.wbg.__wbg_queueMicrotask_d3219def82552485 = function(arg0) {
        const ret = arg0.queueMicrotask;
        return ret;
    };
    imports.wbg.__wbg_rangeBytes_768c9a003fff5181 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        const ret = rangeBytes(arg1, arg2 >>> 0, arg3 >>> 0);
        const ptr1 = passArray8ToWasm0(ret, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_readAddonOwner_9a91332f41638ce2 = function() { return handleError(function (arg0) {
        const ret = readAddonOwner(arg0);
        return ret;
    }, arguments) };
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
    imports.wbg.__wbg_relatedTarget_d245b4078428fc22 = function(arg0) {
        const ret = arg0.relatedTarget;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_removeAddon_0b57b680edf0a3ab = function() { return handleError(function (arg0, arg1) {
        removeAddon(arg0, arg1 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_removeChild_841bf1dc802c0a2c = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.removeChild(arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_removeEventListener_056dfe8c3d6c58f9 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        arg0.removeEventListener(getStringFromWasm0(arg1, arg2), arg3);
    }, arguments) };
    imports.wbg.__wbg_removeEventListener_d365ee1c2a7b08f0 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        arg0.removeEventListener(getStringFromWasm0(arg1, arg2), arg3, arg4 !== 0);
    }, arguments) };
    imports.wbg.__wbg_remove_282d941ca37d0c63 = function() { return handleError(function (arg0, arg1, arg2) {
        arg0.remove(getStringFromWasm0(arg1, arg2));
    }, arguments) };
    imports.wbg.__wbg_remove_e2d2659f3128c045 = function(arg0) {
        arg0.remove();
    };
    imports.wbg.__wbg_requestAnimationFrame_d7fd890aaefc3246 = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.requestAnimationFrame(arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_requestIdleCallback_e3eefd34962470e1 = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.requestIdleCallback(arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_requestWrite_d8d5c484ebdd100d = function() { return handleError(function (arg0) {
        const ret = requestWrite(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_requireQueueRuntime_555a0ddd636e3add = function() { return handleError(function () {
        requireQueueRuntime();
    }, arguments) };
    imports.wbg.__wbg_resolve_4851785c9c5f573d = function(arg0) {
        const ret = Promise.resolve(arg0);
        return ret;
    };
    imports.wbg.__wbg_rowAdd_7344b431ebbf2f12 = function() { return handleError(function (arg0, arg1) {
        const ret = rowAdd(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_rowAddress_d47cb3fd5ce3dd5d = function() { return handleError(function (arg0) {
        const ret = rowAddress(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_rowGet_d35c63ad896e6c4d = function() { return handleError(function (arg0, arg1) {
        const ret = rowGet(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_rowIncrement_74aa4734a68e7b70 = function() { return handleError(function (arg0) {
        const ret = rowIncrement(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_rowLessEqual_184a56448cc774a0 = function() { return handleError(function (arg0, arg1) {
        const ret = rowLessEqual(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_rowLess_261b4f0fc9ed035a = function() { return handleError(function (arg0, arg1) {
        const ret = rowLess(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_rowMin_b7c4f7991e2b0544 = function() { return handleError(function (arg0, arg1) {
        const ret = rowMin(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_rowMultiply_13e7c9b1674ee512 = function() { return handleError(function (arg0, arg1) {
        const ret = rowMultiply(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_rowShift_42713df95b452231 = function() { return handleError(function (arg0) {
        const ret = rowShift(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_rowSubtractOne_0f234c8d063e9801 = function() { return handleError(function (arg0) {
        const ret = rowSubtractOne(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_rowSubtract_c48c20ae4cdfc317 = function() { return handleError(function (arg0, arg1) {
        const ret = rowSubtract(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_scrollChainEntry_72f7b86815aa672e = function() { return handleError(function (arg0, arg1) {
        const ret = scrollChainEntry(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_scrollClock_f78d225ad762995b = function() { return handleError(function () {
        const ret = scrollClock();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_scrollDivide_2f31e45a3198e562 = function() { return handleError(function (arg0, arg1) {
        const ret = scrollDivide(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_scrollGet_33345298e67defd2 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = scrollGet(arg0, getStringFromWasm0(arg1, arg2));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_scrollMethod0_788b9e190b41191d = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = scrollMethod0(arg0, getStringFromWasm0(arg1, arg2));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_scrollMethod1_497841f6ce0635d1 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        const ret = scrollMethod1(arg0, getStringFromWasm0(arg1, arg2), arg3);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_scrollMethod2_157e231492a10048 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        const ret = scrollMethod2(arg0, getStringFromWasm0(arg1, arg2), arg3, arg4);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_scrollMultiply_0b9bd00029daf5d5 = function() { return handleError(function (arg0, arg1) {
        const ret = scrollMultiply(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_scrollNegate_2a308325d9eb983f = function() { return handleError(function (arg0) {
        const ret = scrollNegate(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_scrollSet_19bbd38bd4d740c3 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        scrollSet(arg0, getStringFromWasm0(arg1, arg2), arg3);
    }, arguments) };
    imports.wbg.__wbg_scrollSubtract_68ba12a38ffc90b2 = function() { return handleError(function (arg0, arg1) {
        const ret = scrollSubtract(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_scrollWeakWindow_e74fbe8638e94967 = function() { return handleError(function (arg0) {
        const ret = scrollWeakWindow(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_scrollX_02fa7e84cc48b8fe = function() { return handleError(function (arg0) {
        const ret = arg0.scrollX;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_scrollY_94bf061418186bb3 = function() { return handleError(function (arg0) {
        const ret = arg0.scrollY;
        return ret;
    }, arguments) };
    imports.wbg.__wbg_searchAddonFlag_c10d9ac8384b0f7b = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = searchAddonFlag(arg0, getStringFromWasm0(arg1, arg2));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_searchAddonGet_e774659713e6ae7c = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = searchAddonGet(arg0, getStringFromWasm0(arg1, arg2));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_searchAddonLimitEnd_7057ee3f55c9894b = function() { return handleError(function (arg0, arg1) {
        const ret = searchAddonLimitEnd(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_searchAddonLimitReached_76f033cd2c3bab6e = function() { return handleError(function (arg0, arg1) {
        const ret = searchAddonLimitReached(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_searchAddonRefresh_67881bff4fc43b2f = function() { return handleError(function (arg0) {
        const ret = searchAddonRefresh(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_searchAddonRegex_7bbcc027c23aa4f3 = function() { return handleError(function (arg0) {
        const ret = searchAddonRegex(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_searchAddonValidTerm_9d4f4773e6fe0811 = function() { return handleError(function (arg0) {
        const ret = searchAddonValidTerm(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_searchsubscription_new = function(arg0) {
        const ret = SearchSubscription.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_select_a6f799366a79c31b = function(arg0) {
        arg0.select();
    };
    imports.wbg.__wbg_selectionEnd_a88c1e30434af1b1 = function() { return handleError(function (arg0) {
        const ret = arg0.selectionEnd;
        return isLikeNone(ret) ? 0x100000001 : (ret) >>> 0;
    }, arguments) };
    imports.wbg.__wbg_selectionStart_dc44e18e14639dcd = function() { return handleError(function (arg0) {
        const ret = arg0.selectionStart;
        return isLikeNone(ret) ? 0x100000001 : (ret) >>> 0;
    }, arguments) };
    imports.wbg.__wbg_self_67c28212685dfde2 = function(arg0) {
        const ret = arg0.self;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_serializeMissingCell_8b48c72015608c57 = function() { return handleError(function (arg0, arg1) {
        serializeMissingCell(arg0, arg1 >>> 0);
    }, arguments) };
    imports.wbg.__wbg_setAttribute_2704501201f15687 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        arg0.setAttribute(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
    }, arguments) };
    imports.wbg.__wbg_setFontFeatures_0a62be3e26b5837f = function() { return handleError(function (arg0, arg1) {
        setFontFeatures(arg0, arg1);
    }, arguments) };
    imports.wbg.__wbg_setInterval_cfbb32b46c873db2 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.setInterval(arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_setProperty_f2cf326652b9a713 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        arg0.setProperty(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
    }, arguments) };
    imports.wbg.__wbg_setTimeout_461fec76662b35ea = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.setTimeout(arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_setTimeout_f2fe5af8e3debeb3 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.setTimeout(arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_set_17060c045911830c = function(arg0, arg1, arg2) {
        const ret = arg0.set(arg1, arg2);
        return ret;
    };
    imports.wbg.__wbg_set_65595bdd868b3009 = function(arg0, arg1, arg2) {
        arg0.set(arg1, arg2 >>> 0);
    };
    imports.wbg.__wbg_set_8fc6bf8a5b1071d1 = function(arg0, arg1, arg2) {
        const ret = arg0.set(arg1, arg2);
        return ret;
    };
    imports.wbg.__wbg_set_bb8cecf6a62b9f46 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = Reflect.set(arg0, arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_setcapture_46bd7043887eba02 = function(arg0, arg1) {
        arg0.capture = arg1 !== 0;
    };
    imports.wbg.__wbg_setclassName_6bdd0705e646d028 = function(arg0, arg1, arg2) {
        arg0.className = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_setdir_b4e481c92bdd1ab4 = function(arg0, arg1, arg2) {
        arg0.dir = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_setfillStyle_42e38d3d485e9a92 = function(arg0, arg1) {
        arg0.fillStyle = arg1;
    };
    imports.wbg.__wbg_setglobalCompositeOperation_9a7a92bac2fb7ffd = function() { return handleError(function (arg0, arg1, arg2) {
        arg0.globalCompositeOperation = getStringFromWasm0(arg1, arg2);
    }, arguments) };
    imports.wbg.__wbg_setheight_da683a33fa99843c = function(arg0, arg1) {
        arg0.height = arg1 >>> 0;
    };
    imports.wbg.__wbg_setindex_c430b78b97744fcc = function(arg0, arg1, arg2) {
        arg0[arg1 >>> 0] = arg2 >>> 0;
    };
    imports.wbg.__wbg_setlineWidth_ec730c524f09baa9 = function(arg0, arg1) {
        arg0.lineWidth = arg1;
    };
    imports.wbg.__wbg_setpassive_57a5a4c4b00a7c62 = function(arg0, arg1) {
        arg0.passive = arg1 !== 0;
    };
    imports.wbg.__wbg_setpreventscroll_430f83f49dc6f2f6 = function(arg0, arg1) {
        arg0.preventScroll = arg1 !== 0;
    };
    imports.wbg.__wbg_setreadOnly_e7a89c3f2e846f51 = function(arg0, arg1) {
        arg0.readOnly = arg1 !== 0;
    };
    imports.wbg.__wbg_settabIndex_31adfec3c7eafbce = function(arg0, arg1) {
        arg0.tabIndex = arg1;
    };
    imports.wbg.__wbg_settextContent_d29397f7b994d314 = function(arg0, arg1, arg2) {
        arg0.textContent = arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_settextcontent_4516425a935f461b = function(arg0, arg1, arg2) {
        arg0.textContent = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_setvalue_08d17a42e5d5069d = function(arg0, arg1, arg2) {
        arg0.value = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_setwidth_c5fed9f5e7f0b406 = function(arg0, arg1) {
        arg0.width = arg1 >>> 0;
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
    imports.wbg.__wbg_shiftKey_2bebb3b703254f47 = function(arg0) {
        const ret = arg0.shiftKey;
        return ret;
    };
    imports.wbg.__wbg_shiftKey_86e737105bab1a54 = function(arg0) {
        const ret = arg0.shiftKey;
        return ret;
    };
    imports.wbg.__wbg_sixelTrapText_0fbe2eddb3c39502 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = sixelTrapText(getStringFromWasm0(arg1, arg2));
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
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
    imports.wbg.__wbg_statusClearPrefix_afd7957021c042f3 = function() { return handleError(function (arg0, arg1) {
        const ret = statusClearPrefix(arg1);
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_statusLess_9b29fdcd6e0a99fd = function() { return handleError(function (arg0, arg1) {
        const ret = statusLess(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_statusNumber_cb0849d7b54bc152 = function() { return handleError(function (arg0) {
        const ret = statusNumber(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_statusProperty_e774659713e6ae7c = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = statusProperty(arg0, getStringFromWasm0(arg1, arg2));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_stopImmediatePropagation_b7f329ceca9e5cb4 = function(arg0) {
        arg0.stopImmediatePropagation();
    };
    imports.wbg.__wbg_stopPropagation_11d220a858e5e0fb = function(arg0) {
        arg0.stopPropagation();
    };
    imports.wbg.__wbg_stringify_f7ed6987935b4a24 = function() { return handleError(function (arg0) {
        const ret = JSON.stringify(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_style_fb30c14e5815805c = function(arg0) {
        const ret = arg0.style;
        return ret;
    };
    imports.wbg.__wbg_target_0a62d9d79a2a1ede = function(arg0) {
        const ret = arg0.target;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_terminalElement_e84eede36bf31d0b = function() { return handleError(function (arg0) {
        const ret = terminalElement(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_terminal_new = function(arg0) {
        const ret = Terminal.__wrap(arg0);
        return ret;
    };
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
    imports.wbg.__wbg_timeRemaining_7a2ae8c12c9cc9a5 = function(arg0) {
        const ret = arg0.timeRemaining();
        return ret;
    };
    imports.wbg.__wbg_timeStamp_77f4ec8b6669253c = function(arg0) {
        const ret = arg0.timeStamp;
        return ret;
    };
    imports.wbg.__wbg_toFixed_43af7895cf202c17 = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.toFixed(arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_toLocaleLowerCase_a2b32231c0146e55 = function(arg0, arg1, arg2) {
        const ret = arg0.toLocaleLowerCase(arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2));
        return ret;
    };
    imports.wbg.__wbg_toggle_b764823ee3836d0d = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        const ret = arg0.toggle(getStringFromWasm0(arg1, arg2), arg3 !== 0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_top_ec9fceb1f030f2ea = function(arg0) {
        const ret = arg0.top;
        return ret;
    };
    imports.wbg.__wbg_touchCall1_24ab6e4e1e1f179d = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = touchCall1(arg0, arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_touchField_15f4dd3926aaf8d6 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = touchField(arg0, getStringFromWasm0(arg1, arg2));
        return ret;
    }, arguments) };
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
    imports.wbg.__wbg_unicodeservice_new = function(arg0) {
        const ret = UnicodeService.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_userAgent_12e9d8e62297563f = function() { return handleError(function (arg0, arg1) {
        const ret = arg1.userAgent;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    }, arguments) };
    imports.wbg.__wbg_username_882e4d53f03b6565 = function(arg0, arg1) {
        const ret = arg1.username;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_validateMarkerOffset_95b25c148c39eabd = function() { return handleError(function (arg0) {
        const ret = validateMarkerOffset(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_value_cd1ffa7b1ab794f1 = function(arg0) {
        const ret = arg0.value;
        return ret;
    };
    imports.wbg.__wbg_values_623d00ad3703dfaa = function(arg0) {
        const ret = arg0.values();
        return ret;
    };
    imports.wbg.__wbg_view_aaaec521dcbdac51 = function(arg0) {
        const ret = arg0.view;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_warnQueue_d423c59f248b91e5 = function() { return handleError(function (arg0, arg1) {
        warnQueue(getStringFromWasm0(arg0, arg1));
    }, arguments) };
    imports.wbg.__wbg_warn_4ca3906c248c47c4 = function(arg0) {
        console.warn(arg0);
    };
    imports.wbg.__wbg_wbgtestinvoke_d8e1edfc10304da7 = function() { return handleError(function (arg0, arg1) {
        try {
            var state0 = {a: arg0, b: arg1};
            var cb0 = () => {
                const a = state0.a;
                state0.a = 0;
                try {
                    return __wbg_adapter_1295(a, state0.b, );
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
    imports.wbg.__wbg_weakAddonOwner_c0b911801879ee25 = function() { return handleError(function (arg0) {
        const ret = weakAddonOwner(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_wheelReentryAndReplacement_c8fc4053cd0f93cc = function() { return handleError(function (arg0) {
        const ret = wheelReentryAndReplacement(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_wheelResultsAndThrows_654f6a1248689e9f = function() { return handleError(function (arg0) {
        const ret = wheelResultsAndThrows(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_width_5dde457d606ba683 = function(arg0) {
        const ret = arg0.width;
        return ret;
    };
    imports.wbg.__wbg_width_f0759bd8bad335bd = function(arg0) {
        const ret = arg0.width;
        return ret;
    };
    imports.wbg.__wbg_win32TemplateText_407e1363da5a5ddf = function() { return handleError(function (arg0) {
        const ret = win32TemplateText(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_xtermAssignProperty_7fdfb54b8dba6581 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        xtermAssignProperty(arg0, getStringFromWasm0(arg1, arg2), arg3);
    }, arguments) };
    imports.wbg.__wbg_xtermBindLogger_eeb14a8ffc2feafa = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
        const ret = xtermBindLogger(arg0, getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4), arg5);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_xtermConsoleMethod_7c1dfb0c7c696eb5 = function() { return handleError(function (arg0, arg1) {
        const ret = xtermConsoleMethod(getStringFromWasm0(arg0, arg1));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_xtermInvokeLog_bc6b1226f5d30537 = function() { return handleError(function (arg0, arg1, arg2) {
        xtermInvokeLog(arg0, arg1, arg2);
    }, arguments) };
    imports.wbg.__wbg_xtermLogPropertyKey_18d9ccbcab8f6684 = function() { return handleError(function (arg0) {
        const ret = xtermLogPropertyKey(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_xtermMeasureNumber_67b90aa9e8c89bb7 = function() { return handleError(function (arg0) {
        const ret = xtermMeasureNumber(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_xtermMouseBit_e358ea383d421bf4 = function() { return handleError(function (arg0, arg1) {
        const ret = xtermMouseBit(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_xtermMouseDivide_d19fb0f7f96d139d = function() { return handleError(function (arg0, arg1) {
        const ret = xtermMouseDivide(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_xtermMouseLess_1acaa8fe9dbba3b9 = function() { return handleError(function (arg0, arg1) {
        const ret = xtermMouseLess(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_xtermMouseMultiply_f63b5e1b4501c201 = function() { return handleError(function (arg0, arg1) {
        const ret = xtermMouseMultiply(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_xtermMouseNumber_52574b84147d2fde = function() { return handleError(function (arg0) {
        const ret = xtermMouseNumber(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_xtermMousePadding_f35b9e57e1dd9ec7 = function() { return handleError(function (arg0) {
        const ret = xtermMousePadding(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_xtermMouseProperty_2f8b5ebba8d2fd70 = function() { return handleError(function (arg0, arg1) {
        const ret = xtermMouseProperty(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_xtermMouseSubtract_80e399d8b82b32dd = function() { return handleError(function (arg0, arg1) {
        const ret = xtermMouseSubtract(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_xtermMouseWindow_a739abfee81bc635 = function() { return handleError(function () {
        const ret = xtermMouseWindow();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_xtermOffscreenCanvasAvailable_5c44f47c0e072082 = function() { return handleError(function () {
        const ret = xtermOffscreenCanvasAvailable();
        return ret;
    }, arguments) };
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
    imports.wbg.__wbindgen_closure_wrapper20795 = function(arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 1862, __wbg_adapter_93);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper20797 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 1860, __wbg_adapter_96);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper20799 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 1864, __wbg_adapter_99);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper28152 = function(arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 2111, __wbg_adapter_102);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper28154 = function(arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 2124, __wbg_adapter_105);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper28156 = function(arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 2109, __wbg_adapter_108);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper28158 = function(arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 2126, __wbg_adapter_111);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper28160 = function(arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 2128, __wbg_adapter_114);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper28162 = function(arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 2122, __wbg_adapter_117);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper28164 = function(arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 2132, __wbg_adapter_120);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper28166 = function(arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 2130, __wbg_adapter_123);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper28168 = function(arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 2134, __wbg_adapter_126);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper48565 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 2923, __wbg_adapter_129);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper5479 = function(arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 624, __wbg_adapter_42);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper5481 = function(arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 651, __wbg_adapter_45);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper5483 = function(arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 647, __wbg_adapter_48);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper5485 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 661, __wbg_adapter_51);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper5487 = function(arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 622, __wbg_adapter_54);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper5489 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 667, __wbg_adapter_57);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper5491 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 663, __wbg_adapter_60);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper5493 = function(arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 620, __wbg_adapter_63);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper5495 = function(arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 657, __wbg_adapter_66);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper5497 = function(arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 655, __wbg_adapter_69);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper5499 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 643, __wbg_adapter_72);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper5501 = function(arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 649, __wbg_adapter_75);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper5503 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 669, __wbg_adapter_78);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper5505 = function(arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 653, __wbg_adapter_81);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper5507 = function(arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 645, __wbg_adapter_84);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper5509 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 665, __wbg_adapter_87);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper5511 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 659, __wbg_adapter_90);
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
    imports.wbg.__wbindgen_is_symbol = function(arg0) {
        const ret = typeof(arg0) === 'symbol';
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
    imports['./snippets/xterm-wasm-support-11516dc9b8bb3743/inline0.js'] = __wbg_star0;
    imports['./snippets/xterm-wasm-support-11516dc9b8bb3743/inline4.js'] = __wbg_star1;
    imports['./snippets/xterm-wasm-support-11516dc9b8bb3743/inline5.js'] = __wbg_star2;
    imports['./snippets/xterm-wasm-support-11516dc9b8bb3743/inline6.js'] = __wbg_star3;
    imports['./snippets/xterm-wasm-support-11516dc9b8bb3743/inline7.js'] = __wbg_star4;
    imports['./snippets/xterm-wasm-support-11516dc9b8bb3743/src/public_api/addon_bridge.js'] = __wbg_star5;
    imports['./snippets/xterm-web-f147851b6cb0c0ef/inline0.js'] = __wbg_star6;
    imports['./snippets/xterm-web-f147851b6cb0c0ef/inline11.js'] = __wbg_star7;
    imports['./snippets/xterm-web-f147851b6cb0c0ef/inline17.js'] = __wbg_star8;
    imports['./snippets/xterm-web-f147851b6cb0c0ef/inline31.js'] = __wbg_star9;
    imports['./snippets/xterm-web-f147851b6cb0c0ef/src/input/kitty_tables.js'] = __wbg_star10;
    imports['./snippets/xterm-web-f147851b6cb0c0ef/src/input/win32_tables.js'] = __wbg_star11;
    imports['./snippets/xterm-web-f147851b6cb0c0ef/src/links/raw_link_bridge.js'] = __wbg_star12;
    imports['./snippets/xterm-web-f147851b6cb0c0ef/src/task_queue_bridge.js'] = __wbg_star13;
    imports['./snippets/xterm-web-f147851b6cb0c0ef/src/write_scheduler_bridge.js'] = __wbg_star14;

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
