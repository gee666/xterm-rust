// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
// Generated test-only intact source oracle. SHA-256: 17edb34b5e757cc7a20ece8384ce44d8bcaccf09b36b081f5da5ec985b2805fa
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __decorateClass = (decorators, target, key, kind) => {
  var result = kind > 1 ? void 0 : kind ? __getOwnPropDesc(target, key) : target;
  for (var i = decorators.length - 1, decorator; i >= 0; i--)
    if (decorator = decorators[i])
      result = (kind ? decorator(target, key, result) : decorator(result)) || result;
  if (kind && result) __defProp(target, key, result);
  return result;
};
var __decorateParam = (index, decorator) => (target, key) => decorator(target, key, index);

// source_repo/src/browser/RenderDebouncer.ts
var RenderDebouncer = class {
  constructor(_renderCallback, _coreBrowserService) {
    this._renderCallback = _renderCallback;
    this._coreBrowserService = _coreBrowserService;
  }
  _renderCallback;
  _coreBrowserService;
  _rowStart;
  _rowEnd;
  _rowCount;
  _animationFrame;
  _refreshCallbacks = [];
  dispose() {
    if (this._animationFrame !== void 0) {
      this._coreBrowserService.window.cancelAnimationFrame(this._animationFrame);
      this._animationFrame = void 0;
    }
  }
  addRefreshCallback(callback) {
    this._refreshCallbacks.push(callback);
    this._animationFrame ??= this._coreBrowserService.window.requestAnimationFrame(() => this._innerRefresh());
    return this._animationFrame;
  }
  refresh(rowStart, rowEnd, rowCount) {
    this._rowCount = rowCount;
    rowStart = rowStart ?? 0;
    rowEnd = rowEnd ?? this._rowCount - 1;
    this._rowStart = this._rowStart !== void 0 ? Math.min(this._rowStart, rowStart) : rowStart;
    this._rowEnd = this._rowEnd !== void 0 ? Math.max(this._rowEnd, rowEnd) : rowEnd;
    if (this._animationFrame !== void 0) {
      return;
    }
    this._animationFrame = this._coreBrowserService.window.requestAnimationFrame(() => this._innerRefresh());
  }
  _innerRefresh() {
    this._animationFrame = void 0;
    if (this._rowStart === void 0 || this._rowEnd === void 0 || this._rowCount === void 0) {
      this._runRefreshCallbacks();
      return;
    }
    const start = Math.max(this._rowStart, 0);
    const end = Math.min(this._rowEnd, this._rowCount - 1);
    this._rowStart = void 0;
    this._rowEnd = void 0;
    this._renderCallback(start, end);
    this._runRefreshCallbacks();
  }
  _runRefreshCallbacks() {
    for (const callback of this._refreshCallbacks) {
      callback(0);
    }
    this._refreshCallbacks = [];
  }
};

// source_repo/src/common/services/ServiceRegistry.ts
var serviceRegistry = /* @__PURE__ */ new Map();
function createDecorator(id) {
  if (serviceRegistry.has(id)) {
    return serviceRegistry.get(id);
  }
  const decorator = function(target, key, index) {
    if (arguments.length !== 3) {
      throw new Error("@IServiceName-decorator can only be used to decorate a parameter");
    }
    storeServiceDependency(decorator, target, index);
  };
  decorator._id = id;
  serviceRegistry.set(id, decorator);
  return decorator;
}
function storeServiceDependency(id, target, index) {
  if (target["di$target" /* DI_TARGET */] === target) {
    target["di$dependencies" /* DI_DEPENDENCIES */].push({ id, index });
  } else {
    target["di$dependencies" /* DI_DEPENDENCIES */] = [{ id, index }];
    target["di$target" /* DI_TARGET */] = target;
  }
}

// source_repo/src/browser/services/Services.ts
var ICharSizeService = createDecorator("CharSizeService");
var ICoreBrowserService = createDecorator("CoreBrowserService");
var IMouseCoordsService = createDecorator("MouseCoordsService");
var IMouseService = createDecorator("MouseService");
var IRenderService = createDecorator("RenderService");
var ISelectionService = createDecorator("SelectionService");
var ICharacterJoinerService = createDecorator("CharacterJoinerService");
var IThemeService = createDecorator("ThemeService");
var ILinkProviderService = createDecorator("LinkProviderService");
var IKeyboardService = createDecorator("KeyboardService");

// source_repo/src/common/Lifecycle.ts
function toDisposable(fn) {
  return { dispose: fn };
}
var DisposableStore = class {
  _disposables = /* @__PURE__ */ new Set();
  _isDisposed = false;
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
  static None = Object.freeze({ dispose() {
  } });
  _store = new DisposableStore();
  dispose() {
    this._store.dispose();
  }
  _register(o) {
    return this._store.add(o);
  }
};
var MutableDisposable = class {
  _value;
  _isDisposed = false;
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

// source_repo/src/common/TaskQueue.ts
var TaskQueue = class {
  _tasks = [];
  _idleCallback;
  _i = 0;
  _logService;
  constructor(logService) {
    this._logService = logService;
  }
  enqueue(task) {
    this._tasks.push(task);
    this._start();
  }
  flush() {
    while (this._i < this._tasks.length) {
      if (!this._tasks[this._i]()) {
        this._i++;
      }
    }
    this.clear();
  }
  clear() {
    if (this._idleCallback) {
      this._cancelCallback(this._idleCallback);
      this._idleCallback = void 0;
    }
    this._i = 0;
    this._tasks.length = 0;
  }
  _start() {
    if (!this._idleCallback) {
      this._idleCallback = this._requestCallback(this._process.bind(this));
    }
  }
  _process(deadline) {
    this._idleCallback = void 0;
    let taskDuration;
    let longestTask = 0;
    let lastDeadlineRemaining = deadline.timeRemaining();
    let deadlineRemaining;
    while (this._i < this._tasks.length) {
      taskDuration = performance.now();
      if (!this._tasks[this._i]()) {
        this._i++;
      }
      taskDuration = Math.max(1, performance.now() - taskDuration);
      longestTask = Math.max(taskDuration, longestTask);
      deadlineRemaining = deadline.timeRemaining();
      if (longestTask * 1.5 > deadlineRemaining) {
        if (lastDeadlineRemaining - taskDuration < -20) {
          this._logService.warn(`task queue exceeded allotted deadline by ${Math.abs(Math.round(lastDeadlineRemaining - taskDuration))}ms`);
        }
        this._start();
        return;
      }
      lastDeadlineRemaining = deadlineRemaining;
    }
    this.clear();
  }
};
var PriorityTaskQueue = class extends TaskQueue {
  _requestCallback(callback) {
    return setTimeout(() => callback(this._createDeadline(16)));
  }
  _cancelCallback(identifier) {
    clearTimeout(identifier);
  }
  _createDeadline(duration) {
    const end = performance.now() + duration;
    return {
      timeRemaining: () => Math.max(0, end - performance.now())
    };
  }
};
var IdleTaskQueueInternal = class extends TaskQueue {
  _requestCallback(callback) {
    return requestIdleCallback(callback);
  }
  _cancelCallback(identifier) {
    cancelIdleCallback(identifier);
  }
};
var IdleTaskQueue = "requestIdleCallback" in globalThis ? IdleTaskQueueInternal : PriorityTaskQueue;
var DebouncedIdleTask = class {
  _queue;
  constructor(logService) {
    this._queue = new IdleTaskQueue(logService);
  }
  set(task) {
    this._queue.clear();
    this._queue.enqueue(task);
  }
  flush() {
    this._queue.flush();
  }
  dispose() {
    this._queue.clear();
  }
};

// source_repo/src/common/services/Services.ts
var IBufferService = createDecorator("BufferService");
var IMouseStateService = createDecorator("MouseStateService");
var ICoreService = createDecorator("CoreService");
var ICharsetService = createDecorator("CharsetService");
var IInstantiationService = createDecorator("InstantiationService");
var ILogService = createDecorator("LogService");
var IOptionsService = createDecorator("OptionsService");
var IOscLinkService = createDecorator("OscLinkService");
var IUnicodeService = createDecorator("UnicodeService");
var IDecorationService = createDecorator("DecorationService");

// source_repo/src/common/Event.ts
var Emitter = class {
  _listeners = [];
  _disposed = false;
  _event;
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

// source_repo/src/browser/services/RenderService.ts
var RenderService = class extends Disposable {
  constructor(_rowCount, screenElement, _optionsService, _logService, _charSizeService, _coreService, decorationService, bufferService, _coreBrowserService, themeService) {
    super();
    this._rowCount = _rowCount;
    this._optionsService = _optionsService;
    this._logService = _logService;
    this._charSizeService = _charSizeService;
    this._coreService = _coreService;
    this._coreBrowserService = _coreBrowserService;
    this._pausedResizeTask = this._register(new DebouncedIdleTask(this._logService));
    this._renderDebouncer = new RenderDebouncer((start, end) => this._renderRows(start, end), this._coreBrowserService);
    this._register(this._renderDebouncer);
    this._syncOutputHandler = new SynchronizedOutputHandler(
      this._coreBrowserService,
      this._coreService,
      () => this._fullRefresh()
    );
    this._register(toDisposable(() => this._syncOutputHandler.dispose()));
    this._register(this._coreBrowserService.onDprChange(() => this.handleDevicePixelRatioChange()));
    this._register(bufferService.onResize(() => this._fullRefresh()));
    this._register(bufferService.buffers.onBufferActivate(() => this._renderer.value?.clear()));
    this._register(this._optionsService.onOptionChange(() => this._handleOptionsChanged()));
    this._register(this._charSizeService.onCharSizeChange(() => this.handleCharSizeChanged()));
    this._register(decorationService.onDecorationRegistered(() => this._fullRefresh()));
    this._register(decorationService.onDecorationRemoved(() => this._fullRefresh()));
    this._register(this._optionsService.onMultipleOptionChange([
      "drawBoldTextInBrightColors",
      "letterSpacing",
      "lineHeight",
      "fontFamily",
      "fontSize",
      "fontWeight",
      "fontWeightBold",
      "minimumContrastRatio",
      "rescaleOverlappingGlyphs"
    ], () => {
      this.clear();
      this.handleResize(bufferService.cols, bufferService.rows);
      this._fullRefresh();
    }));
    this._register(this._optionsService.onMultipleOptionChange([
      "cursorBlink",
      "cursorStyle"
    ], () => this.refreshRows(bufferService.buffer.y, bufferService.buffer.y, void 0, true)));
    this._register(themeService.onChangeColors(() => this._fullRefresh()));
    this._registerIntersectionObserver(this._coreBrowserService.window, screenElement);
    this._register(this._coreBrowserService.onWindowChange((w) => this._registerIntersectionObserver(w, screenElement)));
  }
  _rowCount;
  _optionsService;
  _logService;
  _charSizeService;
  _coreService;
  _coreBrowserService;
  serviceBrand;
  _renderer = this._register(new MutableDisposable());
  _renderDebouncer;
  _pausedResizeTask;
  _observerDisposable = this._register(new MutableDisposable());
  _intersectionObserver;
  _isPaused = false;
  _needsFullRefresh = false;
  _isNextRenderRedrawOnly = true;
  _needsSelectionRefresh = false;
  _canvasWidth = 0;
  _canvasHeight = 0;
  _syncOutputHandler;
  _selectionState = {
    start: void 0,
    end: void 0,
    columnSelectMode: false
  };
  _onDimensionsChange = this._register(new Emitter());
  onDimensionsChange = this._onDimensionsChange.event;
  _onRenderedViewportChange = this._register(new Emitter());
  onRenderedViewportChange = this._onRenderedViewportChange.event;
  _onRender = this._register(new Emitter());
  onRender = this._onRender.event;
  _onRefreshRequest = this._register(new Emitter());
  onRefreshRequest = this._onRefreshRequest.event;
  get dimensions() {
    return this._renderer.value.dimensions;
  }
  _registerIntersectionObserver(w, screenElement) {
    if ("IntersectionObserver" in w) {
      const observer = new w.IntersectionObserver((e) => this._handleIntersectionChange(e[e.length - 1]), { threshold: 0 });
      this._observerDisposable.value = toDisposable(() => {
        this._intersectionObserver?.disconnect();
        this._intersectionObserver = void 0;
      });
      this._intersectionObserver = observer;
      observer.observe(screenElement);
    }
  }
  _handleIntersectionChange(entry) {
    this._isPaused = entry.isIntersecting === void 0 ? entry.intersectionRatio === 0 : !entry.isIntersecting;
    this._renderer.value?.handleViewportVisibilityChange?.(!this._isPaused);
    if (!this._isPaused && !this._charSizeService.hasValidSize) {
      this._charSizeService.measure();
    }
    if (!this._isPaused && this._needsFullRefresh) {
      this._pausedResizeTask.flush();
      this.refreshRows(0, this._rowCount - 1);
      this._needsFullRefresh = false;
    }
  }
  refreshRows(start, end, sync = false, isRedrawOnly = false) {
    if (this._isPaused) {
      this._needsFullRefresh = true;
      return;
    }
    if (this._coreService.decPrivateModes.synchronizedOutput) {
      this._syncOutputHandler.bufferRows(start, end);
      return;
    }
    const buffered = this._syncOutputHandler.flush();
    if (buffered) {
      start = Math.min(start, buffered.start);
      end = Math.max(end, buffered.end);
    }
    if (!isRedrawOnly) {
      this._isNextRenderRedrawOnly = false;
    }
    if (sync) {
      this._renderRows(start, end);
    } else {
      this._renderDebouncer.refresh(start, end, this._rowCount);
    }
  }
  _renderRows(start, end) {
    if (!this._renderer.value) {
      return;
    }
    if (this._coreService.decPrivateModes.synchronizedOutput) {
      this._syncOutputHandler.bufferRows(start, end);
      return;
    }
    start = Math.min(start, this._rowCount - 1);
    end = Math.min(end, this._rowCount - 1);
    this._renderer.value.renderRows(start, end);
    if (this._needsSelectionRefresh) {
      this._renderer.value.handleSelectionChanged(this._selectionState.start, this._selectionState.end, this._selectionState.columnSelectMode);
      this._needsSelectionRefresh = false;
    }
    if (!this._isNextRenderRedrawOnly) {
      this._onRenderedViewportChange.fire({ start, end });
    }
    this._onRender.fire({ start, end });
    this._isNextRenderRedrawOnly = true;
  }
  resize(cols, rows) {
    this._rowCount = rows;
    this._fireOnCanvasResize();
  }
  _handleOptionsChanged() {
    if (!this._renderer.value) {
      return;
    }
    this.refreshRows(0, this._rowCount - 1);
    this._fireOnCanvasResize();
  }
  _fireOnCanvasResize() {
    if (!this._renderer.value) {
      return;
    }
    if (this._renderer.value.dimensions.css.canvas.width === this._canvasWidth && this._renderer.value.dimensions.css.canvas.height === this._canvasHeight) {
      return;
    }
    this._onDimensionsChange.fire(this._renderer.value.dimensions);
  }
  hasRenderer() {
    return !!this._renderer.value;
  }
  setRenderer(renderer) {
    this._renderer.value = renderer;
    if (this._renderer.value) {
      this._renderer.value.onRequestRedraw((e) => this.refreshRows(e.start, e.end, e.sync, true));
      this._needsSelectionRefresh = true;
      this._fullRefresh();
    }
  }
  addRefreshCallback(callback) {
    return this._renderDebouncer.addRefreshCallback(callback);
  }
  _fullRefresh() {
    if (this._isPaused) {
      this._needsFullRefresh = true;
    } else {
      this.refreshRows(0, this._rowCount - 1);
    }
  }
  clearTextureAtlas() {
    if (!this._renderer.value) {
      return;
    }
    this._renderer.value.clearTextureAtlas?.();
    this._fullRefresh();
  }
  handleDevicePixelRatioChange() {
    this._charSizeService.measure();
    if (!this._renderer.value) {
      return;
    }
    this._renderer.value.handleDevicePixelRatioChange();
    this.refreshRows(0, this._rowCount - 1);
  }
  handleResize(cols, rows) {
    if (!this._renderer.value) {
      return;
    }
    if (this._isPaused) {
      this._pausedResizeTask.set(() => this._renderer.value?.handleResize(cols, rows));
    } else {
      this._renderer.value.handleResize(cols, rows);
    }
    this._fullRefresh();
  }
  // TODO: Is this useful when we have onResize?
  handleCharSizeChanged() {
    this._renderer.value?.handleCharSizeChanged();
  }
  handleBlur() {
    this._renderer.value?.handleBlur();
  }
  handleFocus() {
    this._renderer.value?.handleFocus();
  }
  handleSelectionChanged(start, end, columnSelectMode) {
    this._selectionState.start = start;
    this._selectionState.end = end;
    this._selectionState.columnSelectMode = columnSelectMode;
    this._renderer.value?.handleSelectionChanged(start, end, columnSelectMode);
  }
  handleCursorMove() {
    this._renderer.value?.handleCursorMove();
  }
  clear() {
    this._renderer.value?.clear();
  }
};
RenderService = __decorateClass([
  __decorateParam(2, IOptionsService),
  __decorateParam(3, ILogService),
  __decorateParam(4, ICharSizeService),
  __decorateParam(5, ICoreService),
  __decorateParam(6, IDecorationService),
  __decorateParam(7, IBufferService),
  __decorateParam(8, ICoreBrowserService),
  __decorateParam(9, IThemeService)
], RenderService);
var SynchronizedOutputHandler = class {
  constructor(_coreBrowserService, _coreService, _onTimeout) {
    this._coreBrowserService = _coreBrowserService;
    this._coreService = _coreService;
    this._onTimeout = _onTimeout;
  }
  _coreBrowserService;
  _coreService;
  _onTimeout;
  _start = 0;
  _end = 0;
  _timeout;
  _isBuffering = false;
  bufferRows(start, end) {
    if (!this._isBuffering) {
      this._start = start;
      this._end = end;
      this._isBuffering = true;
    } else {
      this._start = Math.min(this._start, start);
      this._end = Math.max(this._end, end);
    }
    this._timeout ??= this._coreBrowserService.window.setTimeout(() => {
      this._timeout = void 0;
      this._coreService.decPrivateModes.synchronizedOutput = false;
      this._onTimeout();
    }, 1e3 /* SYNCHRONIZED_OUTPUT_TIMEOUT_MS */);
  }
  flush() {
    if (this._timeout !== void 0) {
      this._coreBrowserService.window.clearTimeout(this._timeout);
      this._timeout = void 0;
    }
    if (!this._isBuffering) {
      return void 0;
    }
    const result = { start: this._start, end: this._end };
    this._isBuffering = false;
    return result;
  }
  dispose() {
    if (this._timeout !== void 0) {
      this._coreBrowserService.window.clearTimeout(this._timeout);
      this._timeout = void 0;
    }
  }
};

// render-sync-cancel-entry.ts
function makeSource(host) {
  const logs = [];
  const event = () => ({ dispose() {
  } });
  const core = { decPrivateModes: { synchronizedOutput: false } };
  const service = new RenderService(
    3,
    {},
    { onOptionChange: event, onMultipleOptionChange: event },
    { warn() {
    } },
    { hasValidSize: true, onCharSizeChange: event },
    core,
    { onDecorationRegistered: event, onDecorationRemoved: event },
    { onResize: event, buffers: { onBufferActivate: event } },
    { window: host.window, onDprChange: event, onWindowChange: event },
    { onChangeColors: event }
  );
  service.onRenderedViewportChange((e) => logs.push("viewport:" + e.start + ":" + e.end));
  service.onRender((e) => logs.push("render:" + e.start + ":" + e.end));
  service.setRenderer({
    onRequestRedraw: event,
    dispose() {
      logs.push("A.dispose");
    },
    renderRows(start, end) {
      logs.push("A.rows:" + start + ":" + end);
    },
    handleSelectionChanged() {
      logs.push("A.selection:-:-:false");
    }
  });
  host.frame();
  logs.length = 0;
  return (command) => {
    switch (command.op) {
      case "mode":
        core.decPrivateModes.synchronizedOutput = command.value;
        return;
      case "refresh":
        return service.refreshRows(command.start, command.end, true, false);
      case "dispose":
        return service.dispose();
      case "state":
        return core.decPrivateModes.synchronizedOutput;
      case "drain":
        return logs.splice(0);
      default:
        throw new Error("Unknown command " + command.op);
    }
  };
}
export {
  makeSource
};
/**
 * Copyright (c) 2018 The xterm.js authors. All rights reserved.
 * @license MIT
 */
/**
 * Copyright (c) 2019 The xterm.js authors. All rights reserved.
 * @license MIT
 *
 * This was heavily inspired from microsoft/vscode's dependency injection system (MIT).
 */
/**
 * Copyright (c) 2019 The xterm.js authors. All rights reserved.
 * @license MIT
 */
/**
 * Copyright (c) 2024-2026 The xterm.js authors. All rights reserved.
 * @license MIT
 *
 * Minimal lifecycle utilities for xterm.js core.
 * Simplified from VS Code's lifecycle.ts - no tracking/leak detection.
 */
/**
 * Copyright (c) 2022 The xterm.js authors. All rights reserved.
 * @license MIT
 */
/**
 * Copyright (c) 2024-2026 The xterm.js authors. All rights reserved.
 * @license MIT
 *
 * Minimal event utilities for xterm.js core.
 * Simplified from VS Code's event.ts - no leak detection/profiling.
 */

// Test-only shared programs. They drive the real source or Rust RenderService.
import { withRenderHarness } from './render_service_host.mjs';
export const syncCancelPrograms = [
  'plain', 'get-buffer', 'call-buffer', 'get-flush', 'call-flush',
  'get-replace', 'call-replace', 'get-dispose', 'call-dispose',
  ...['get', 'call'].flatMap(phase => ['object', 'undefined', 'null', 'string'].flatMap(kind =>
    [`${phase}-throw-${kind}`, `${phase}-buffer-throw-${kind}`,
      `dispose-${phase}-throw-${kind}`, `dispose-${phase}-buffer-throw-${kind}`])),
  'call-deliver', 'get-deliver', 'call-buffer-deliver', 'get-buffer-deliver',
  'retained-after-clear', 'get-replace-retained', 'call-replace-retained',
  'method-own-call', ...['undefined', 'null', 'number', 'string', 'object'].map(kind => `method-noncallable-${kind}`)
];
export function runSyncCancelProgram(make, program) {
  return withRenderHarness(host => {
    const dispatch = make(host);
    const log = [];
    const command = (op, extra = {}) => {
      try { return dispatch({ op, ...extra }); }
      finally { log.push(...dispatch({ op: 'drain' })); }
    };
    const mode = value => command('mode', { value });
    const refresh = (start = 1, end = 2) => command('refresh', { start, end });
    const originalSet = host.window.setTimeout;
    const originalClear = host.window.clearTimeout;
    const failureKind = program.split('-').at(-1);
    const failure = ({ object: Object.freeze({ sentinel: program }), undefined: undefined,
      null: null, string: 'sync-clear-failure' })[failureKind];
    let fired = false;
    let malformedRead = false;
    let retained;
    host.window.setTimeout = function(fn, delay) {
      retained ??= fn;
      const id = originalSet.call(this, fn, delay);
      log.push(`set:${id}:${delay}:${this === host.window}`);
      return id;
    };
    function hook(phase) {
      if (fired || !program.replace(/^dispose-/, '').startsWith(phase + '-')) return;
      fired = true;
      log.push(`hook:${phase}`);
      if (program.includes('buffer')) { mode(true); refresh(0, 4); }
      if (program.includes('flush') || program.includes('replace')) {
        mode(false); refresh(3, 3);
      }
      if (program.includes('replace')) { mode(true); refresh(2, 4); }
      if (program.includes('dispose')) command('dispose');
      if (program.includes('deliver')) host.safety();
      if (program.includes('throw')) throw failure;
    }
    Object.defineProperty(host.window, 'clearTimeout', { configurable: true, get() {
      log.push('get-clear');
      hook('get');
      if (!malformedRead && program.startsWith('method-noncallable-')) {
        malformedRead = true;
        return ({ undefined: undefined, null: null, number: 0, string: 'clear',
          object: { call() { throw new Error('Noncallable .call must not be invoked'); } } })[failureKind];
      }
      const method = function(id) {
        log.push(`clear:${String(id)}:${this === host.window}`);
        hook('call');
        return originalClear.call(this, id);
      };
      if (program === 'method-own-call') method.call = () => { throw new Error('Own .call must not be invoked'); };
      return method;
    } });
    function attempt(label, action) {
      try { action(); log.push(`${label}:ok`); }
      catch (error) {
        if (program.startsWith('method-noncallable-') && error instanceof TypeError) log.push(`${label}:type-error`);
        else {
          if (!program.includes('throw') || error !== failure) throw error;
          log.push(`${label}:same-error`);
        }
      }
      log.push(`state:${command('state')}:${JSON.stringify(host.pending())}`);
    }
    try {
      mode(true); refresh();
      mode(false);
      if (program.startsWith('dispose-')) attempt('dispose-initial', () => command('dispose'));
      else attempt('flush', () => refresh(1, 1));
      // A failed cancellation must leave the original callback callable. Its
      // timeout delivery disables synchronized output and requests a refresh.
      if (program.includes('throw') || program.startsWith('method-noncallable-')) attempt('deliver-after-error', () => host.safety());
      if (program.includes('retained')) attempt('deliver-after-clear', () => retained());
      mode(true); attempt('buffer-again', () => refresh(2, 2));
      mode(false); attempt('flush-again', () => refresh(0, 0));
      attempt('dispose', () => command('dispose'));
      return log;
    } finally {
      Object.defineProperty(host.window, 'clearTimeout', { configurable: true, writable: true, value: originalClear });
      host.window.setTimeout = originalSet;
      command('dispose');
    }
  });
}
export function runSyncCancelCases(make) {
  return syncCancelPrograms.map(program => ({ program, log: runSyncCancelProgram(make, program) }));
}

// Test-only raw timer values shared by intact source and actual Rust RenderService.
import { withRenderHarness as withHandleHarness } from './render_service_host.mjs';
const handleKinds = ['undefined', 'null', 'false', 'true', 'zero', 'negative-zero',
  'fraction', 'nan', 'infinity', 'wide', 'string', 'empty-string', 'symbol', 'bigint', 'object', 'function'];
const handleActions = ['flush', 'dispose', 'deliver'];
const handlePhases = ['get-buffer', 'call-buffer', 'get-flush', 'call-flush',
  'get-dispose', 'call-dispose', 'call-deliver', 'clear-get-replace', 'clear-call-replace',
  ...['get', 'call'].flatMap(phase => ['object', 'undefined', 'null', 'string'].map(kind => `${phase}-throw-${kind}`))];
export const syncHandlePrograms = [
  ...handleKinds.flatMap(kind => handleActions.map(action => `${kind}/${action}`)),
  ...['object', 'symbol', 'null', 'undefined'].flatMap(kind => handlePhases.map(action => `${kind}/${action}`))
];
export function runSyncHandleProgram(make, program) {
  return withHandleHarness(host => {
    const dispatch = make(host);
    const log = [];
    const [kind, action] = program.split('/');
    const records = [];
    const command = (op, extra = {}) => {
      try { return dispatch({ op, ...extra }); }
      finally { log.push(...dispatch({ op: 'drain' })); }
    };
    const mode = value => command('mode', { value });
    const refresh = (start = 1, end = 2) => command('refresh', { start, end });
    const failureKind = action.split('-').at(-1);
    const failure = ({ object: Object.freeze({ sentinel: program }), undefined: undefined,
      null: null, string: 'sync-set-failure' })[failureKind];
    const originalSet = host.window.setTimeout;
    const originalClear = host.window.clearTimeout;
    let fired = false;
    function value(index) {
      const converted = () => { log.push(`unexpected-conversion:${index}`); throw new Error('Timer handle converted'); };
      switch (kind) {
        case 'undefined': return undefined;
        case 'null': return null;
        case 'false': return false;
        case 'true': return true;
        case 'zero': return 0;
        case 'negative-zero': return -0;
        case 'fraction': return 1.25;
        case 'nan': return NaN;
        case 'infinity': return Infinity;
        case 'wide': return 2 ** 40 + 17;
        case 'string': return 'raw-timer';
        case 'empty-string': return '';
        case 'symbol': return Symbol(`timer-${index}`);
        case 'bigint': return 2n ** 80n + BigInt(index);
        case 'object': return { [Symbol.toPrimitive]: converted, valueOf: converted, toString: converted };
        case 'function': return Object.assign(function() {}, { [Symbol.toPrimitive]: converted });
        default: throw new Error(`Unknown handle kind ${kind}`);
      }
    }
    function deliver(index) {
      const record = records[index];
      if (!record) throw new Error('Missing retained timer');
      log.push(`deliver:${index}:${record.cleared}`);
      try { record.fn(); host.frame(); }
      finally { log.push(...dispatch({ op: 'drain' })); }
    }
    function hook(phase) {
      if (fired || !action.startsWith(phase + '-')) return;
      fired = true;
      log.push(`hook:${phase}`);
      if (action.includes('buffer')) { mode(true); refresh(0, 4); }
      if (action.includes('flush') || action.includes('replace')) { mode(false); refresh(3, 3); }
      if (action.includes('replace')) { mode(true); refresh(2, 4); }
      if (action.includes('dispose')) command('dispose');
      if (action.includes('deliver')) deliver(records.length - 1);
      if (action.includes('throw')) throw failure;
    }
    Object.defineProperty(host.window, 'setTimeout', { configurable: true, get() {
      log.push('get-set');
      hook('get');
      const schedule = function(fn, delay) {
        const index = records.length;
        const handle = value(index);
        records.push({ fn, handle, cleared: false });
        log.push(`set:${index}:${delay}:${this === host.window}`);
        hook('call');
        return handle;
      };
      // Invocation must not use an overridable Function.prototype.call property.
      schedule.call = () => { throw new Error('setTimeout own .call invoked'); };
      return schedule;
    } });
    Object.defineProperty(host.window, 'clearTimeout', { configurable: true, get() {
      log.push('get-clear');
      hook('clear-get');
      return function(handle) {
        const index = records.findLastIndex(record => Object.is(record.handle, handle));
        log.push(`clear:${index}:${handle === undefined}:${Object.is(handle, -0)}:${this === host.window}`);
        if (index < 0 && handle !== undefined) throw new Error('Coerced or foreign timer handle');
        hook('clear-call');
        for (const record of records) if (Object.is(record.handle, handle)) record.cleared = true;
      };
    } });
    function attempt(label, callback) {
      try { callback(); log.push(`${label}:ok`); }
      catch (error) {
        if (!action.includes('throw') || error !== failure) throw error;
        log.push(`${label}:same-error`);
      }
      log.push(`state:${command('state')}:registrations:${records.length}:${JSON.stringify(host.pending())}`);
    }
    try {
      mode(true);
      attempt('buffer', () => refresh());
      attempt('buffer-again', () => refresh(0, 4));
      if (action === 'deliver' || action.includes('throw') && records.length) {
        attempt('retained', () => deliver(0));
      }
      mode(false);
      if (action === 'dispose') attempt('dispose-initial', () => command('dispose'));
      else attempt('flush', () => refresh(1, 1));
      // Snapshot the count so a broken callback that schedules more work fails
      // the trace comparison instead of growing an unbounded delivery loop.
      const retainedCount = records.length;
      for (let index = 0; index < retainedCount; index++) attempt(`retained-${index}`, () => deliver(index));
      mode(true); attempt('new-buffer', () => refresh(2, 2));
      mode(false); attempt('new-flush', () => refresh(0, 0));
      attempt('dispose', () => command('dispose'));
      return log;
    } finally {
      Object.defineProperty(host.window, 'setTimeout', { configurable: true, writable: true, value: originalSet });
      Object.defineProperty(host.window, 'clearTimeout', { configurable: true, writable: true, value: originalClear });
      command('dispose');
    }
  });
}
export function runSyncHandleCases(make) {
  return syncHandlePrograms.map(program => ({ program, log: runSyncHandleProgram(make, program) }));
}

export function sourceSyncCancelProgram(program) { return runSyncCancelProgram(makeSource, program); }
export function rustSyncCancelProgram(program, callback) { return runSyncCancelProgram(host => command => callback(host, command), program); }
export function sourceSyncHandleProgram(program) { return runSyncHandleProgram(makeSource, program); }
export function rustSyncHandleProgram(program, callback) { return runSyncHandleProgram(host => command => callback(host, command), program); }
