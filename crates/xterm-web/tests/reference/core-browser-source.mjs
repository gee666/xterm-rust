// Copyright (c) 2014-2026 The xterm.js authors. MIT license.
// Generated unchanged test-only oracle, never production code.
// Source hashes and license provenance are recorded in core-browser-source.json.

// source_repo/src/common/Lifecycle.ts
function toDisposable(fn) {
  return { dispose: fn };
}
var DisposableStore = class {
  constructor() {
    this._disposables = /* @__PURE__ */ new Set();
    this._isDisposed = false;
  }
  get isDisposed() {
    return this._isDisposed;
  }
  add(o) {
    if (this._isDisposed) {
      o.dispose();
    } else {
      this._disposables.add(o);
    }
    return o;
  }
  dispose() {
    if (this._isDisposed) {
      return;
    }
    this._isDisposed = true;
    for (const d of this._disposables) {
      d.dispose();
    }
    this._disposables.clear();
  }
  clear() {
    for (const d of this._disposables) {
      d.dispose();
    }
    this._disposables.clear();
  }
};
var Disposable = class {
  constructor() {
    this._store = new DisposableStore();
  }
  static {
    this.None = Object.freeze({ dispose() {
    } });
  }
  dispose() {
    this._store.dispose();
  }
  _register(o) {
    return this._store.add(o);
  }
};
var MutableDisposable = class {
  constructor() {
    this._isDisposed = false;
  }
  get value() {
    return this._isDisposed ? void 0 : this._value;
  }
  set value(value) {
    if (this._isDisposed || value === this._value) {
      return;
    }
    this._value?.dispose();
    this._value = value;
  }
  clear() {
    this.value = void 0;
  }
  dispose() {
    this._isDisposed = true;
    this._value?.dispose();
    this._value = void 0;
  }
};

// source_repo/src/common/Event.ts
var Emitter = class {
  constructor() {
    this._listeners = [];
    this._disposed = false;
  }
  get event() {
    if (this._event) {
      return this._event;
    }
    this._event = (listener, thisArgs, disposables) => {
      if (this._disposed) {
        return toDisposable(() => {
        });
      }
      const entry = { fn: listener, thisArgs };
      this._listeners = this._listeners.slice();
      this._listeners.push(entry);
      const result = toDisposable(() => {
        const idx = this._listeners.indexOf(entry);
        if (idx !== -1) {
          this._listeners = this._listeners.slice();
          this._listeners.splice(idx, 1);
        }
      });
      if (disposables) {
        if (Array.isArray(disposables)) {
          disposables.push(result);
        } else {
          disposables.add(result);
        }
      }
      return result;
    };
    return this._event;
  }
  fire(event) {
    if (this._disposed || !this._listeners.length) {
      return;
    }
    if (this._listeners.length === 1) {
      this._listeners[0].fn.call(this._listeners[0].thisArgs, event);
      return;
    }
    const listeners = this._listeners;
    for (let i = 0, len = listeners.length; i < len; ++i) {
      listeners[i].fn.call(listeners[i].thisArgs, event);
    }
  }
  dispose() {
    if (this._disposed) {
      return;
    }
    this._disposed = true;
    this._listeners.length = 0;
  }
};
var EventUtils;
((EventUtils2) => {
  function forward(from, to) {
    return from((e) => to.fire(e));
  }
  EventUtils2.forward = forward;
  function map(event, map2) {
    return (listener, thisArgs, disposables) => {
      return event((i) => listener.call(thisArgs, map2(i)), void 0, disposables);
    };
  }
  EventUtils2.map = map;
  function any(...events) {
    return (listener, thisArgs, disposables) => {
      const store = new DisposableStore();
      for (const event of events) {
        store.add(event((e) => listener.call(thisArgs, e)));
      }
      if (disposables) {
        if (Array.isArray(disposables)) {
          disposables.push(store);
        } else {
          disposables.add(store);
        }
      }
      return store;
    };
  }
  EventUtils2.any = any;
  function runAndSubscribe(event, handler, initial) {
    handler(initial);
    return event((e) => handler(e));
  }
  EventUtils2.runAndSubscribe = runAndSubscribe;
})(EventUtils || (EventUtils = {}));

// source_repo/src/browser/Dom.ts
var DomListener = class {
  constructor(node, type, handler, options) {
    this._node = node;
    this._type = type;
    this._handler = handler;
    this._options = options;
    node.addEventListener(type, handler, options);
  }
  dispose() {
    if (!this._node || !this._handler) {
      return;
    }
    this._node.removeEventListener(this._type, this._handler, this._options);
    this._node = null;
    this._handler = null;
  }
};
function addDisposableListener(node, type, handler, useCaptureOrOptions) {
  return new DomListener(node, type, handler, useCaptureOrOptions);
}

// source_repo/src/browser/services/CoreBrowserService.ts
var CoreBrowserService = class extends Disposable {
  constructor(_textarea, _window, mainDocument) {
    super();
    this._textarea = _textarea;
    this._window = _window;
    this.mainDocument = mainDocument;
    this._isFocused = false;
    this._cachedIsFocused = void 0;
    this._onDprChange = this._register(new Emitter());
    this.onDprChange = this._onDprChange.event;
    this._onWindowChange = this._register(new Emitter());
    this.onWindowChange = this._onWindowChange.event;
    this._screenDprMonitor = this._register(new ScreenDprMonitor(this._window));
    this._register(this.onWindowChange((w) => this._screenDprMonitor.setWindow(w)));
    this._register(EventUtils.forward(this._screenDprMonitor.onDprChange, this._onDprChange));
    this._register(addDisposableListener(this._textarea, "focus", () => this._isFocused = true));
    this._register(addDisposableListener(this._textarea, "blur", () => this._isFocused = false));
  }
  get window() {
    return this._window;
  }
  set window(value) {
    if (this._window !== value) {
      this._window = value;
      this._onWindowChange.fire(this._window);
    }
  }
  get dpr() {
    return this.window.devicePixelRatio;
  }
  get isFocused() {
    if (this._cachedIsFocused === void 0) {
      this._cachedIsFocused = this._isFocused && this._textarea.ownerDocument.hasFocus();
      queueMicrotask(() => this._cachedIsFocused = void 0);
    }
    return this._cachedIsFocused;
  }
};
var ScreenDprMonitor = class extends Disposable {
  constructor(_parentWindow) {
    super();
    this._parentWindow = _parentWindow;
    this._windowResizeListener = this._register(new MutableDisposable());
    this._onDprChange = this._register(new Emitter());
    this.onDprChange = this._onDprChange.event;
    this._outerListener = () => this._setDprAndFireIfDiffers();
    this._currentDevicePixelRatio = this._parentWindow.devicePixelRatio;
    this._updateDpr();
    this._setWindowResizeListener();
    this._register(toDisposable(() => this.clearListener()));
  }
  setWindow(parentWindow) {
    this._parentWindow = parentWindow;
    this._setWindowResizeListener();
    this._setDprAndFireIfDiffers();
  }
  _setWindowResizeListener() {
    this._windowResizeListener.value = addDisposableListener(this._parentWindow, "resize", () => this._setDprAndFireIfDiffers());
  }
  _setDprAndFireIfDiffers() {
    if (this._parentWindow.devicePixelRatio !== this._currentDevicePixelRatio) {
      this._onDprChange.fire(this._parentWindow.devicePixelRatio);
    }
    this._updateDpr();
  }
  _updateDpr() {
    if (!this._outerListener) {
      return;
    }
    this._resolutionMediaMatchList?.removeListener(this._outerListener);
    this._currentDevicePixelRatio = this._parentWindow.devicePixelRatio;
    this._resolutionMediaMatchList = this._parentWindow.matchMedia(`screen and (resolution: ${this._parentWindow.devicePixelRatio}dppx)`);
    this._resolutionMediaMatchList.addListener(this._outerListener);
  }
  clearListener() {
    if (!this._resolutionMediaMatchList || !this._outerListener) {
      return;
    }
    this._resolutionMediaMatchList.removeListener(this._outerListener);
    this._resolutionMediaMatchList = void 0;
    this._outerListener = void 0;
  }
};
export {
  CoreBrowserService
};
/**
 * Copyright (c) 2024-2026 The xterm.js authors. All rights reserved.
 * @license MIT
 *
 * Minimal lifecycle utilities for xterm.js core.
 * Simplified from VS Code's lifecycle.ts - no tracking/leak detection.
 */
/**
 * Copyright (c) 2024-2026 The xterm.js authors. All rights reserved.
 * @license MIT
 *
 * Minimal event utilities for xterm.js core.
 * Simplified from VS Code's event.ts - no leak detection/profiling.
 */
/**
 * Copyright (c) 2026 The xterm.js authors. All rights reserved.
 * @license MIT
 *
 * Minimal async helpers for xterm.js core.
 */
/**
 * Copyright (c) 2026 The xterm.js authors. All rights reserved.
 * @license MIT
 *
 * Minimal DOM helpers for xterm.js browser code.
 */
/**
 * Copyright (c) 2019 The xterm.js authors. All rights reserved.
 * @license MIT
 */
