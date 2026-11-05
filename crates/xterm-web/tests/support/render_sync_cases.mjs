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
