// Copyright (c) 2018-2026 The xterm.js authors. All rights reserved. MIT license.
// The original constructor installs the actual event closures. Only GPU/model
// initialization is replaced, since this suite isolates context/observer phases.
import { Terminal } from 'browser/public/Terminal';
import { WebglRenderer } from 'source-webgl/WebglRenderer';

export function sourceContextFactory(document: Document, preserve: boolean | undefined, host: any): any {
  const terminal = new Terminal({ cols: 6, rows: 3, cursorBlink: false, logLevel: 'off' });
  const parent = document.createElement('div');
  document.body.appendChild(parent);
  terminal.open(parent);
  const core = (terminal as any)._core;
  const prototype = WebglRenderer.prototype as any;
  const initialize = prototype._initializeWebGLState;
  let renderer: any;
  try {
    prototype._initializeWebGLState = () => [undefined, undefined];
    renderer = new WebglRenderer(terminal, core._characterJoinerService,
      core._charSizeService, core._coreBrowserService, core.coreService,
      core._decorationService, core._logService, core.optionsService,
      core._themeService, true, preserve);
  } catch (error) {
    terminal.dispose(); parent.remove(); throw error;
  } finally { prototype._initializeWebGLState = initialize; }
  renderer._logService = {
    debug: () => host.log('lost'),
    warn: (message: string) => host.log(message.startsWith('webglcontextrestored') ? 'restored' : 'not-restored')
  };
  // This callback represents the cache-release and GPU-initialization portion,
  // which are excluded from both test paths, not replaced by a fake renderer.
  renderer._initializeWebGLState = () => { host.restore(); return [undefined, undefined]; };
  renderer._cursorBlinkStateManager.value = { restartBlinkAnimation: () => host.cursor(), dispose() {} };
  Object.defineProperty(terminal, 'rows', { get: () => host.rows() });
  renderer.onRequestRedraw((value: any) => host.redraw(value.start, value.end, value.sync ?? false));
  renderer.onContextLoss((event: Event) => host.loss(event));
  return {
    canvas: renderer._canvas,
    gl: renderer._gl,
    resize: (width: any, height: any) => renderer._setCanvasDevicePixelDimensions(width, height),
    window: (value: Window) => { core._coreBrowserService.window = value; },
    dispose: () => renderer.dispose(),
    cleanup: () => { renderer.dispose(); terminal.dispose(); parent.remove(); }
  };
}
